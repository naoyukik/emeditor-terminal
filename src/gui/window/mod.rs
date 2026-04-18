use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{
    GetLastError, ERROR_CLASS_ALREADY_EXISTS, HWND, LPARAM, LRESULT, WPARAM,
};
use windows::Win32::Graphics::Gdi::{COLOR_WINDOW, HBRUSH};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, LoadCursorW, RegisterClassW, SendMessageW,
    CS_HREDRAW, CS_VREDRAW, IDC_ARROW, WM_CHAR, WM_DESTROY, WM_ERASEBKGND, WM_GETDLGCODE,
    WM_IME_COMPOSITION, WM_IME_ENDCOMPOSITION, WM_IME_SETCONTEXT, WM_IME_STARTCOMPOSITION,
    WM_KEYDOWN, WM_KEYUP, WM_KILLFOCUS, WM_LBUTTONDOWN, WM_MOUSEWHEEL, WM_PAINT, WM_SETFOCUS,
    WM_SIZE, WM_SYSCHAR, WM_SYSCOMMAND, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_VSCROLL, WNDCLASSW,
    WS_CHILD, WS_VISIBLE, WS_CLIPCHILDREN, WS_CLIPSIBLINGS,
};

use crate::domain::model::window_id_value::WindowId;
use crate::gui::common::SendHWND;
use crate::gui::driver::window_gui_driver::WindowGuiDriver;
use crate::gui::resolver::terminal_window_resolver::get_terminal_data;
use crate::infra::driver::emeditor_io_driver::{
    CUSTOM_BAR_BOTTOM, CUSTOM_BAR_INFO, EE_CUSTOM_BAR_OPEN,
};
use std::ffi::c_void;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::gui::resolver::window_message_resolver as handlers;

// Custom message for repaint from background thread
const WM_APP: u32 = 0x8000;
const WM_APP_REPAINT: u32 = WM_APP + 1;

static CLASS_REGISTERED: AtomicBool = AtomicBool::new(false);
const CLASS_NAME: PCWSTR = w!("EmEditorTerminalClass");

/// IME が変換中であるかを確認する
pub fn is_ime_composing(hwnd: HWND) -> bool {
    crate::gui::driver::ime_gui_driver::is_composing(WindowId(hwnd.0 as isize))
}

/// カスタムバーを開く (エントリポイント)
pub fn open_custom_bar(hwnd_editor: HWND) -> bool {
    unsafe {
        let h_instance = crate::get_instance_handle();

        let data_arc = get_terminal_data();
        let existing_hwnd = {
            let window_data = data_arc.lock().unwrap();
            window_data.window_handle.map(|h| h.0)
        };

        if let Some(hwnd) = existing_hwnd {
            if WindowGuiDriver::focus_existing_window(WindowId(hwnd.0 as isize)) {
                return false; 
            } else {
                WindowGuiDriver::destroy_window(WindowId(hwnd.0 as isize));
                let mut window_data = data_arc.lock().unwrap();
                window_data.window_handle = None;
            }
        }

        if !CLASS_REGISTERED.load(Ordering::Relaxed) {
            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: h_instance,
                lpszClassName: CLASS_NAME,
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as isize as *mut c_void),
                ..Default::default()
            };
            let atom = RegisterClassW(&wc);
            if atom == 0 {
                let err = GetLastError();
                if err != ERROR_CLASS_ALREADY_EXISTS {
                    log::error!("Failed to register window class: {:?}", err);
                    return false;
                }
            }
            CLASS_REGISTERED.store(true, Ordering::Relaxed);
        }

        let hwnd_client_result = CreateWindowExW(
            Default::default(),
            CLASS_NAME,
            w!("Terminal"),
            WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
            0,
            0,
            0,
            0,
            Some(hwnd_editor),
            None,
            Some(h_instance),
            None,
        );

        match hwnd_client_result {
            Ok(hwnd_client) => {
                let mut window_data = data_arc.lock().unwrap();
                window_data.window_handle = Some(SendHWND(hwnd_client));
                drop(window_data);

                let mut info = CUSTOM_BAR_INFO {
                    cbSize: size_of::<CUSTOM_BAR_INFO>(),
                    hwndCustomBar: HWND::default(),
                    hwndClient: hwnd_client,
                    pszTitle: w!("Terminal"),
                    iPos: CUSTOM_BAR_BOTTOM,
                };

                SendMessageW(
                    hwnd_editor,
                    EE_CUSTOM_BAR_OPEN,
                    Some(WPARAM(0)),
                    Some(LPARAM(&mut info as *mut _ as isize)),
                );

                WindowGuiDriver::focus_existing_window(WindowId(hwnd_client.0 as isize));
                true
            }
            Err(e) => {
                log::error!("Failed to create custom bar window: {}", e);
                false
            }
        }
    }
}

