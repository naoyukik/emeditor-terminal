use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM, HANDLE, BOOL};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, TextOutW, PAINTSTRUCT, HBRUSH, COLOR_WINDOW, InvalidateRect};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassW, LoadCursorW,
    CS_HREDRAW, CS_VREDRAW, IDC_ARROW, WM_PAINT, WNDCLASSW,
    WS_CHILD, WS_VISIBLE, WS_CLIPCHILDREN, WS_CLIPSIBLINGS,
    SendMessageW, WM_CHAR,
};
use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile};
use std::ffi::c_void;
use std::mem::size_of;
use std::thread;
use std::sync::{Arc, Mutex, OnceLock};
use crate::conpty::ConPTY;
use crate::terminal::TerminalBuffer;

// Constants from EmEditor SDK
const WM_USER: u32 = 0x0400;
const EE_FIRST: u32 = WM_USER + 0x400;
const EE_CUSTOM_BAR_OPEN: u32 = EE_FIRST + 73;

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
                                
                                let output = String::from_utf8_lossy(&buffer[..bytes_read as usize]);
                                // log::debug!("ConPTY Output: {}", output);
                                
                                {
                                    let mut data = data_arc.lock().unwrap();
                                    data.buffer.write_string(&output);
                                }
                                
                                // Trigger repaint
                                let _ = InvalidateRect(HWND(hwnd_client_ptr as *mut _), None, BOOL(0));
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
        WM_CHAR => {
            let char_code = wparam.0 as u16;
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
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
