use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM, HANDLE, BOOL};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, TextOutW, PAINTSTRUCT, HBRUSH, COLOR_WINDOW, InvalidateRect};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassW, LoadCursorW,
    CS_HREDRAW, CS_VREDRAW, IDC_ARROW, WM_PAINT, WNDCLASSW,
    WS_CHILD, WS_VISIBLE, WS_CLIPCHILDREN, WS_CLIPSIBLINGS,
    SendMessageW, PostMessageW, WM_CHAR, WM_LBUTTONDOWN, WM_SETFOCUS, WM_KILLFOCUS, WM_KEYDOWN, WM_GETDLGCODE, DLGC_WANTALLKEYS,
    SetWindowsHookExW, UnhookWindowsHookEx, CallNextHookEx, WH_KEYBOARD, HHOOK, WM_SIZE, WM_DESTROY,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SetFocus, GetKeyState,
    VK_BACK, VK_RETURN, VK_LEFT, VK_RIGHT, VK_UP, VK_DOWN,
    VK_HOME, VK_END, VK_DELETE, VK_INSERT, VK_PRIOR, VK_NEXT,
    VK_TAB, VK_ESCAPE, VK_F1, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6,
    VK_F7, VK_F8, VK_F9, VK_F10, VK_F11, VK_F12,
    VK_CONTROL, VK_SHIFT, VK_MENU,
};
use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile};
use std::ffi::c_void;
use std::mem::size_of;
use std::thread;
use std::sync::{Arc, Mutex, OnceLock};
use std::cell::RefCell;
use crate::conpty::ConPTY;
use crate::terminal::TerminalBuffer;

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

static mut CLASS_REGISTERED: bool = false;
const CLASS_NAME: PCWSTR = w!("EmEditorTerminalClass");

static TERMINAL_DATA: OnceLock<Arc<Mutex<TerminalData>>> = OnceLock::new();

// Keyboard hook for capturing Backspace
thread_local! {
    static KEYBOARD_HOOK: RefCell<Option<HHOOK>> = const { RefCell::new(None) };
    static TERMINAL_HWND: RefCell<Option<HWND>> = const { RefCell::new(None) };
}

struct TerminalData {
    buffer: TerminalBuffer,
    conpty: Option<ConPTY>,
}

fn get_terminal_data() -> Arc<Mutex<TerminalData>> {
    TERMINAL_DATA.get_or_init(|| {
        Arc::new(Mutex::new(TerminalData {
            buffer: TerminalBuffer::new(80, 25),
            conpty: None,
        }))
    }).clone()
}

