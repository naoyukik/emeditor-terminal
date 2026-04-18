use super::terminal_window_resolver::{get_terminal_data, TerminalWindowResolver};
use crate::domain::model::window_id_value::WindowId;
use crate::gui::driver::ime_gui_driver::{
    handle_composition, handle_end_composition, handle_start_composition, sync_system_caret,
    ImeResult,
};
use crate::gui::driver::keyboard_gui_driver::KeyboardGuiDriver;
use crate::gui::driver::scroll_gui_driver::{update_window_scroll_info, ScrollAction};
use crate::gui::driver::terminal_gui_driver::CompositionInfo;
use crate::gui::driver::window_gui_driver::WindowGuiDriver;

/// Win32 固有の定数定義
const ISC_SHOWUICOMPOSITIONWINDOW: u32 = 0x80000000;
const VK_MENU: usize = 0x12;
const DLGC_WANTALLKEYS: isize = 0x0004;

pub fn on_vscroll(window_id: WindowId, wparam: usize, lparam: isize) -> isize {
    let data_arc = get_terminal_data();
    let action = {
        let mut window_data = data_arc.lock().unwrap();
        let history_count = window_data.service.get_history_count() as i32;
        let height = window_data.service.get_buffer().get_height() as i32;
        window_data.scroll_manager.max = history_count + height - 1;
        window_data.scroll_manager.page = height as u32;
        window_data.scroll_manager.handle_vscroll(wparam, lparam)
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
    update_window_scroll_info(window_id);
    WindowGuiDriver::invalidate_rect(window_id, false);
    0
}

pub fn on_mousewheel(window_id: WindowId, wparam: usize) -> isize {
    let data_arc = get_terminal_data();
    let action = {
        let window_data = data_arc.lock().unwrap();
        window_data.scroll_manager.handle_mousewheel(wparam)
    };
    if let ScrollAction::ScrollBy(lines) = action {
        let mut window_data = data_arc.lock().unwrap();
        window_data.service.scroll_lines(lines);
    }
    update_window_scroll_info(window_id);
    WindowGuiDriver::invalidate_rect(window_id, false);
    0
}

pub fn on_paint(window_id: WindowId) -> isize {
    WindowGuiDriver::perform_paint(window_id, |ctx| {
        let data_arc = get_terminal_data();
        let mut window_data = data_arc.lock().unwrap();
        let TerminalWindowResolver {
            ref service,
            ref mut renderer,
            ref composition,
            ref caret,
            ..
        } = *window_data;
        renderer.render(
            ctx.hdc,
            &ctx.rect,
            service.get_buffer(),
            composition.as_ref(),
            &service.color_theme,
            &service.config,
        );
        sync_system_caret(
            window_id,
            service.get_buffer().get_ime_anchor_pos(),
            service.get_buffer().get_viewport_offset(),
            renderer,
            caret.as_ref(),
        );
    });
    0
}

pub fn on_lbuttondown(window_id: WindowId) -> isize {
    log::info!("WM_LBUTTONDOWN: Setting focus");
    WindowGuiDriver::focus_existing_window(window_id);
    0
}

pub fn on_set_focus(window_id: WindowId) -> isize {
    log::info!("WM_SETFOCUS: Focus received");
    let data_arc = get_terminal_data();
    {
        let mut window_data = data_arc.lock().unwrap();
        let (cw, ch) = if let Some(m) = window_data.renderer.get_metrics() {
            (m.base_width, m.char_height)
        } else {
            (8, 16)
        };
        window_data.caret = Some(crate::gui::driver::ime_gui_driver::CaretHandle::new(
            window_id, cw, ch,
        ));
        KeyboardGuiDriver::install(window_id);
    }
    0
}

pub fn on_kill_focus() -> isize {
    log::info!("WM_KILLFOCUS: Focus lost");
    let data_arc = get_terminal_data();
    {
        let mut window_data = data_arc.lock().unwrap();
        window_data.caret = None;
        KeyboardGuiDriver::uninstall();
    }
    0
}

pub fn on_keydown(window_id: WindowId, msg: u32, wparam: usize, lparam: isize) -> isize {
    log::debug!("WM_KEYDOWN received: 0x{:04X}", wparam);
    WindowGuiDriver::default_window_proc(window_id, msg, wparam, lparam)
}

pub fn on_syskeydown(window_id: WindowId, msg: u32, wparam: usize, lparam: isize) -> isize {
    log::debug!("WM_SYSKEYDOWN received: 0x{:04X}", wparam);
    if WindowGuiDriver::is_system_shortcut(wparam as u16, true) {
        return WindowGuiDriver::default_window_proc(window_id, msg, wparam, lparam);
    }
    if wparam == VK_MENU {
        return 0;
    }
    WindowGuiDriver::default_window_proc(window_id, msg, wparam, lparam)
}

pub fn on_syskeyup(window_id: WindowId, msg: u32, wparam: usize, lparam: isize) -> isize {
    log::debug!("WM_SYSKEYUP received: 0x{:04X}", wparam);
    WindowGuiDriver::default_window_proc(window_id, msg, wparam, lparam)
}

pub fn on_keyup(window_id: WindowId, msg: u32, wparam: usize, lparam: isize) -> isize {
    log::debug!("WM_KEYUP received for 0x{:04X}", wparam);
    WindowGuiDriver::default_window_proc(window_id, msg, wparam, lparam)
}

pub fn on_syschar(window_id: WindowId, msg: u32, wparam: usize, lparam: isize) -> isize {
    log::debug!("WM_SYSCHAR received: 0x{:04X}", wparam);
    WindowGuiDriver::default_window_proc(window_id, msg, wparam, lparam)
}

pub fn on_syscommand(window_id: WindowId, msg: u32, wparam: usize, lparam: isize) -> isize {
    log::debug!("WM_SYSCOMMAND received");
    WindowGuiDriver::default_window_proc(window_id, msg, wparam, lparam)
}

pub fn on_char(window_id: WindowId, wparam: usize) -> isize {
    let char_code = wparam as u16;
    if char_code == 0x0D || char_code == 0x09 || char_code == 0x1B || char_code == 0x08 {
        return 0;
    }
    {
        let data_arc = get_terminal_data();
        let mut window_data = data_arc.lock().unwrap();
        window_data.service.reset_viewport();
        let s = String::from_utf16_lossy(&[char_code]);
        let _ = window_data.service.send_input(s.as_bytes());
    }
    update_window_scroll_info(window_id);
    WindowGuiDriver::invalidate_rect(window_id, false);
    0
}

pub fn on_size(window_id: WindowId, lparam: isize) -> isize {
    WindowGuiDriver::handle_resize(window_id, lparam);
    0
}

pub fn on_get_dlg_code() -> isize {
    DLGC_WANTALLKEYS
}

pub fn on_ime_set_context(window_id: WindowId, msg: u32, wparam: usize, lparam: isize) -> isize {
    let mut lp = lparam;
    lp &= !(ISC_SHOWUICOMPOSITIONWINDOW as isize);
    WindowGuiDriver::default_window_proc(window_id, msg, wparam, lp)
}

pub fn on_ime_start_composition(window_id: WindowId) -> isize {
    handle_start_composition(window_id);
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
        sync_system_caret(
            window_id,
            service.get_buffer().get_ime_anchor_pos(),
            service.get_buffer().get_viewport_offset(),
            renderer,
            caret.as_ref(),
        );
    }
    update_window_scroll_info(window_id);
    0
}

