use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{
    GetLastError, ERROR_CLASS_ALREADY_EXISTS, HWND, LPARAM, LRESULT, WPARAM,
};
use windows::Win32::Graphics::Gdi::{COLOR_WINDOW, HBRUSH};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, LoadCursorW, PostMessageW, RegisterClassW, SendMessageW,
    CS_HREDRAW, CS_VREDRAW, IDC_ARROW, WM_CHAR, WM_DESTROY, WM_GETDLGCODE, WM_IME_COMPOSITION,
    WM_IME_ENDCOMPOSITION, WM_IME_SETCONTEXT, WM_IME_STARTCOMPOSITION, WM_KEYDOWN, WM_KEYUP,
    WM_KILLFOCUS, WM_LBUTTONDOWN, WM_MOUSEWHEEL, WM_PAINT, WM_SETFOCUS, WM_SIZE, WM_SYSCHAR,
    WM_SYSCOMMAND, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_VSCROLL, WNDCLASSW, WS_CHILD, WS_CLIPCHILDREN,
    WS_CLIPSIBLINGS, WS_VISIBLE,
};

use crate::gui::resolver::terminal_window_resolver::{get_terminal_data, SendHWND};
use crate::infra::driver::conpty_io_driver::{ConptyIoDriver, SendHandle};
use crate::infra::driver::emeditor_io_driver::{
    CUSTOM_BAR_BOTTOM, CUSTOM_BAR_INFO, EE_CUSTOM_BAR_OPEN,
};
use std::ffi::c_void;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use windows::Win32::Storage::FileSystem::ReadFile;
use windows::Win32::UI::Input::KeyboardAndMouse::{SetFocus, VK_ESCAPE, VK_F4, VK_SPACE, VK_TAB};

use crate::gui::resolver::window_message_resolver as handlers;

// Custom message for repaint from background thread
const WM_APP: u32 = 0x8000;
const WM_APP_REPAINT: u32 = WM_APP + 1;

static CLASS_REGISTERED: AtomicBool = AtomicBool::new(false);
const CLASS_NAME: PCWSTR = w!("EmEditorTerminalClass");

// Helper to check if IME is composing (Re-export for handlers if needed, or just keep public)
// handlers.rs doesn't seem to use this directly, it calls crate::gui::ime::is_composing
pub fn is_ime_composing(hwnd: HWND) -> bool {
    crate::gui::driver::ime_gui_driver::is_composing(hwnd)
}

fn start_conpty_and_reader_thread(hwnd: HWND, cols: i16, rows: i16) -> bool {
    match ConptyIoDriver::new("pwsh.exe", cols, rows) {
        Ok(conpty) => {
            log::info!(
                "ConptyIoDriver started successfully with size {}x{}",
                cols,
                rows
            );

            let data_arc = get_terminal_data();
            let output_handle: SendHandle = conpty.get_output_handle();
            // ConptyIoDriver retains ownership of the handle and closes it on drop.
            // SendHandle is Copy, so we can pass a copy to the thread.

            {
                let mut window_data = data_arc.lock().unwrap();
                let output_repo = Box::new(
                    crate::infra::repository::conpty_repository_impl::ConptyRepositoryImpl::new(
                        conpty,
                    ),
                );
                let config_repo = Box::new(crate::infra::repository::emeditor_config_repository_impl::EmEditorConfigRepositoryImpl::new());

                // 新しいリポジトリでサービスを再構築する（DI）
                window_data.service = crate::application::TerminalWorkflow::new(
                    cols as usize,
                    rows as usize,
                    output_repo,
                    config_repo,
                );
            }

            let send_hwnd = SendHWND(hwnd);

            thread::spawn(move || {
                let output_handle = output_handle; // Force capture of SendHandle to avoid disjoint capture of !Send HANDLE
                let send_hwnd = send_hwnd; // Force capture of SendHWND
                let mut buffer = [0u8; 1024];
                let mut bytes_read = 0;
                loop {
                    let read_result = unsafe {
                        ReadFile(
                            output_handle.0,
                            Some(&mut buffer),
                            Some(&mut bytes_read),
                            None,
                        )
                    };

                    if let Err(e) = read_result {
                        log::error!("ReadFile failed: {}", e);
                        break;
                    }
                    if bytes_read == 0 {
                        log::info!("ReadFile returned 0 bytes (EOF)");
                        break;
                    }

                    let raw_bytes = &buffer[..bytes_read as usize];
                    let hex_output: String =
                        raw_bytes.iter().map(|b| format!("{:02X} ", b)).collect();
                    log::debug!(
                        "ConptyIoDriver Raw Output ({} bytes): {}",
                        bytes_read,
                        hex_output
                    );
                    let output = String::from_utf8_lossy(raw_bytes);
                    log::debug!("ConptyIoDriver Output: {}", output);

                    {
                        let mut window_data = data_arc.lock().unwrap();
                        window_data.service.process_output(&output);
                    }

                    // Trigger repaint via PostMessage (thread-safe)
                    unsafe {
                        let _ = PostMessageW(send_hwnd.0, WM_APP_REPAINT, WPARAM(0), LPARAM(0));
                    }
                }
                log::info!("ConptyIoDriver output thread finished");
            });
            true
        }
        Err(e) => {
            log::error!("Failed to start ConptyIoDriver: {}", e);
            false
        }
    }
}

