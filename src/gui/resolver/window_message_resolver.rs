use crate::gui::driver::ime_gui_driver::{
    handle_composition, handle_end_composition, handle_start_composition, sync_system_caret,
    CaretHandle, ImeResult,
};
use crate::gui::driver::scroll_gui_driver::{update_window_scroll_info, ScrollAction};
use crate::gui::resolver::terminal_window_resolver::{get_terminal_data, TerminalWindowResolver};
use crate::infra::driver::keyboard_io_driver::KeyboardIoDriver;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, InvalidateRect, PAINTSTRUCT};
use windows::Win32::UI::Input::KeyboardAndMouse::{SetFocus, VK_MENU};
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

        // Destructure to allow simultaneous mutable borrow of renderer and service
        let TerminalWindowResolver {
            ref mut service,
            ref mut renderer,
            ref composition,
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

        // If metrics were updated during render, trigger a resize to match new font dimensions
        if renderer.metrics_changed {
            renderer.metrics_changed = false;
            let char_width = renderer.metrics.map(|m| m.base_width).unwrap_or(8);
            let char_height = renderer.metrics.map(|m| m.char_height).unwrap_or(16);
            let width = client_rect.right - client_rect.left;
            let height = client_rect.bottom - client_rect.top;

            if char_width > 0 && char_height > 0 {
                // Use maximum available space
                let cols = (width / char_width).max(1);
                let rows = (height / char_height).max(1);
                log::info!("Metrics changed detected in on_paint. Re-sizing to {}x{}", cols, rows);
                service.resize(cols as usize, rows as usize);
            }
        }

        let _ = EndPaint(hwnd, &ps);
    }
    LRESULT(0)
}

pub fn on_lbuttondown(hwnd: HWND) -> LRESULT {
    log::info!("WM_LBUTTONDOWN: Setting focus");
    unsafe {
        let _ = SetFocus(hwnd);
    }
    LRESULT(0)
}

pub fn on_set_focus(hwnd: HWND) -> LRESULT {
    log::info!("WM_SETFOCUS: Focus received, installing keyboard hook and system caret");

    let data_arc = get_terminal_data();
    {
        let mut window_data = data_arc.lock().unwrap();
        // Create RAII system caret handle
        window_data.caret = Some(CaretHandle::new(hwnd));
    }

    KeyboardIoDriver::install_global(hwnd);
    LRESULT(0)
}

pub fn on_kill_focus() -> LRESULT {
    log::info!("WM_KILLFOCUS: Focus lost, uninstalling keyboard hook and system caret");
    let data_arc = get_terminal_data();
    {
        let mut window_data = data_arc.lock().unwrap();
        // RAII handle will automatically call DestroyCaret when dropped
        window_data.caret = None;
    }
    KeyboardIoDriver::uninstall_global();
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

pub fn on_syscommand(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    log::debug!("WM_SYSCOMMAND received: 0x{:04X}", wparam.0);
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn on_syschar(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let vk_code = wparam.0 as u16;
    log::debug!("WM_SYSCHAR received: 0x{:04X}", vk_code);
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn on_char(_hwnd: HWND, wparam: WPARAM) -> LRESULT {
    let ch = wparam.0 as u8;
    log::debug!("WM_CHAR received: 0x{:02X}", ch);
    let data_arc = get_terminal_data();
    let window_data = data_arc.lock().unwrap();
    let _ = window_data.service.send_input(&[ch]);
    LRESULT(0)
}

pub fn on_size(hwnd: HWND, _lparam: LPARAM) -> LRESULT {
    let mut client_rect = windows::Win32::Foundation::RECT::default();
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut client_rect);
    }
    let width = (client_rect.right - client_rect.left) as i32;
    let height = (client_rect.bottom - client_rect.top) as i32;
    log::info!("WM_SIZE (GetClientRect): width={}, height={}", width, height);

    let data_arc = get_terminal_data();
    {
        let mut window_data = data_arc.lock().unwrap();
        let (char_width, char_height) = if let Some(metrics) = window_data.renderer.get_metrics() {
            (metrics.base_width, metrics.char_height)
        } else {
            // 初回などメトリクスが未定の場合は 8x16 を仮定するが、
            // 描画が始まれば TerminalGuiDriver 側で metrics が更新され、
            // 次の描画サイクルで再調整される。
            (8, 16)
        };

        if char_width > 0 && char_height > 0 {
            // Calculate max columns that can fit in the physical width
            let cols = (width / char_width).max(1);
            let rows = (height / char_height).max(1);
            log::info!("Updating logical size to {}x{} based on pixel size {}x{}", cols, rows, width, height);
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
            service.get_buffer().get_cursor_pos(),
            service.get_buffer().get_viewport_offset(),
            renderer,
            caret.as_ref(),
        );
    }
    update_window_scroll_info(hwnd);
    LRESULT(0)
}

pub fn on_ime_composition(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
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
            service.get_buffer().get_cursor_pos(),
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
                Some(crate::gui::driver::terminal_gui_driver::CompositionInfo { text: text.clone() });
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

        // Sync system caret before triggering repaint to ensure correct IME position.
        // We sync even when composing because apps like Ink (Gemini CLI) move the cursor
        // during composition or prompt redraw.
        sync_system_caret(
            hwnd,
            service.get_buffer().get_cursor_pos(),
            service.get_buffer().get_viewport_offset(),
            renderer,
            caret.as_ref(),
        );
    }
    unsafe {
        let _ = InvalidateRect(hwnd, None, BOOL(0));
    }
    LRESULT(0)
}