pub fn open_custom_bar(hwnd_editor: HWND) {
    unsafe {
        let h_instance = crate::get_instance_handle();

        if !CLASS_REGISTERED {
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
            CLASS_REGISTERED = true;
        }

        let hwnd_client_result = CreateWindowExW(
            Default::default(),
            CLASS_NAME,
            w!("Terminal"),
            WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
            0, 0, 0, 0,
            hwnd_editor,
            None,
            h_instance,
            None,
        );

        match hwnd_client_result {
            Ok(hwnd_client) => {
                if hwnd_client.0 == std::ptr::null_mut() {
                    log::error!("Failed to create custom bar window: Handle is NULL");
                    return;
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

                // Start ConPTY
                match ConPTY::new("cmd.exe", 80, 25) {
                    Ok(conpty) => {
                        log::info!("ConPTY started successfully");
                        
                        let data_arc = get_terminal_data();
                        let output_handle = conpty.get_output_handle();
                        {
                            let mut data = data_arc.lock().unwrap();
                            data.conpty = Some(conpty);
                        }

                        let output_handle_raw = output_handle.0 .0 as usize;
                        let hwnd_client_ptr = hwnd_client.0 as usize;

                        thread::spawn(move || {
                            let mut buffer = [0u8; 1024];
                            let mut bytes_read = 0;
                            loop {
                                if let Err(e) = unsafe { ReadFile(
                                    HANDLE(output_handle_raw as *mut _),
                                    Some(&mut buffer),
                                    Some(&mut bytes_read),
                                    None
                                ) } {
                                    log::error!("ReadFile failed: {}", e);
                                    break;
                                }
                                if bytes_read == 0 {
                                    log::info!("ReadFile returned 0 bytes (EOF)");
                                    break;
                                }
                                
                                let raw_bytes = &buffer[..bytes_read as usize];
                                let hex_output: String = raw_bytes.iter().map(|b| format!("{:02X} ", b)).collect();
                                log::debug!("ConPTY Raw Output ({} bytes): {}", bytes_read, hex_output);
                                let output = String::from_utf8_lossy(raw_bytes);
                                log::debug!("ConPTY Output: {}", output);
                                
                                {
                                    let mut data = data_arc.lock().unwrap();
                                    data.buffer.write_string(&output);
                                }
                                
                                // Trigger repaint via PostMessage (thread-safe)
                                let _ = PostMessageW(HWND(hwnd_client_ptr as *mut _), WM_APP_REPAINT, WPARAM(0), LPARAM(0));
                            }
                            log::info!("ConPTY output thread finished");
                        });
                    },
                    Err(e) => {
                        log::error!("Failed to start ConPTY: {}", e);
                    }
                }
            },
            Err(e) => {
                log::error!("Failed to create custom bar window: {}", e);
            }
        }
    }
}

// Convert virtual key code to VT sequence
fn vk_to_vt_sequence(vk_code: u16, ctrl_pressed: bool, shift_pressed: bool, alt_pressed: bool) -> Option<&'static [u8]> {
    // Handle Ctrl+ combinations first
    if ctrl_pressed && !alt_pressed {
        match vk_code {
            0x41..=0x5A => { // Ctrl+A through Ctrl+Z
                // Return control character (A=1, B=2, ..., Z=26)
                let ctrl_char = (vk_code - 0x40) as u8;
                return match ctrl_char {
                    1 => Some(b"\x01"),   // Ctrl+A
                    2 => Some(b"\x02"),   // Ctrl+B
                    3 => Some(b"\x03"),   // Ctrl+C
                    4 => Some(b"\x04"),   // Ctrl+D
                    5 => Some(b"\x05"),   // Ctrl+E
                    6 => Some(b"\x06"),   // Ctrl+F
                    7 => Some(b"\x07"),   // Ctrl+G
                    8 => Some(b"\x08"),   // Ctrl+H (same as Backspace)
                    9 => Some(b"\x09"),   // Ctrl+I (same as Tab)
                    10 => Some(b"\x0a"),  // Ctrl+J
                    11 => Some(b"\x0b"),  // Ctrl+K
                    12 => Some(b"\x0c"),  // Ctrl+L
                    13 => Some(b"\x0d"),  // Ctrl+M (same as Enter)
                    14 => Some(b"\x0e"),  // Ctrl+N
                    15 => Some(b"\x0f"),  // Ctrl+O
                    16 => Some(b"\x10"),  // Ctrl+P
                    17 => Some(b"\x11"),  // Ctrl+Q
                    18 => Some(b"\x12"),  // Ctrl+R
                    19 => Some(b"\x13"),  // Ctrl+S
                    20 => Some(b"\x14"),  // Ctrl+T
                    21 => Some(b"\x15"),  // Ctrl+U
                    22 => Some(b"\x16"),  // Ctrl+V
                    23 => Some(b"\x17"),  // Ctrl+W
                    24 => Some(b"\x18"),  // Ctrl+X
                    25 => Some(b"\x19"),  // Ctrl+Y
                    26 => Some(b"\x1a"),  // Ctrl+Z
                    _ => None,
                };
            }
            _ => {}
        }
    }

    // Special keys with modifiers
    match vk_code {
        k if k == VK_UP.0 => {
            if ctrl_pressed { Some(b"\x1b[1;5A") }
            else if shift_pressed { Some(b"\x1b[1;2A") }
            else if alt_pressed { Some(b"\x1b[1;3A") }
            else { Some(b"\x1b[A") }
        }
        k if k == VK_DOWN.0 => {
            if ctrl_pressed { Some(b"\x1b[1;5B") }
            else if shift_pressed { Some(b"\x1b[1;2B") }
            else if alt_pressed { Some(b"\x1b[1;3B") }
            else { Some(b"\x1b[B") }
        }
        k if k == VK_RIGHT.0 => {
            if ctrl_pressed { Some(b"\x1b[1;5C") }
            else if shift_pressed { Some(b"\x1b[1;2C") }
            else if alt_pressed { Some(b"\x1b[1;3C") }
            else { Some(b"\x1b[C") }
        }
        k if k == VK_LEFT.0 => {
            if ctrl_pressed { Some(b"\x1b[1;5D") }
            else if shift_pressed { Some(b"\x1b[1;2D") }
            else if alt_pressed { Some(b"\x1b[1;3D") }
            else { Some(b"\x1b[D") }
        }
        k if k == VK_HOME.0 => {
            if ctrl_pressed { Some(b"\x1b[1;5H") }
            else if shift_pressed { Some(b"\x1b[1;2H") }
            else { Some(b"\x1b[H") }
        }
        k if k == VK_END.0 => {
            if ctrl_pressed { Some(b"\x1b[1;5F") }
            else if shift_pressed { Some(b"\x1b[1;2F") }
            else { Some(b"\x1b[F") }
        }
        k if k == VK_DELETE.0 => Some(b"\x1b[3~"),
        k if k == VK_INSERT.0 => Some(b"\x1b[2~"),
        k if k == VK_PRIOR.0 => Some(b"\x1b[5~"),  // Page Up
        k if k == VK_NEXT.0 => Some(b"\x1b[6~"),   // Page Down
        k if k == VK_BACK.0 => Some(b"\x7f"),      // Backspace (DEL)
        k if k == VK_RETURN.0 => Some(b"\r"),      // Enter
        k if k == VK_TAB.0 => Some(b"\t"),         // Tab
        k if k == VK_ESCAPE.0 => Some(b"\x1b"),    // Escape
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
        WriteFile(
            handle,
            Some(data),
            Some(&mut bytes_written),
            None
        )?;
    }
    log::debug!("Wrote {} bytes to ConPTY: {:?}", bytes_written, data);
    Ok(())
}

