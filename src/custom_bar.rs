use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM, CloseHandle, HANDLE};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, TextOutW, PAINTSTRUCT, HBRUSH, COLOR_WINDOW};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassW, LoadCursorW,
    CS_HREDRAW, CS_VREDRAW, IDC_ARROW, WM_PAINT, WNDCLASSW,
    WS_CHILD, WS_VISIBLE, WS_CLIPCHILDREN, WS_CLIPSIBLINGS,
    SendMessageW,
};
use windows::Win32::Storage::FileSystem::ReadFile;
use std::ffi::c_void;
use std::mem::size_of;
use std::thread;
use crate::conpty::ConPTY;

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
                        let output_handle_val = conpty.get_output_handle().0 as usize;
                        
                        thread::spawn(move || {
                            let output_handle = HANDLE(output_handle_val as *mut c_void);
                            let mut buffer = [0u8; 1024];
                            let mut bytes_read = 0;
                            loop {
                                unsafe {
                                    if ReadFile(
                                        output_handle,
                                        Some(&mut buffer),
                                        Some(&mut bytes_read),
                                        None
                                    ).is_err() || bytes_read == 0 {
                                        break;
                                    }
                                }
                                let output = String::from_utf8_lossy(&buffer[..bytes_read as usize]);
                                log::info!("ConPTY Output: {}", output);
                            }
                            log::info!("ConPTY output thread finished");
                            // conpty is dropped here
                            let _ = conpty; 
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
                
                let text = w!("Hello ConPTY");
                let _ = TextOutW(hdc, 10, 10, text.as_wide());
                
                let _ = EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
