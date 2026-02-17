use crate::gui::driver::scroll_gui_driver::{update_window_scroll_info, ScrollAction};
use crate::gui::resolver::terminal_window_resolver::{get_terminal_data, TerminalWindowResolver};
use crate::infra::driver::keyboard_io_driver::KeyboardIoDriver;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, InvalidateRect, PAINTSTRUCT};
use windows::Win32::UI::Input::KeyboardAndMouse::{SetFocus, VK_MENU};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateCaret, DefWindowProcW, DestroyCaret, DLGC_WANTALLKEYS,
};

const ISC_SHOWUICOMPOSITIONWINDOW: u32 = 0x80000000;

pub fn on_vscroll(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let data_arc = get_terminal_data();
    let action = {
        let mut window_data = data_arc.lock().unwrap();

        // Sync state before handling
        let history_count = window_data.service.get_history_count() as i32;
        let height = window_data.service.buffer.height as i32;
        window_data.scroll_manager.max = history_count + height - 1;
        window_data.scroll_manager.page = height as u32;

        window_data.scroll_manager.handle_vscroll(wparam.0, lparam.0)
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

        let TerminalWindowResolver {
            ref service,
            ref mut renderer,
            ref composition,
            ..
        } = *window_data;

        renderer.render(hdc, &client_rect, &service.buffer, composition.as_ref());

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
    log::info!("WM_SETFOCUS: Focus received, installing keyboard hook");

    // Note: TERMINAL_HWND logic was specific to where it's defined.
    // If we need it, we should move it to infra/input.rs or terminal_data.rs
    // But infra/input.rs manages hook instance now.
    // Let's assume infra/input sets its own target hwnd on install.
    // The previous implementation had a separate TERMINAL_HWND in window.rs.
    // For caret creation:

    let data_arc = get_terminal_data();
    let window_data = data_arc.lock().unwrap();
    let char_height = window_data
        .renderer
        .get_metrics()
        .map(|m| m.char_height)
        .unwrap_or(16);
    unsafe {
        let _ = CreateCaret(
            hwnd,
            windows::Win32::Graphics::Gdi::HBITMAP::default(),
            2,
            char_height,
        );
    }

    KeyboardIoDriver::install_global(hwnd);
    LRESULT(0)
}

pub fn on_kill_focus() -> LRESULT {
    log::info!("WM_KILLFOCUS: Focus lost, uninstalling keyboard hook");
    unsafe {
        let _ = DestroyCaret();
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
    {
        let data_arc = get_terminal_data();
        let mut window_data = data_arc.lock().unwrap();
        crate::gui::driver::ime_gui_driver::handle_start_composition(hwnd, &mut window_data.service);
    }
    update_window_scroll_info(hwnd);

    LRESULT(0)
}

pub fn on_ime_composition(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let handled = {
        let data_arc = get_terminal_data();
        let mut window_data = data_arc.lock().unwrap();
        let data_inner = &mut *window_data;

        crate::gui::driver::ime_gui_driver::handle_composition(
            hwnd,
            lparam,
            &mut data_inner.service,
            &data_inner.renderer,
            &mut data_inner.composition,
        )
    };

    if !handled {
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    } else {
        LRESULT(0)
    }
}

pub fn on_ime_end_composition(hwnd: HWND) -> LRESULT {
    {
        let data_arc = get_terminal_data();
        let mut window_data = data_arc.lock().unwrap();
        crate::gui::driver::ime_gui_driver::handle_end_composition(hwnd, &mut window_data.composition);
    }
    LRESULT(0)
}

pub fn on_destroy() -> LRESULT {
    log::info!("WM_DESTROY: Cleaning up terminal resources");
    KeyboardIoDriver::uninstall_global();

    // 先にグローバルデータをリセット（ConPTY解放を含む）
    crate::gui::window::cleanup_terminal();

    let data_arc = get_terminal_data();
    let mut window_data = data_arc.lock().unwrap();

    window_data.window_handle = None;
    window_data.renderer.clear_resources();

    log::info!("Terminal resources cleared");
    LRESULT(0)
}

pub fn on_get_dlg_code() -> LRESULT {
    log::debug!("WM_GETDLGCODE received");
    LRESULT(DLGC_WANTALLKEYS as isize)
}

pub fn on_app_repaint(hwnd: HWND) -> LRESULT {
    update_window_scroll_info(hwnd);
    unsafe {
        let _ = InvalidateRect(hwnd, None, BOOL(0));
    }
    LRESULT(0)
}
