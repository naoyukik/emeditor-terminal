use crate::gui::driver::ime_gui_driver::{
    handle_composition, handle_end_composition, handle_start_composition, sync_system_caret,
    CaretHandle, ImeResult,
};
use crate::gui::driver::keyboard_gui_driver::KeyboardGuiDriver;
use crate::gui::driver::scroll_gui_driver::{update_window_scroll_info, ScrollAction};
use crate::gui::resolver::terminal_window_resolver::{get_terminal_data, TerminalWindowResolver};
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

        // If IME is active, we use the anchor position to prevent jumping.
        let ime_anchor = window_data.ime_anchor;

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
            ime_anchor,
            &service.color_theme,
            &service.config,
        );

        let sync_pos = ime_anchor.unwrap_or_else(|| service.get_buffer().get_last_valid_cursor_pos());

        // During IME composition (ime_anchor is Some), always treat cursor as visible.
        // anchor_pos was captured from get_last_valid_cursor_pos(), so visibility is guaranteed.
        // Without this, TUI apps that hide/show cursor during redraws (e.g. Gemini CLI) would
        // cause sync_system_caret to bail out and leave the IME window at the wrong position.
        let is_visible = ime_anchor.is_some() || service.get_buffer().is_cursor_visible();

        // Always sync system caret position with virtual cursor during paint
        sync_system_caret(
            hwnd,
            sync_pos,
            is_visible,
            service.get_buffer(),
            renderer,
            caret.as_ref(),
            service.get_font_face(),
        );

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
        let (char_width, char_height) = if let Some(metrics) = window_data.renderer.get_metrics() {
            (metrics.base_width, metrics.char_height)
        } else {
            (8, 16)
        };
        // Create RAII system caret handle with proper dimensions
        window_data.caret = Some(CaretHandle::new(hwnd, char_width, char_height));

        // Ensure proper IME context association
        crate::gui::driver::ime_gui_driver::associate_ime_context(hwnd);

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

        let (char_width, char_height) = if let Some(metrics) = window_data.renderer.get_metrics() {
            (metrics.base_width, metrics.char_height)
        } else {
            (8, 16)
        };

        let cols = (width / char_width).max(1) as i16;
        let rows = (height / char_height).max(1) as i16;

        log::info!("Resizing ConptyIoDriver to cols={}, rows={}", cols, rows);
        window_data.service.resize(cols as usize, rows as usize);
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
    // Suppress the default OS composition window because we draw it ourselves in TerminalGuiDriver.
    // This prevents double-drawing and "edge-jumping" of the composition string.
    let mut lparam = lparam;
    lparam.0 &= !(ISC_SHOWUICOMPOSITIONWINDOW as isize);
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn on_ime_start_composition(hwnd: HWND) -> LRESULT {
    log::info!("WM_IME_STARTCOMPOSITION");
    handle_start_composition(hwnd);
    let data_arc = get_terminal_data();
    {
        let mut window_data = data_arc.lock().unwrap();
        window_data.service.reset_viewport();

        // Record the current valid cursor position as the "anchor" for this composition session.
        let anchor_pos = window_data.service.get_buffer().get_last_valid_cursor_pos();
        window_data.ime_anchor = Some(anchor_pos);
        log::info!("IME composition started, anchor set to {:?} (buffer_pos={:?})",
            anchor_pos, window_data.service.get_buffer().get_cursor_pos());

        let font_face = window_data.service.get_font_face().to_string();

        // anchor_pos comes from get_last_valid_cursor_pos(), which is only set when the cursor
        // is visible. So we can always treat it as visible here, even if the TUI app has
        // temporarily hidden the cursor between paint frames.
        // Sync IME position at the very beginning of composition to prime the position.
        // We use CFS_FORCE_POSITION in sync_system_caret to ensure this is applied.
        let buffer = window_data.service.get_buffer();
        sync_system_caret(
            hwnd,
            anchor_pos,
            true,
            buffer,
            &window_data.renderer,
            window_data.caret.as_ref(),
            &font_face,
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
            ref ime_anchor,
            ..
        } = *window_data;

        // Use the anchor position if available, otherwise fallback to current buffer position
        let sync_pos = ime_anchor.unwrap_or_else(|| service.get_buffer().get_cursor_pos());

        handle_composition(
            hwnd,
            lparam,
            sync_pos,
            service.get_buffer(),
            renderer,
            caret.as_ref(),
            service.get_font_face(),
        )
    };

    match result {
        ImeResult::Result(ref text) => {
            log::info!("IME Result: {}", text);
            let data_arc = get_terminal_data();
            let mut window_data = data_arc.lock().unwrap();
            let _ = window_data.service.send_input(text.as_bytes());
            window_data.composition = None;
            unsafe {
                let _ = InvalidateRect(hwnd, None, BOOL(0));
            }
        }
        ImeResult::Composition(ref text) => {
            log::debug!("IME Composition: {}", text);
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
    log::info!("WM_IME_ENDCOMPOSITION");
    handle_end_composition(hwnd);
    let data_arc = get_terminal_data();
    {
        let mut window_data = data_arc.lock().unwrap();
        window_data.composition = None;
        window_data.ime_anchor = None; // Clear the anchor
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
    update_window_scroll_info(hwnd);
    let data_arc = get_terminal_data();
    {
        let window_data = data_arc.lock().unwrap();
        let ime_anchor = window_data.ime_anchor;

        let TerminalWindowResolver {
            ref service,
            ref renderer,
            ref caret,
            ..
        } = *window_data;

        // ALWAYS sync system caret on repaint to ensure correct IME position,
        // even during composition, as TUI apps may move the cursor.
        // If IME is active, we use the anchor position to prevent jumping.
        // We use the last known VALID cursor position to avoid "parking" artifacts.
        let sync_pos = ime_anchor.unwrap_or_else(|| service.get_buffer().get_last_valid_cursor_pos());

        // During IME composition, always treat cursor as visible (same rationale as on_paint).
        let is_visible = ime_anchor.is_some() || service.get_buffer().is_cursor_visible();
        let buffer = service.get_buffer();

        sync_system_caret(
            hwnd,
            sync_pos,
            is_visible,
            buffer,
            renderer,
            caret.as_ref(),
            service.get_font_face(),
        );
    }
    unsafe {
        let _ = InvalidateRect(hwnd, None, BOOL(0));
    }
    LRESULT(0)
}