pub fn on_ime_composition(window_id: WindowId, msg: u32, wparam: usize, lparam: isize) -> isize {
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
            window_id,
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
            WindowGuiDriver::invalidate_rect(window_id, false);
        }
        ImeResult::Composition(ref text) => {
            let data_arc = get_terminal_data();
            let mut window_data = data_arc.lock().unwrap();
            window_data.composition = Some(CompositionInfo { text: text.clone() });
            WindowGuiDriver::invalidate_rect(window_id, false);
        }
        _ => {}
    }
    if let ImeResult::NotHandled = result {
        WindowGuiDriver::default_window_proc(window_id, msg, wparam, lparam)
    } else {
        0
    }
}

pub fn on_ime_end_composition(window_id: WindowId) -> isize {
    handle_end_composition(window_id);
    let data_arc = get_terminal_data();
    {
        let mut window_data = data_arc.lock().unwrap();
        window_data.composition = None;
    }
    WindowGuiDriver::invalidate_rect(window_id, false);
    0
}

pub fn on_destroy() -> isize {
    log::info!("WM_DESTROY: Cleaning up terminal resources");

    // キーボードフックを解除
    KeyboardGuiDriver::uninstall();

    // ターミナルリソース（ConPTY等）を解放し、ダミーサービスを注入
    crate::gui::window::cleanup_terminal();

    let data_arc = get_terminal_data();
    {
        let mut window_data = data_arc.lock().unwrap();
        window_data.renderer.clear_resources();
        window_data.window_handle = None;
        window_data.caret = None;
    }
    0
}

pub fn on_app_repaint(window_id: WindowId) -> isize {
    update_window_scroll_info(window_id);
    let data_arc = get_terminal_data();
    let is_composing = {
        let window_data = data_arc.lock().unwrap();
        let TerminalWindowResolver {
            ref service,
            ref renderer,
            ref caret,
            ref composition,
            ..
        } = *window_data;
        sync_system_caret(
            window_id,
            service.get_buffer().get_ime_anchor_pos(),
            service.get_buffer().get_viewport_offset(),
            renderer,
            caret.as_ref(),
        );
        composition.is_some()
    };
    WindowGuiDriver::invalidate_rect(window_id, false);
    if is_composing {
        WindowGuiDriver::update_window(window_id);
    }
    0
}
