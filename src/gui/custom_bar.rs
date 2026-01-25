use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{BOOL, HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, InvalidateRect, COLOR_WINDOW, HBRUSH, PAINTSTRUCT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, CreateCaret, CreateWindowExW, DefWindowProcW, DestroyCaret, LoadCursorW,
    PostMessageW, RegisterClassW, SendMessageW, SetCaretPos, SetWindowsHookExW,
    UnhookWindowsHookEx, CS_HREDRAW, CS_VREDRAW, DLGC_WANTALLKEYS, HHOOK, IDC_ARROW, WH_KEYBOARD,
    WM_CHAR, WM_DESTROY, WM_GETDLGCODE, WM_IME_COMPOSITION, WM_IME_ENDCOMPOSITION,
    WM_IME_SETCONTEXT, WM_IME_STARTCOMPOSITION, WM_KEYDOWN, WM_KEYUP, WM_KILLFOCUS, WM_LBUTTONDOWN,
    WM_PAINT, WM_SETFOCUS, WM_SIZE, WM_SYSCHAR, WM_SYSCOMMAND, WM_SYSKEYDOWN, WM_SYSKEYUP,
    WNDCLASSW, WS_CHILD, WS_VISIBLE, WS_CLIPCHILDREN, WS_CLIPSIBLINGS,
};
const ISC_SHOWUICOMPOSITIONWINDOW: u32 = 0x80000000;
use crate::domain::terminal::TerminalBuffer;
use crate::application::TerminalService;
use crate::gui::renderer::{CompositionData, TerminalRenderer};
use crate::infra::conpty::ConPTY;
use std::cell::RefCell;
use std::ffi::c_void;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile};
use windows::Win32::UI::Input::Ime::{
    ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext, ImmSetCompositionWindow, CFS_POINT,
    COMPOSITIONFORM, GCS_COMPSTR, GCS_RESULTSTR,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, SetFocus, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1,
    VK_F10, VK_F11, VK_F12, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_HOME,
    VK_INSERT, VK_LEFT, VK_MENU, VK_NEXT, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_SHIFT, VK_SPACE,
    VK_TAB, VK_UP,
};

// Constants from EmEditor SDK
const WM_USER: u32 = 0x0400;
const EE_FIRST: u32 = WM_USER + 0x400;
const EE_CUSTOM_BAR_OPEN: u32 = EE_FIRST + 73;

// Custom message for repaint from background thread
const WM_APP: u32 = 0x8000;
const WM_APP_REPAINT: u32 = WM_APP + 1;

// Custom Bar Positions
// const CUSTOM_BAR_LEFT: i32 = 0;
// const CUSTOM_BAR_TOP: i32 = 1;
// const CUSTOM_BAR_RIGHT: i32 = 2;
const CUSTOM_BAR_BOTTOM: i32 = 3;

#[repr(C)]
#[allow(non_snake_case)]
struct CUSTOM_BAR_INFO {
    cbSize: usize,
    hwndCustomBar: HWND,
    hwndClient: HWND,
    pszTitle: PCWSTR,
    iPos: i32,
}

static CLASS_REGISTERED: AtomicBool = AtomicBool::new(false);
const CLASS_NAME: PCWSTR = w!("EmEditorTerminalClass");

static TERMINAL_DATA: OnceLock<Arc<Mutex<TerminalData>>> = OnceLock::new();

// Keyboard hook for capturing Backspace
thread_local! {
    static KEYBOARD_HOOK: RefCell<Option<HHOOK>> = const { RefCell::new(None) };
    static TERMINAL_HWND: RefCell<Option<HWND>> = const { RefCell::new(None) };
}

#[derive(Clone, Copy)]
/// Wrapper around a Windows `HWND` handle that is treated as `Send` and `Sync`.
///
/// On Windows, many operations on `HWND` (such as `PostMessageW`) are documented
/// as cross-thread safe, but some operations must only be performed on the thread
/// that created/owns the window (for example, most UI updates and message loops).
struct SendHWND(HWND);