/// システムショートカットであるかを判定する
pub fn is_system_shortcut(vk_code: u16, alt_pressed: bool) -> bool {
    WindowGuiDriver::is_system_shortcut(vk_code, alt_pressed)
}

pub fn cleanup_terminal() {
    log::info!("cleanup_terminal: Starting cleanup");
    let data_arc = get_terminal_data();
    let mut window_data = data_arc.lock().unwrap();
    
    // Workflow に新しいダミーサービスを注入してリセット
    use crate::infra::repository::conpty_repository_impl::DummyOutputRepository;
    use crate::infra::repository::emeditor_config_repository_impl::EmEditorConfigRepositoryImpl;
    
    let output_repo = Box::new(DummyOutputRepository);
    let config_repo = Box::new(EmEditorConfigRepositoryImpl::new(WindowId(0)));
    let is_dark = crate::infra::driver::emeditor_io_driver::is_system_dark_mode();
    let service = crate::application::TerminalWorkflow::new(80, 25, output_repo, config_repo, is_dark);
    
    window_data.reset_service(service);
}

pub extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let window_id = WindowId(hwnd.0 as isize);
    match msg {
        WM_VSCROLL => LRESULT(handlers::on_vscroll(window_id, wparam.0, lparam.0)),
        WM_MOUSEWHEEL => LRESULT(handlers::on_mousewheel(window_id, wparam.0)),
        WM_PAINT => LRESULT(handlers::on_paint(window_id)),
        WM_LBUTTONDOWN => LRESULT(handlers::on_lbuttondown(window_id)),
        WM_SETFOCUS => LRESULT(handlers::on_set_focus(window_id)),
        WM_KILLFOCUS => LRESULT(handlers::on_kill_focus()),
        WM_KEYDOWN => LRESULT(handlers::on_keydown(window_id, msg, wparam.0, lparam.0)),
        WM_SYSKEYDOWN => LRESULT(handlers::on_syskeydown(window_id, msg, wparam.0, lparam.0)),
        WM_SYSKEYUP => LRESULT(handlers::on_syskeyup(window_id, msg, wparam.0, lparam.0)),
        WM_KEYUP => LRESULT(handlers::on_keyup(window_id, msg, wparam.0, lparam.0)),
        WM_SYSCHAR => LRESULT(handlers::on_syschar(window_id, msg, wparam.0, lparam.0)),
        WM_SYSCOMMAND => LRESULT(handlers::on_syscommand(window_id, msg, wparam.0, lparam.0)),
        WM_GETDLGCODE => LRESULT(handlers::on_get_dlg_code()),
        WM_CHAR => LRESULT(handlers::on_char(window_id, wparam.0)),
        msg if msg == WM_APP_REPAINT => LRESULT(handlers::on_app_repaint(window_id)),
        WM_SIZE => {
            let width = (lparam.0 & 0xFFFF) as i32;
            let height = ((lparam.0 >> 16) & 0xFFFF) as i32;
            LRESULT(handlers::on_size(window_id, width, height))
        }
        WM_IME_SETCONTEXT => LRESULT(handlers::on_ime_set_context(window_id, msg, wparam.0, lparam.0)),
        WM_IME_STARTCOMPOSITION => LRESULT(handlers::on_ime_start_composition(window_id)),
        WM_IME_COMPOSITION => LRESULT(handlers::on_ime_composition(window_id, msg, wparam.0, lparam.0)),
        WM_IME_ENDCOMPOSITION => LRESULT(handlers::on_ime_end_composition(window_id)),
        WM_ERASEBKGND => LRESULT(1),
        WM_DESTROY => LRESULT(handlers::on_destroy()),
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