pub fn send_input(text: &str) {
    let data_arc = get_terminal_data();
    let data = data_arc.lock().unwrap();
    if let Some(conpty) = &data.conpty {
        let utf8_bytes = text.as_bytes();
        let mut bytes_written = 0;
        unsafe {
            let _ = WriteFile(
                conpty.get_input_handle().0,
                Some(utf8_bytes),
                Some(&mut bytes_written),
                None
            );
            // 改行を送る
            let _ = WriteFile(
                conpty.get_input_handle().0,
                Some(b"\r"),
                Some(&mut bytes_written),
                None
            );
        }
    }
}

pub fn cleanup_terminal() {
    log::info!("cleanup_terminal: Starting cleanup");
    let data_arc = get_terminal_data();
    let mut data = data_arc.lock().unwrap();
    if let Some(_conpty) = data.conpty.take() {
        log::info!("ConPTY instance found, will be dropped and cleaned up");
        // Drop happens automatically
    } else {
        log::info!("No ConPTY instance to clean up");
    }
}

fn send_key_to_conpty(vk_code: u16) {
    let ctrl_pressed = unsafe { GetKeyState(VK_CONTROL.0 as i32) } < 0;
    let shift_pressed = unsafe { GetKeyState(VK_SHIFT.0 as i32) } < 0;
    let alt_pressed = unsafe { GetKeyState(VK_MENU.0 as i32) } < 0;

    if let Some(vt_sequence) = vk_to_vt_sequence(vk_code, ctrl_pressed, shift_pressed, alt_pressed) {
        let handle = {
            let data_arc = get_terminal_data();
            let data = data_arc.lock().unwrap();
            data.conpty.as_ref().map(|c| c.get_input_handle().0)
        };

        if let Some(handle) = handle {
            log::debug!("Keyboard hook: Sending VT sequence for vk_code 0x{:04X}", vk_code);
            if let Err(e) = write_to_conpty(handle, vt_sequence) {
                log::error!("Keyboard hook: Failed to write VT sequence: {}", e);
            }
        } else {
            log::warn!("Keyboard hook: No ConPTY available");
        }
    }
}

extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let vk_code = wparam.0 as u16;
        let key_up = (lparam.0 >> 31) & 1; // bit 31 = transition state (1 = key up)
        let prev_key_state = (lparam.0 >> 30) & 1; // bit 30 = previous key state (1 = key was down)

        // Handle Backspace on key down - WM_KEYDOWN doesn't receive it due to EmEditor consuming it
        // Only process on initial key down (prev_key_state == 0) to prevent key repeat
        if vk_code == VK_BACK.0 && key_up == 0 && prev_key_state == 0 {
            log::debug!("Keyboard hook: Backspace detected (initial key down)");
            send_key_to_conpty(vk_code);
            // Return 1 to prevent further processing (avoid double handling)
            return LRESULT(1);
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
                let data = data_arc.lock().unwrap();
                
                let mut y = 0;
                let char_height = 16; // 簡易的な固定フォント高さ
                for line in data.buffer.get_lines() {
                    let wide_line: Vec<u16> = line.encode_utf16().collect();
                    let _ = TextOutW(hdc, 0, y, &wide_line);
                    y += char_height;
                }
                
                let _ = EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            log::info!("WM_LBUTTONDOWN: Setting focus");
            unsafe { let _ = SetFocus(hwnd); }
            LRESULT(0)
        }
        WM_SETFOCUS => {
            log::info!("WM_SETFOCUS: Focus received, installing keyboard hook");
            TERMINAL_HWND.with(|h| {
                *h.borrow_mut() = Some(hwnd);
            });
            install_keyboard_hook();
            LRESULT(0)
        }
        WM_KILLFOCUS => {
            log::info!("WM_KILLFOCUS: Focus lost, uninstalling keyboard hook");
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

            log::debug!("Modifiers - Ctrl: {}, Shift: {}, Alt: {}", ctrl_pressed, shift_pressed, alt_pressed);

            // Try to convert to VT sequence
            if let Some(vt_sequence) = vk_to_vt_sequence(vk_code, ctrl_pressed, shift_pressed, alt_pressed) {
                log::info!("WM_KEYDOWN: Sending VT sequence for vk_code 0x{:04X}: {:?}", vk_code, vt_sequence);

                let handle = {
                    let data_arc = get_terminal_data();
                    let data = data_arc.lock().unwrap();
                    data.conpty.as_ref().map(|c| c.get_input_handle().0)
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
                log::debug!("WM_KEYDOWN: No VT sequence for vk_code 0x{:04X}, passing to DefWindowProc", vk_code);
                unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
            }
        }
        WM_GETDLGCODE => {
            log::debug!("WM_GETDLGCODE received");
            LRESULT(DLGC_WANTALLKEYS as isize)
        }
        WM_CHAR => {
            let char_code = wparam.0 as u16;
            log::debug!("WM_CHAR received: 0x{:04X} ({})", char_code, String::from_utf16_lossy(&[char_code]));

            // Skip characters that were already handled by WM_KEYDOWN as special keys
            // This includes Enter (0x0D), Tab (0x09), Escape (0x1B), Backspace (0x08)
            if char_code == 0x0D || char_code == 0x09 || char_code == 0x1B || char_code == 0x08 {
                log::debug!("WM_CHAR: Skipping char 0x{:04X} (already handled by WM_KEYDOWN)", char_code);
                return LRESULT(0);
            }

            let data_arc = get_terminal_data();
            let data = data_arc.lock().unwrap();

            if let Some(conpty) = &data.conpty {
                let s = String::from_utf16_lossy(&[char_code]);
                let utf8_bytes = s.as_bytes();
                let mut bytes_written = 0;
                unsafe {
                    let _ = WriteFile(
                        conpty.get_input_handle().0,
                        Some(utf8_bytes),
                        Some(&mut bytes_written),
                        None
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

            // Convert pixel dimensions to console character dimensions
            let char_width = 8; // Approximate fixed-width character width
            let char_height = 16; // Fixed font height from WM_PAINT
            let cols = (width / char_width).max(1) as i16;
            let rows = (height / char_height).max(1) as i16;

            log::info!("Resizing ConPTY to cols={}, rows={}", cols, rows);

            let data_arc = get_terminal_data();
            let mut data = data_arc.lock().unwrap();
            if let Some(conpty) = &data.conpty {
                match conpty.resize(cols, rows) {
                    Ok(_) => {
                        log::info!("ConPTY resized successfully to {}x{}", cols, rows);
                        // Update the buffer size as well
                        data.buffer = TerminalBuffer::new(cols as usize, rows as usize);
                    }
                    Err(e) => {
                        log::error!("Failed to resize ConPTY: {}", e);
                    }
                }
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            log::info!("WM_DESTROY: Cleaning up terminal resources");
            uninstall_keyboard_hook();

            // Clean up ConPTY
            let data_arc = get_terminal_data();
            let mut data = data_arc.lock().unwrap();
            if let Some(_conpty) = data.conpty.take() {
                log::info!("ConPTY will be dropped and cleaned up");
                // Drop happens automatically when _conpty goes out of scope
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