/// SAFETY:
/// - The `HWND` handle value itself may be moved across threads.
/// - Callers must only perform operations from other threads that the Windows
///   API documents as thread-safe for `HWND` (e.g., `PostMessageW`).
/// - Thread-affine operations must still be invoked on the thread that owns the window.
unsafe impl Send for SendHWND {}

/// SAFETY:
/// - Sharing an `HWND` between threads does not in itself cause data races, as
///   long as all threads confine thread-affine operations to the owning thread
///   and only perform cross-thread-safe operations from other threads.
unsafe impl Sync for SendHWND {}

struct TerminalData {
    service: TerminalService,
    renderer: TerminalRenderer,
    window_handle: Option<SendHWND>,
    composition: Option<CompositionData>,
}

fn get_terminal_data() -> Arc<Mutex<TerminalData>> {
    TERMINAL_DATA
        .get_or_init(|| {
            Arc::new(Mutex::new(TerminalData {
                service: TerminalService::new(80, 25),
                renderer: TerminalRenderer::new(),
                window_handle: None,
                composition: None,
            }))
        })
        .clone()
}

// Helper to update IME window position
fn update_ime_window_position(hwnd: HWND) {
    let data_arc = get_terminal_data();
    let data = data_arc.lock().unwrap();

    if let Some(metrics) = data.renderer.get_metrics() {
        let (cursor_x, cursor_y) = data.service.buffer.get_cursor_pos();
        let display_cols = data.service.buffer.get_display_width_up_to(cursor_y, cursor_x);

        let pixel_x = display_cols as i32 * metrics.base_width;
        let pixel_y = cursor_y as i32 * metrics.char_height;

        log::debug!(
            "Updating IME Window position: cursor=({}, {}), pixel=({}, {})",
            cursor_x,
            cursor_y,
            pixel_x,
            pixel_y
        );

        unsafe {
            // Update system caret position (IME uses this as reference)
            let _ = SetCaretPos(pixel_x, pixel_y);

            // Explicitly set composition window position
            let himc = ImmGetContext(hwnd);
            if !himc.0.is_null() {
                let form = COMPOSITIONFORM {
                    dwStyle: CFS_POINT,
                    ptCurrentPos: windows::Win32::Foundation::POINT {
                        x: pixel_x,
                        y: pixel_y,
                    },
                    rcArea: windows::Win32::Foundation::RECT::default(),
                };
                let _ = ImmSetCompositionWindow(himc, &form);
                let _ = ImmReleaseContext(hwnd, himc);
            }
        }
    }
}

// Helper to check if IME is composing
fn is_ime_composing(hwnd: HWND) -> bool {
    unsafe {
        let himc = ImmGetContext(hwnd);
        if himc.0.is_null() {
            return false;
        }
        let len = ImmGetCompositionStringW(himc, GCS_COMPSTR, None, 0);
        let _ = ImmReleaseContext(hwnd, himc);
        if len < 0 {
            // ImmGetCompositionStringW failed; treat as not composing
            return false;
        }
        len > 0
    }
}