pub fn open_custom_bar(hwnd_editor: HWND) -> bool {
    unsafe {
        let h_instance = crate::get_instance_handle();

        // Check if already open
        let data_arc = get_terminal_data();
        {
            let window_data = data_arc.lock().unwrap();
            if let Some(h) = window_data.window_handle {
                let _ = SetFocus(h.0);
                return false;
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
            hwnd_editor,
            None,
            h_instance,
            None,
        );

        match hwnd_client_result {
            Ok(hwnd_client) => {
                if hwnd_client.0.is_null() {
                    log::error!("Failed to create custom bar window: Handle is NULL");
                    return false;
                }

                // Store window handle immediately
                {
                    let mut window_data = data_arc.lock().unwrap();
                    // Double check if another window was created concurrently (unlikely in UI thread but safe)
                    if let Some(h) = window_data.window_handle {
                        // Another window exists, destroy this one and focus the existing one
                        let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd_client);
                        let _ = SetFocus(h.0);
                        return false;
                    }
                    window_data.window_handle = Some(SendHWND(hwnd_client));
                }

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
                    WPARAM(0),
                    LPARAM(&mut info as *mut _ as isize),
                );

                // Calculate initial size
                let mut client_rect = windows::Win32::Foundation::RECT::default();
                let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(
                    hwnd_client,
                    &mut client_rect,
                );
                let width_px = client_rect.right - client_rect.left;
                let height_px = client_rect.bottom - client_rect.top;

                let (initial_cols, initial_rows) = if width_px > 0 && height_px > 0 {
                    let window_data = data_arc.lock().unwrap();
                    let (char_width, char_height) =
                        if let Some(metrics) = window_data.renderer.get_metrics() {
                            (metrics.base_width, metrics.char_height)
                        } else {
                            (8, 16) // Fallback
                        };
                    (
                        (width_px / char_width).max(1) as i16,
                        (height_px / char_height).max(1) as i16,
                    )
                } else {
                    (80, 25)
                };

                start_conpty_and_reader_thread(hwnd_client, initial_cols, initial_rows)
            }
            Err(e) => {
                log::error!("Failed to create custom bar window: {}", e);
                false
            }
        }
    }
}

/// システムショートカット（Alt+Tab 等）であるかを判定する
pub fn is_system_shortcut(vk_code: u16, alt_pressed: bool) -> bool {
    alt_pressed
        && (vk_code == VK_F4.0
            || vk_code == VK_TAB.0
            || vk_code == VK_SPACE.0
            || vk_code == VK_ESCAPE.0)
}

#[allow(dead_code)]
pub fn send_input(text: &str) {
    let data_arc = get_terminal_data();
    let window_data = data_arc.lock().unwrap();
    let _ = window_data.service.send_input(text.as_bytes());
    // 改行を送る
    let _ = window_data.service.send_input(b"\r");
}

pub fn cleanup_terminal() {
    log::info!("cleanup_terminal: Starting cleanup");
    let data_arc = get_terminal_data();
    let mut window_data = data_arc.lock().unwrap();
    // TerminalServiceが差し替えられる（古いサービスがドロップされる）際に、
    // 内部のリポジトリ経由でConPTYもドロップされる。
    window_data.reset_service();
    log::info!("TerminalWorkflow reset in cleanup_terminal");
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_VSCROLL => handlers::on_vscroll(hwnd, wparam, lparam),
        WM_MOUSEWHEEL => handlers::on_mousewheel(hwnd, wparam),
        WM_PAINT => handlers::on_paint(hwnd),
        WM_LBUTTONDOWN => handlers::on_lbuttondown(hwnd),
        WM_SETFOCUS => handlers::on_set_focus(hwnd),
        WM_KILLFOCUS => handlers::on_kill_focus(),
        WM_KEYDOWN => handlers::on_keydown(hwnd, msg, wparam, lparam),
        WM_SYSKEYDOWN => handlers::on_syskeydown(hwnd, msg, wparam, lparam),
        WM_SYSKEYUP => handlers::on_syskeyup(hwnd, msg, wparam, lparam),
        WM_KEYUP => handlers::on_keyup(hwnd, msg, wparam, lparam),
        WM_SYSCHAR => handlers::on_syschar(hwnd, msg, wparam, lparam),
        WM_SYSCOMMAND => handlers::on_syscommand(hwnd, msg, wparam, lparam),
        WM_GETDLGCODE => handlers::on_get_dlg_code(),
        WM_CHAR => handlers::on_char(hwnd, wparam),
        msg if msg == WM_APP_REPAINT => handlers::on_app_repaint(hwnd),
        WM_SIZE => handlers::on_size(hwnd, lparam),
        WM_IME_SETCONTEXT => handlers::on_ime_set_context(hwnd, msg, wparam, lparam),
        WM_IME_STARTCOMPOSITION => handlers::on_ime_start_composition(hwnd),
        WM_IME_COMPOSITION => handlers::on_ime_composition(hwnd, msg, wparam, lparam),
        WM_IME_ENDCOMPOSITION => handlers::on_ime_end_composition(hwnd),
        WM_DESTROY => handlers::on_destroy(),
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
