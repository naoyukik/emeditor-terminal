use crate::gui::driver::ime_gui_driver::{
    handle_composition, handle_end_composition, handle_start_composition, sync_system_caret,
    CaretHandle, ImeResult,
};
use crate::gui::driver::keyboard_gui_driver::KeyboardGuiDriver;
use crate::gui::driver::scroll_gui_driver::{update_window_scroll_info, ScrollAction};
use crate::gui::driver::window_gui_driver::WindowGuiDriver;
use crate::gui::resolver::terminal_window_resolver::{get_terminal_data, TerminalWindowResolver};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, InvalidateRect, PAINTSTRUCT};
use windows::Win32::UI::Input::KeyboardAndMouse::VK_MENU;
use windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, DLGC_WANTALLKEYS};

const ISC_SHOWUICOMPOSITIONWINDOW: u32 = 0x80000000;

pub fn on_vscroll(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let data_arc = get_terminal_data();
    let action = {
        let mut window_data = data_arc.lock().unwrap();

        // Sync state before handling
        let history_count = window_data.service.get_history_count() as i32;
        let height = window_data.service.get_buffer().get_height() as i32;
        window_data.scroll_manager.max = history_count + height - 1;
        window_data.scroll_manager.page = height as u32;

        window_data
            .scroll_manager
            .handle_vscroll(wparam.0, lparam.0)
    };

    match action {
        ScrollAction::ScrollTo(pos) => {
            let mut window_data = data_arc.lock().unwrap();
            window_data.service.scroll_to(pos);
        }
        ScrollAction::ScrollBy(delta) => {
            let mut window_data = data_arc.lock().unwrap();
            window_data.service.scroll_lines(delta);
        }
        _ => {}
    }

    update_window_scroll_info(hwnd);
    unsafe {
        let _ = InvalidateRect(hwnd, None, BOOL(0));
    }
    LRESULT(0)
}

pub fn on_mousewheel(hwnd: HWND, wparam: WPARAM) -> LRESULT {
    let data_arc = get_terminal_data();
    let action = {
        let window_data = data_arc.lock().unwrap();
        window_data.scroll_manager.handle_mousewheel(wparam.0)
    };

    if let ScrollAction::ScrollBy(lines) = action {
        let mut window_data = data_arc.lock().unwrap();
        window_data.service.scroll_lines(lines);
    }

    update_window_scroll_info(hwnd);
    unsafe {
        let _ = InvalidateRect(hwnd, None, BOOL(0));
    }
    LRESULT(0)
}

pub fn on_paint(hwnd: HWND) -> LRESULT {
    unsafe {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);

        let data_arc = get_terminal_data();
        let mut window_data = data_arc.lock().unwrap();

        let mut client_rect = windows::Win32::Foundation::RECT::default();
        let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut client_rect);

        // Destructure to allow simultaneous mutable borrow of renderer and immutable borrow of service
        let TerminalWindowResolver {
            ref service,
            ref mut renderer,
            ref composition,
            ref caret,
            ..
        } = *window_data;

        renderer.render(
            hdc,
            &client_rect,
            service.get_buffer(),
            composition.as_ref(),
            &service.color_theme,
            &service.config,
        );

        // Always sync system caret position with virtual cursor during paint
        sync_system_caret(
            hwnd,
            service.get_buffer().get_ime_anchor_pos(),
            service.get_buffer().get_viewport_offset(),
            renderer,
            caret.as_ref(),
        );

        let _ = EndPaint(hwnd, &ps);
    }
    LRESULT(0)
}

pub fn on_lbuttondown(hwnd: HWND) -> LRESULT {
    log::info!("WM_LBUTTONDOWN: Setting focus");
    WindowGuiDriver::focus_existing_window(hwnd);
    LRESULT(0)
}