pub fn open_custom_bar(hwnd_editor: HWND) -> bool {
    unsafe {
        let h_instance = crate::get_instance_handle();

        // Check if already open
        let data_arc = get_terminal_data();
        {
            let data = data_arc.lock().unwrap();
            if let Some(h) = data.window_handle {
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
            RegisterClassW(&wc);
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
                    let mut data = data_arc.lock().unwrap();
                    // Double check if another window was created concurrently (unlikely in UI thread but safe)
                    if let Some(h) = data.window_handle {
                        // Another window exists, destroy this one and focus the existing one
                        let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd_client);
                        let _ = SetFocus(h.0);
                        return false;
                    }
                    data.window_handle = Some(SendHWND(hwnd_client));
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
                    let data = data_arc.lock().unwrap();
                    let (char_width, char_height) =
                        if let Some(metrics) = data.renderer.get_metrics() {
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

                // Start ConPTY
                match ConPTY::new("pwsh.exe", initial_cols, initial_rows) {
                    Ok(conpty) => {
                        log::info!(
                            "ConPTY started successfully with size {}x{}",
                            initial_cols,
                            initial_rows
                        );

                        let data_arc = get_terminal_data();
                        let output_handle = conpty.get_output_handle();
                        {
                            let mut data = data_arc.lock().unwrap();
                            data.service.conpty = Some(conpty);
                            // Sync buffer size with ConPTY
                            data.service.resize(initial_cols as usize, initial_rows as usize);
                        }

                        let output_handle_raw = output_handle.0 .0 as usize;
                        let hwnd_client_ptr = hwnd_client.0 as usize;

                        thread::spawn(move || {
                            let mut buffer = [0u8; 1024];
                            let mut bytes_read = 0;
                            loop {
                                if let Err(e) = ReadFile(
                                    HANDLE(output_handle_raw as *mut _),
                                    Some(&mut buffer),
                                    Some(&mut bytes_read),
                                    None,
                                ) {
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
                                    "ConPTY Raw Output ({} bytes): {}",
                                    bytes_read,
                                    hex_output
                                );
                                let output = String::from_utf8_lossy(raw_bytes);
                                log::debug!("ConPTY Output: {}", output);

                                {
                                    let mut data = data_arc.lock().unwrap();
                                    data.service.process_output(&output);
                                }

                                // Trigger repaint via PostMessage (thread-safe)
                                let _ = PostMessageW(
                                    HWND(hwnd_client_ptr as *mut _),
                                    WM_APP_REPAINT,
                                    WPARAM(0),
                                    LPARAM(0),
                                );
                            }
                            log::info!("ConPTY output thread finished");
                        });
                        true
                    }
                    Err(e) => {
                        log::error!("Failed to start ConPTY: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to create custom bar window: {}", e);
                false
            }
        }
    }
}

// Convert virtual key code to VT sequence
fn vk_to_vt_sequence(
    vk_code: u16,
    ctrl_pressed: bool,
    shift_pressed: bool,
    alt_pressed: bool,
) -> Option<&'static [u8]> {
    // Handle Ctrl+ combinations first
    if ctrl_pressed && !alt_pressed {
        if let 0x41..=0x5A = vk_code {
            // Ctrl+A through Ctrl+Z
            // Return control character (A=1, B=2, ..., Z=26)
            let ctrl_char = (vk_code - 0x40) as u8;
            return match ctrl_char {
                1 => Some(b"\x01"),  // Ctrl+A
                2 => Some(b"\x02"),  // Ctrl+B
                3 => Some(b"\x03"),  // Ctrl+C
                4 => Some(b"\x04"),  // Ctrl+D
                5 => Some(b"\x05"),  // Ctrl+E
                6 => Some(b"\x06"),  // Ctrl+F
                7 => Some(b"\x07"),  // Ctrl+G
                8 => Some(b"\x08"),  // Ctrl+H (same as Backspace)
                9 => Some(b"\x09"),  // Ctrl+I (same as Tab)
                10 => Some(b"\x0a"), // Ctrl+J
                11 => Some(b"\x0b"), // Ctrl+K
                12 => Some(b"\x0c"), // Ctrl+L
                13 => Some(b"\x0d"), // Ctrl+M (same as Enter)
                14 => Some(b"\x0e"), // Ctrl+N
                15 => Some(b"\x0f"), // Ctrl+O
                16 => Some(b"\x10"), // Ctrl+P
                17 => Some(b"\x11"), // Ctrl+Q
                18 => Some(b"\x12"), // Ctrl+R
                19 => Some(b"\x13"), // Ctrl+S
                20 => Some(b"\x14"), // Ctrl+T
                21 => Some(b"\x15"), // Ctrl+U
                22 => Some(b"\x16"), // Ctrl+V
                23 => Some(b"\x17"), // Ctrl+W
                24 => Some(b"\x18"), // Ctrl+X
                25 => Some(b"\x19"), // Ctrl+Y
                26 => Some(b"\x1a"), // Ctrl+Z
                _ => None,
            };
        }
    }

    // Special keys with modifiers
    match vk_code {
        k if k == VK_UP.0 => {
            if ctrl_pressed {
                Some(b"\x1b[1;5A")
            } else if shift_pressed {
                Some(b"\x1b[1;2A")
            } else if alt_pressed {
                Some(b"\x1b[1;3A")
            } else {
                Some(b"\x1b[A")
            }
        }
        k if k == VK_DOWN.0 => {
            if ctrl_pressed {
                Some(b"\x1b[1;5B")
            } else if shift_pressed {
                Some(b"\x1b[1;2B")
            } else if alt_pressed {
                Some(b"\x1b[1;3B")
            } else {
                Some(b"\x1b[B")
            }
        }
        k if k == VK_RIGHT.0 => {
            if ctrl_pressed {
                Some(b"\x1b[1;5C")
            } else if shift_pressed {
                Some(b"\x1b[1;2C")
            } else if alt_pressed {
                Some(b"\x1b[1;3C")
            } else {
                Some(b"\x1b[C")
            }
        }
        k if k == VK_LEFT.0 => {
            if ctrl_pressed {
                Some(b"\x1b[1;5D")
            } else if shift_pressed {
                Some(b"\x1b[1;2D")
            } else if alt_pressed {
                Some(b"\x1b[1;3D")
            } else {
                Some(b"\x1b[D")
            }
        }
        k if k == VK_HOME.0 => {
            if ctrl_pressed {
                Some(b"\x1b[1;5H")
            } else if shift_pressed {
                Some(b"\x1b[1;2H")
            } else {
                Some(b"\x1b[H")
            }
        }
        k if k == VK_END.0 => {
            if ctrl_pressed {
                Some(b"\x1b[1;5F")
            } else if shift_pressed {
                Some(b"\x1b[1;2F")
            } else {
                Some(b"\x1b[F")
            }
        }
        k if k == VK_DELETE.0 => Some(b"\x1b[3~"),
        k if k == VK_INSERT.0 => Some(b"\x1b[2~"),
        k if k == VK_PRIOR.0 => Some(b"\x1b[5~"), // Page Up
        k if k == VK_NEXT.0 => Some(b"\x1b[6~"),  // Page Down
        k if k == VK_BACK.0 => Some(b"\x7f"),     // Backspace (DEL)
        k if k == VK_RETURN.0 => Some(b"\r"),     // Enter
        k if k == VK_TAB.0 => Some(b"\t"),        // Tab
        k if k == VK_ESCAPE.0 => Some(b"\x1b"),   // Escape
        k if k == VK_F1.0 => Some(b"\x1bOP"),
        k if k == VK_F2.0 => Some(b"\x1bOQ"),
        k if k == VK_F3.0 => Some(b"\x1bOR"),
        k if k == VK_F4.0 => Some(b"\x1bOS"),
        k if k == VK_F5.0 => Some(b"\x1b[15~"),
        k if k == VK_F6.0 => Some(b"\x1b[17~"),
        k if k == VK_F7.0 => Some(b"\x1b[18~"),
        k if k == VK_F8.0 => Some(b"\x1b[19~"),
        k if k == VK_F9.0 => Some(b"\x1b[20~"),
        k if k == VK_F10.0 => Some(b"\x1b[21~"),
        k if k == VK_F11.0 => Some(b"\x1b[23~"),
        k if k == VK_F12.0 => Some(b"\x1b[24~"),
        _ => None,
    }
}

// Helper function to write data to ConPTY input pipe
fn write_to_conpty(handle: HANDLE, data: &[u8]) -> Result<(), windows::core::Error> {
    let mut bytes_written = 0;
    unsafe {
        WriteFile(handle, Some(data), Some(&mut bytes_written), None)?;
    }
    log::debug!("Wrote {} bytes to ConPTY: {:?}", bytes_written, data);
    Ok(())
}

#[allow(dead_code)]
pub fn send_input(text: &str) {
    let data_arc = get_terminal_data();
    let data = data_arc.lock().unwrap();
    if let Some(conpty) = &data.service.conpty {
        let utf8_bytes = text.as_bytes();
        let mut bytes_written = 0;
        unsafe {
            let _ = WriteFile(
                conpty.get_input_handle().0,
                Some(utf8_bytes),
                Some(&mut bytes_written),
                None,
            );
            // 改行を送る
            let _ = WriteFile(
                conpty.get_input_handle().0,
                Some(b"\r"),
                Some(&mut bytes_written),
                None,
            );
        }
    }
}

pub fn cleanup_terminal() {
    log::info!("cleanup_terminal: Starting cleanup");
    let data_arc = get_terminal_data();
    let mut data = data_arc.lock().unwrap();
    if let Some(_conpty) = data.service.conpty.take() {
        log::info!("ConPTY instance found, will be dropped and cleaned up");
        // Drop happens automatically
    } else {
        log::info!("No ConPTY instance to clean up");
    }
}

fn send_key_to_conpty(vk_code: u16) -> bool {
    let ctrl_pressed = unsafe { GetKeyState(VK_CONTROL.0 as i32) } < 0;
    let shift_pressed = unsafe { GetKeyState(VK_SHIFT.0 as i32) } < 0;
    let alt_pressed = unsafe { GetKeyState(VK_MENU.0 as i32) } < 0;

    // Skip system shortcuts in hook to allow Windows/EmEditor to handle them
    if alt_pressed
        && (vk_code == VK_F4.0
            || vk_code == VK_TAB.0
            || vk_code == VK_SPACE.0
            || vk_code == VK_ESCAPE.0)
    {
        return false;
    }

    if let Some(vt_sequence) = vk_to_vt_sequence(vk_code, ctrl_pressed, shift_pressed, alt_pressed)
    {
        let handle = {
            let data_arc = get_terminal_data();
            let data = data_arc.lock().unwrap();
            data.service.conpty.as_ref().map(|c| c.get_input_handle().0)
        };

        if let Some(handle) = handle {
            log::debug!(
                "Keyboard hook: Sending VT sequence for vk_code 0x{:04X}",
                vk_code
            );
            if let Err(e) = write_to_conpty(handle, vt_sequence) {
                log::error!("Keyboard hook: Failed to write VT sequence: {}", e);
            }
            return true;
        } else {
            log::warn!("Keyboard hook: No ConPTY available");
        }
    }
    false
}

extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let vk_code = wparam.0 as u16;
        let key_up = (lparam.0 >> 31) & 1; // bit 31 = transition state (1 = key up)

        // Only process key down events
        if key_up == 0 {
            // Check for IME composition first
            let is_composing = TERMINAL_HWND.with(|h| {
                if let Some(hwnd) = *h.borrow() {
                    is_ime_composing(hwnd)
                } else {
                    false
                }
            });

            if !is_composing {
                // Check if this is a key we want to handle (Arrow keys, Ctrl+Keys, etc.)
                // We use the same logic as vk_to_vt_sequence.
                // If it returns true, we consumed the key logically and sent it to ConPty.
                // Note: For WH_KEYBOARD hooks, the return value is effectively ignored by the system,
                // and we still call CallNextHookEx below to continue the hook chain.
                // Returning 1 here only indicates internally that we consumed the key; it does NOT
                // block EmEditor/Windows from processing the event when using WH_KEYBOARD.
                if send_key_to_conpty(vk_code) {
                    log::debug!("Keyboard hook: Consumed vk_code 0x{:04X}", vk_code);
                    return LRESULT(1);
                }
            } else {
                log::debug!(
                    "Keyboard hook: IME composing, skipping hook for vk_code 0x{:04X}",
                    vk_code
                );
            }
        }
    }

    KEYBOARD_HOOK.with(|hook| {
        let hook_ref = hook.borrow();
        if let Some(hhook) = *hook_ref {
            unsafe { CallNextHookEx(hhook, code, wparam, lparam) }
        } else {
            unsafe { CallNextHookEx(None, code, wparam, lparam) }
        }
    })
}

