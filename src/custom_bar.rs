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
use windows::Win32::UI::Input::KeyboardAndMouse::{SetFocus, VK_BACK, GetKeyState};
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

fn send_backspace_to_conpty() -> Option<HWND> {
    // Get the handle while holding the lock, then release the lock before WriteFile
    // to avoid deadlock with the output thread
    let handle = {
        let data_arc = get_terminal_data();
        let data = data_arc.lock().unwrap();
        if let Some(conpty) = &data.conpty {
            Some(conpty.get_input_handle().0)
        } else {
            None
        }
    }; // Lock is released here
    
    if let Some(handle) = handle {
        let mut bytes_written = 0;
        log::debug!("Attempting to write backspace (0x08) to handle: {:?}", handle);
        unsafe {
            let result = WriteFile(
                handle,
                Some(b"\x08"),
                Some(&mut bytes_written),
                None
            );
            match result {
                Ok(_) => {
                    log::info!("Backspace (0x08) sent to ConPTY: {} bytes written", bytes_written);
                    // ConPTY will echo back the backspace sequence, so no local processing needed
                }
                Err(e) => {
                    log::error!("WriteFile failed for backspace: {}", e);
                }
            }
        }
        // Return None - ConPTY response will trigger repaint via output thread
        None
    } else {
        log::warn!("send_backspace_to_conpty: No ConPTY available");
        None
    }
}

extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let vk_code = wparam.0 as u16;
        let key_up = (lparam.0 >> 31) & 1; // bit 31 = transition state (1 = key up)

        // Handle Backspace - WM_KEYDOWN doesn't receive it due to EmEditor consuming it
        if vk_code == VK_BACK.0 && key_up == 0 {
            log::debug!("Keyboard hook: Backspace detected (key down)");
            if let Some(hwnd) = send_backspace_to_conpty() {
                // Trigger repaint after local backspace processing
                let _ = unsafe { InvalidateRect(hwnd, None, BOOL(0)) };
            }
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
            
            if vk_code == VK_BACK.0 {
                log::info!("Handling Backspace via WM_KEYDOWN");
                let data_arc = get_terminal_data();
                let data = data_arc.lock().unwrap();
                
                if let Some(conpty) = &data.conpty {
                    let utf8_bytes = b"\x08";
                    let mut bytes_written = 0;
                    let handle = conpty.get_input_handle().0;
                    log::debug!("Writing backspace to ConPTY handle: {:?}", handle);
                    unsafe {
                        let result = WriteFile(
                            handle,
                            Some(utf8_bytes),
                            Some(&mut bytes_written),
                            None
                        );
                        match result {
                            Ok(_) => log::info!("WM_KEYDOWN: Backspace sent, {} bytes written", bytes_written),
                            Err(e) => log::error!("WM_KEYDOWN: WriteFile failed: {}", e),
                        }
                    }
                }
                LRESULT(0)
            } else {
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