pub fn on_set_focus(hwnd: HWND) -> LRESULT {
    log::info!("WM_SETFOCUS: Focus received, installing keyboard hook and system caret");

    let data_arc = get_terminal_data();
    {
        let mut window_data = data_arc.lock().unwrap();
        let (char_width, char_height) = if let Some(metrics) = window_data.renderer.get_metrics() {
            (metrics.base_width, metrics.char_height)
        } else {
            (8, 16)
        };
        // Create RAII system caret handle with proper dimensions
        window_data.caret = Some(CaretHandle::new(hwnd, char_width, char_height));

        KeyboardGuiDriver::install(hwnd);
    }

    LRESULT(0)
}

pub fn on_kill_focus() -> LRESULT {
    log::info!("WM_KILLFOCUS: Focus lost, uninstalling keyboard hook and system caret");
    let data_arc = get_terminal_data();
    {
        let mut window_data = data_arc.lock().unwrap();
        // RAII handle will automatically call DestroyCaret when dropped
        window_data.caret = None;
        KeyboardGuiDriver::uninstall();
    }
    LRESULT(0)
}

pub fn on_keydown(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let vk_code = wparam.0 as u16;
    log::debug!("WM_KEYDOWN received: 0x{:04X}", vk_code);
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn on_syskeydown(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let vk_code = wparam.0 as u16;
    log::debug!("WM_SYSKEYDOWN received: 0x{:04X}", vk_code);

    if crate::gui::window::is_system_shortcut(vk_code, true) {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    if vk_code == VK_MENU.0 {
        return LRESULT(0);
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn on_syskeyup(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let vk_code = wparam.0 as u16;
    log::debug!("WM_SYSKEYUP received: 0x{:04X}", vk_code);

    if vk_code == VK_MENU.0 {
        log::debug!("Alt key (VK_MENU) released in WM_SYSKEYUP, suppressing DefWindowProcW");
        return LRESULT(0);
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn on_keyup(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let vk_code = wparam.0 as u16;
    if vk_code == VK_MENU.0 {
        log::debug!("WM_KEYUP received for VK_MENU (Alt)");
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn on_syschar(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let char_code = wparam.0 as u16;
    log::debug!("WM_SYSCHAR received: 0x{:04X}", char_code);

    if char_code == 0x0020 {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    LRESULT(0)
}

pub fn on_syscommand(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let cmd = wparam.0 & 0xFFF0;
    log::debug!("WM_SYSCOMMAND received: 0x{:04X}", cmd);
    if cmd == 0xF100 {
        let key_char = (lparam.0 & 0xFFFF) as u16;
        if key_char == 0x0020 {
            log::debug!("SC_KEYMENU from Alt+Space: passing to DefWindowProcW");
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        }

        log::debug!("SC_KEYMENU received (Menu activation blocked)");
        return LRESULT(0);
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn on_char(hwnd: HWND, wparam: WPARAM) -> LRESULT {
    let char_code = wparam.0 as u16;
    log::debug!(
        "WM_CHAR received: 0x{:04X} ({})",
        char_code,
        String::from_utf16_lossy(&[char_code])
    );

    if char_code == 0x0D || char_code == 0x09 || char_code == 0x1B || char_code == 0x08 {
        log::debug!(
            "WM_CHAR: Skipping char 0x{:04X} (already handled by WM_KEYDOWN)",
            char_code
        );
        return LRESULT(0);
    }

    {
        let data_arc = get_terminal_data();
        let mut window_data = data_arc.lock().unwrap();
        window_data.service.reset_viewport();
        let s = String::from_utf16_lossy(&[char_code]);
        let _ = window_data.service.send_input(s.as_bytes());
    }
    update_window_scroll_info(hwnd);
    unsafe {
        let _ = InvalidateRect(hwnd, None, BOOL(0));
    }

    LRESULT(0)
}

pub fn on_size(hwnd: HWND, lparam: LPARAM) -> LRESULT {
    let width = (lparam.0 & 0xFFFF) as i32;
    let height = ((lparam.0 >> 16) & 0xFFFF) as i32;
    log::info!("WM_SIZE: width={}, height={}", width, height);

    if width <= 0 || height <= 0 {
        return LRESULT(0);
    }

    {
        let data_arc = get_terminal_data();
        let mut window_data = data_arc.lock().unwrap();

        if !window_data.is_conpty_started {
            // 初期起動時の正確なメトリクスを確保するため、一時的にロックを解除して
            // 親ウィンドウ（エディタ）から設定をロードする
            drop(window_data);

            use windows::Win32::UI::WindowsAndMessaging::GetParent;
            let hwnd_editor = match unsafe { GetParent(hwnd) } {
                Ok(h) => h,
                Err(e) => {
                    log::error!("Failed to get parent window for {:?}: {:?}", hwnd, e);
                    return LRESULT(0);
                }
            };

            let config_repo = crate::infra::repository::emeditor_config_repository_impl::EmEditorConfigRepositoryImpl::new(
                crate::domain::model::window_id_value::WindowId(hwnd_editor.0 as isize),
            );
            let config =
                crate::domain::repository::configuration_repository::ConfigurationRepository::load(
                    &config_repo,
                );

            // ロックを再取得してメトリクスを更新
            let mut window_data = data_arc.lock().unwrap();
            use windows::Win32::Graphics::Gdi::GetDC;
            use windows::Win32::Graphics::Gdi::ReleaseDC;
            // SAFETY:
            // - hwnd は有効なウィンドウハンドルであることを前提とする。
            // - 取得した hdc は ReleaseDC により確実に解放される。
            let hdc = unsafe { GetDC(hwnd) };
            if hdc.0.is_null() {
                log::error!("on_size: GetDC failed");
                return LRESULT(0);
            }
            window_data.renderer.update_metrics(hdc, &config);
            unsafe {
                let _ = ReleaseDC(hwnd, hdc);
            }

            let (char_width, char_height) =
                if let Some(metrics) = window_data.renderer.get_metrics() {
                    (metrics.base_width, metrics.char_height)
                } else {
                    (8, 16)
                };

            if char_width <= 0 || char_height <= 0 {
                log::error!("on_size: Invalid metrics ({}x{})", char_width, char_height);
                return LRESULT(0);
            }

            let cols = (width / char_width).max(1) as i16;
            let rows = (height / char_height).max(1) as i16;

            log::debug!(
                "on_size: metrics={}x{}, calculated cols={}, rows={}",
                char_width,
                char_height,
                cols,
                rows
            );

            // コンテキストを一度手放してから起動処理を呼ぶ（デッドロック防止）
            drop(window_data);
            if crate::gui::window::ensure_conpty_started(hwnd, hwnd_editor, cols, rows) {
                log::debug!("ConPTY started in on_size with {}x{}", cols, rows);
            } else {
                log::error!("Failed to start ConPTY. Destroying window.");
                WindowGuiDriver::destroy_window(hwnd);
                // 状態をリセットするために再度ロックを取得
                let mut window_data = data_arc.lock().unwrap();
                window_data.window_handle = None;
                return LRESULT(0);
            }
        } else {
            let (char_width, char_height) =
                if let Some(metrics) = window_data.renderer.get_metrics() {
                    (metrics.base_width, metrics.char_height)
                } else {
                    (8, 16)
                };

            let cols = (width / char_width).max(1) as i16;
            let rows = (height / char_height).max(1) as i16;

            log::debug!("Resizing ConptyIoDriver to cols={}, rows={}", cols, rows);
            window_data.service.resize(cols as usize, rows as usize);
        }
    }

    update_window_scroll_info(hwnd);
    unsafe {
        let _ = InvalidateRect(hwnd, None, BOOL(0));
    }
    LRESULT(0)
}

pub fn on_ime_set_context(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    log::debug!(
        "WM_IME_SETCONTEXT: wparam={:?}, lparam={:?}",
        wparam,
        lparam
    );
    let mut lparam = lparam;
    lparam.0 &= !(ISC_SHOWUICOMPOSITIONWINDOW as isize);
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn on_ime_start_composition(hwnd: HWND) -> LRESULT {
    log::info!("WM_IME_STARTCOMPOSITION: hwnd={:?}", hwnd);
    handle_start_composition(hwnd);
    let data_arc = get_terminal_data();
    {
        let mut window_data = data_arc.lock().unwrap();
        window_data.service.reset_viewport();

        let TerminalWindowResolver {
            ref service,
            ref renderer,
            ref caret,
            ..
        } = *window_data;

        // Sync IME position at the very beginning of composition
        sync_system_caret(
            hwnd,
            service.get_buffer().get_ime_anchor_pos(),
            service.get_buffer().get_viewport_offset(),
            renderer,
            caret.as_ref(),
        );
    }
    update_window_scroll_info(hwnd);

    LRESULT(0)
}

pub fn on_ime_composition(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    log::info!("WM_IME_COMPOSITION: hwnd={:?}, lparam={:x}", hwnd, lparam.0);
    let result = {
        let data_arc = get_terminal_data();
        let mut window_data = data_arc.lock().unwrap();
        let TerminalWindowResolver {
            ref mut service,
            ref renderer,
            ref caret,
            ..
        } = *window_data;

        handle_composition(
            hwnd,
            lparam,
            service.get_buffer().get_ime_anchor_pos(),
            service.get_buffer().get_viewport_offset(),
            renderer,
            caret.as_ref(),
        )
    };

    match result {
        ImeResult::Result(ref text) => {
            let data_arc = get_terminal_data();
            let mut window_data = data_arc.lock().unwrap();
            let _ = window_data.service.send_input(text.as_bytes());
            window_data.composition = None;
            unsafe {
                let _ = InvalidateRect(hwnd, None, BOOL(0));
            }
        }
        ImeResult::Composition(ref text) => {
            let data_arc = get_terminal_data();
            let mut window_data = data_arc.lock().unwrap();
            window_data.composition =
                Some(crate::gui::driver::terminal_gui_driver::CompositionInfo {
                    text: text.clone(),
                });
            unsafe {
                let _ = InvalidateRect(hwnd, None, BOOL(0));
            }
        }
        _ => {}
    }

    if let ImeResult::NotHandled = result {
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    } else {
        LRESULT(0)
    }
}

pub fn on_ime_end_composition(hwnd: HWND) -> LRESULT {
    handle_end_composition(hwnd);
    let data_arc = get_terminal_data();
    {
        let mut window_data = data_arc.lock().unwrap();
        window_data.composition = None;
    }
    unsafe {
        let _ = InvalidateRect(hwnd, None, BOOL(0));
    }
    LRESULT(0)
}

pub fn on_destroy() -> LRESULT {
    log::info!("WM_DESTROY: Cleaning up terminal resources");
    crate::gui::window::cleanup_terminal();

    let data_arc = get_terminal_data();
    {
        let mut window_data = data_arc.lock().unwrap();
        KeyboardGuiDriver::uninstall();
        window_data.renderer.clear_resources();
        window_data.window_handle = None;
        window_data.caret = None;
    }

    log::info!("Terminal resources cleared");
    LRESULT(0)
}

pub fn on_get_dlg_code() -> LRESULT {
    log::debug!("WM_GETDLGCODE received");
    LRESULT(DLGC_WANTALLKEYS as isize)
}

pub fn on_app_repaint(hwnd: HWND) -> LRESULT {
    log::debug!("on_app_repaint: triggered");
    update_window_scroll_info(hwnd);
    let data_arc = get_terminal_data();
    {
        let window_data = data_arc.lock().unwrap();
        let TerminalWindowResolver {
            ref service,
            ref renderer,
            ref caret,
            ..
        } = *window_data;

        // ALWAYS sync system caret on repaint to ensure correct IME position,
        // even during composition, as TUI apps may move the cursor.
        sync_system_caret(
            hwnd,
            service.get_buffer().get_ime_anchor_pos(),
            service.get_buffer().get_viewport_offset(),
            renderer,
            caret.as_ref(),
        );
    }
    unsafe {
        // Force the OS to update the window and caret position immediately.
        // This is crucial for TUI apps where the cursor moves frequently via ConPTY.
        let _ = InvalidateRect(hwnd, None, BOOL(0));
    }
    WindowGuiDriver::update_window(hwnd);
    LRESULT(0)
}