fn install_keyboard_hook() {
    KEYBOARD_HOOK.with(|hook| {
        let mut hook_ref = hook.borrow_mut();
        if hook_ref.is_none() {
            unsafe {
                let h = SetWindowsHookExW(
                    WH_KEYBOARD,
                    Some(keyboard_hook_proc),
                    None,
                    windows::Win32::System::Threading::GetCurrentThreadId(),
                );
                match h {
                    Ok(hhook) => {
                        log::info!("Keyboard hook installed successfully");
                        *hook_ref = Some(hhook);
                    }
                    Err(e) => {
                        log::error!("Failed to install keyboard hook: {}", e);
                    }
                }
            }
        }
    });
}

fn uninstall_keyboard_hook() {
    KEYBOARD_HOOK.with(|hook| {
        let mut hook_ref = hook.borrow_mut();
        if let Some(hhook) = hook_ref.take() {
            unsafe {
                let _ = UnhookWindowsHookEx(hhook);
                log::info!("Keyboard hook uninstalled");
            }
        }
    });
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_PAINT => {
            unsafe {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);

                let data_arc = get_terminal_data();
                let mut data = data_arc.lock().unwrap();

                let mut client_rect = windows::Win32::Foundation::RECT::default();
                let _ =
                    windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut client_rect);

                let TerminalData {
                    ref service,
                    ref mut renderer,
                    ref composition,
                    ..
                } = *data;

                renderer.render(hdc, &client_rect, &service.buffer, composition.as_ref());

                let _ = EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            log::info!("WM_LBUTTONDOWN: Setting focus");
            unsafe {
                let _ = SetFocus(hwnd);
            }
            LRESULT(0)
        }
        WM_SETFOCUS => {
            log::info!("WM_SETFOCUS: Focus received, installing keyboard hook");
            TERMINAL_HWND.with(|h| {
                *h.borrow_mut() = Some(hwnd);
            });

            // Create a caret so SetCaretPos works
            let data_arc = get_terminal_data();
            let data = data_arc.lock().unwrap();
            let char_height = data
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
                // We don't call ShowCaret(hwnd) because we draw our own cursor overlay
            }

            install_keyboard_hook();
            LRESULT(0)
        }
        WM_KILLFOCUS => {
            log::info!("WM_KILLFOCUS: Focus lost, uninstalling keyboard hook");
            unsafe {
                let _ = DestroyCaret();
            }
            uninstall_keyboard_hook();
            LRESULT(0)
        }
        WM_KEYDOWN => {
            let vk_code = wparam.0 as u16;
            log::debug!("WM_KEYDOWN received: 0x{:04X}", vk_code);

            // Check modifier key states
            let ctrl_pressed = unsafe { GetKeyState(VK_CONTROL.0 as i32) } < 0;
            let shift_pressed = unsafe { GetKeyState(VK_SHIFT.0 as i32) } < 0;
            let alt_pressed = unsafe { GetKeyState(VK_MENU.0 as i32) } < 0;

            log::debug!(
                "Modifiers - Ctrl: {}, Shift: {}, Alt: {}",
                ctrl_pressed,
                shift_pressed,
                alt_pressed
            );

            // Try to convert to VT sequence
            if let Some(vt_sequence) =
                vk_to_vt_sequence(vk_code, ctrl_pressed, shift_pressed, alt_pressed)
            {
                log::info!(
                    "WM_KEYDOWN: Sending VT sequence for vk_code 0x{:04X}: {:?}",
                    vk_code,
                    vt_sequence
                );

                let handle = {
                    let data_arc = get_terminal_data();
                    let data = data_arc.lock().unwrap();
                    data.service.conpty.as_ref().map(|c| c.get_input_handle().0)
                };

                if let Some(handle) = handle {
                    if let Err(e) = write_to_conpty(handle, vt_sequence) {
                        log::error!("Failed to write VT sequence: {}", e);
                    }
                } else {
                    log::warn!("No ConPTY available");
                }

                LRESULT(0)
            } else {
                // Not a special key, let WM_CHAR handle it
                log::debug!(
                    "WM_KEYDOWN: No VT sequence for vk_code 0x{:04X}, passing to DefWindowProc",
                    vk_code
                );
                unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
            }
        }
        WM_SYSKEYDOWN => {
            let vk_code = wparam.0 as u16;
            log::debug!("WM_SYSKEYDOWN received: 0x{:04X}", vk_code);

            // Exclusion list for system shortcuts that should be handled by Windows/EmEditor
            if vk_code == VK_TAB.0
                || vk_code == VK_F4.0
                || vk_code == VK_SPACE.0
                || vk_code == VK_ESCAPE.0
            {
                return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
            }

            if vk_code == VK_MENU.0 {
                log::debug!(
                    "Alt key (VK_MENU) pressed in WM_SYSKEYDOWN, suppressing DefWindowProcW"
                );
                return LRESULT(0);
            }

            // Handle Alt + Key combinations for TUI
            let ctrl_pressed = unsafe { GetKeyState(VK_CONTROL.0 as i32) } < 0;
            let shift_pressed = unsafe { GetKeyState(VK_SHIFT.0 as i32) } < 0;
            let alt_pressed = true; // WM_SYSKEYDOWN usually implies Alt is pressed

            // 1. Try special keys (Arrows, F-keys, etc.) via existing vk_to_vt_sequence
            if let Some(vt_sequence) =
                vk_to_vt_sequence(vk_code, ctrl_pressed, shift_pressed, alt_pressed)
            {
                log::debug!(
                    "WM_SYSKEYDOWN: Sending VT sequence for Alt combination: {:?}",
                    vt_sequence
                );
                let handle = {
                    let data_arc = get_terminal_data();
                    let data = data_arc.lock().unwrap();
                    data.service.conpty.as_ref().map(|c| c.get_input_handle().0)
                };
                if let Some(handle) = handle {
                    if let Err(e) = write_to_conpty(handle, vt_sequence) {
                        log::error!("WM_SYSKEYDOWN: Failed to write VT sequence: {}", e);
                    }
                }
                return LRESULT(0);
            }

            // 2. Handle Alt + Letter/Number (Meta key)
            // Send ESC (0x1B) followed by the character
            if (0x30..=0x39).contains(&vk_code) || (0x41..=0x5A).contains(&vk_code) {
                let mut char_to_send = vk_code as u8;
                if (0x41..=0x5A).contains(&vk_code) && !shift_pressed {
                    char_to_send = (vk_code + 0x20) as u8; // To lowercase
                }
                log::debug!(
                    "WM_SYSKEYDOWN: Sending Meta sequence: ESC + '{}'",
                    char_to_send as char
                );
                let seq = [0x1bu8, char_to_send];
                let handle = {
                    let data_arc = get_terminal_data();
                    let data = data_arc.lock().unwrap();
                    data.service.conpty.as_ref().map(|c| c.get_input_handle().0)
                };
                if let Some(handle) = handle {
                    if let Err(e) = write_to_conpty(handle, &seq) {
                        log::error!("WM_SYSKEYDOWN: Failed to write Meta sequence: {}", e);
                    }
                }
                return LRESULT(0);
            }

            // For other system keys, pass to DefWindowProcW
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_SYSKEYUP => {
            let vk_code = wparam.0 as u16;
            log::debug!("WM_SYSKEYUP received: 0x{:04X}", vk_code);

            if vk_code == VK_MENU.0 {
                log::debug!(
                    "Alt key (VK_MENU) released in WM_SYSKEYUP, suppressing DefWindowProcW"
                );
                return LRESULT(0);
            }

            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_KEYUP => {
            let vk_code = wparam.0 as u16;
            if vk_code == VK_MENU.0 {
                log::debug!("WM_KEYUP received for VK_MENU (Alt)");
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_SYSCHAR => {
            let char_code = wparam.0 as u16;
            log::debug!("WM_SYSCHAR received: 0x{:04X}", char_code);

            // Allow Alt+Space (System Menu) to pass through
            if char_code == 0x0020 {
                return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
            }

            // Alt+Key combinations often generate WM_SYSCHAR. Suppress it to prevent system beeps or menus.
            LRESULT(0)
        }
        WM_SYSCOMMAND => {
            let cmd = wparam.0 & 0xFFF0; // Low 4 bits are reserved
            log::debug!("WM_SYSCOMMAND received: 0x{:04X}", cmd);
            if cmd == 0xF100 {
                // SC_KEYMENU
                // When SC_KEYMENU is sent by a keystroke, lParam contains the char code in low word.
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
        WM_GETDLGCODE => {
            log::debug!("WM_GETDLGCODE received");
            LRESULT(DLGC_WANTALLKEYS as isize)
        }
        WM_CHAR => {
            let char_code = wparam.0 as u16;
            log::debug!(
                "WM_CHAR received: 0x{:04X} ({})",
                char_code,
                String::from_utf16_lossy(&[char_code])
            );

            // Skip characters that were already handled by WM_KEYDOWN as special keys
            // This includes Enter (0x0D), Tab (0x09), Escape (0x1B), Backspace (0x08)
            if char_code == 0x0D || char_code == 0x09 || char_code == 0x1B || char_code == 0x08 {
                log::debug!(
                    "WM_CHAR: Skipping char 0x{:04X} (already handled by WM_KEYDOWN)",
                    char_code
                );
                return LRESULT(0);
            }

            let data_arc = get_terminal_data();
            let data = data_arc.lock().unwrap();

            if let Some(conpty) = &data.service.conpty {
                let s = String::from_utf16_lossy(&[char_code]);
                let utf8_bytes = s.as_bytes();
                let mut bytes_written = 0;
                unsafe {
                    let _ = WriteFile(
                        conpty.get_input_handle().0,
                        Some(utf8_bytes),
                        Some(&mut bytes_written),
                        None,
                    );
                }
            }
            LRESULT(0)
        }
        msg if msg == WM_APP_REPAINT => {
            // Handle repaint request from background thread
            unsafe {
                let _ = InvalidateRect(hwnd, None, BOOL(0));
            }
            LRESULT(0)
        }
        WM_SIZE => {
            let width = (lparam.0 & 0xFFFF) as i32;
            let height = ((lparam.0 >> 16) & 0xFFFF) as i32;
            log::info!("WM_SIZE: width={}, height={}", width, height);

            let data_arc = get_terminal_data();
            let mut data = data_arc.lock().unwrap();

            // Use cached metrics if available, otherwise approximation
            let (char_width, char_height) = if let Some(metrics) = data.renderer.get_metrics() {
                (metrics.base_width, metrics.char_height)
            } else {
                (8, 16) // Fallback approximation
            };

            // Convert pixel dimensions to console character dimensions
            let cols = (width / char_width).max(1) as i16;
            let rows = (height / char_height).max(1) as i16;

            log::info!("Resizing ConPTY to cols={}, rows={}", cols, rows);

            data.service.resize(cols as usize, rows as usize);
            LRESULT(0)
        }
        WM_DESTROY => {
            log::info!("WM_DESTROY: Cleaning up terminal resources");
            uninstall_keyboard_hook();

            // Clean up ConPTY and renderer resources
            let data_arc = get_terminal_data();
            let mut data = data_arc.lock().unwrap();

            data.window_handle = None;
            data.renderer.clear_resources();

            if let Some(_conpty) = data.service.conpty.take() {
                log::info!("ConPTY will be dropped and cleaned up");
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
