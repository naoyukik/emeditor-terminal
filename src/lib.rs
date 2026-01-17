use log::LevelFilter;
use simplelog::{Config, WriteLogger};
use std::ffi::c_void;
use std::fs::File;
use std::sync::{Arc, Mutex, OnceLock};
use windows::core::w;
use windows::Win32::Foundation::{BOOL, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK};

mod editor;
mod session;

// EmEditor SDK Constants
pub const EVENT_CREATE: u32 = 0x00000400;
#[allow(dead_code)]
pub const EVENT_CLOSE: u32 = 0x00000800;

static SESSION: OnceLock<Arc<Mutex<Option<session::ShellSession>>>> = OnceLock::new();

fn get_session() -> Arc<Mutex<Option<session::ShellSession>>> {
    SESSION.get_or_init(|| Arc::new(Mutex::new(None))).clone()
}

fn init_logger() {
    let mut path = std::env::temp_dir();
    path.push("emeditor_terminal.log");
    
    let _ = WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create(path).unwrap(),
    );
    log::info!("Logger initialized");
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: u32,
    reserved: *mut c_void,
) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            init_logger();
            log::info!("DllMain: Process Attach");
        }
        DLL_PROCESS_DETACH => {
            log::info!("DllMain: Process Detach");
        }
        _ => {}
    }
    BOOL(1)
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn OnCommand(hwnd: HWND) {
    log::info!("OnCommand called");
    
    let session_arc = get_session();
    let mut session_guard = session_arc.lock().unwrap();

    if session_guard.is_none() {
        log::info!("Starting new shell session");
        
        // Pass HWND safely to background thread
        let hwnd_ptr = hwnd.0 as usize;
        
        let result = session::ShellSession::new(move |s| {
            let hwnd = HWND(hwnd_ptr as *mut c_void);
            editor::output_string(hwnd, &s);
        });

        match result {
            Ok(s) => {
                *session_guard = Some(s);
                log::info!("Session started successfully");
                
                // Show output bar automatically
                editor::show_output_bar(hwnd);

                // Initial command to verify output
                if let Some(session) = session_guard.as_mut() {
                    let _ = session.send("echo Session Started");
                    let _ = session.send("ver");
                }
            },
            Err(e) => {
                log::error!("Failed to start session: {}", e);
                let error_msg = format!("Failed to start session: {}\0", e);
                let wide_msg: Vec<u16> = error_msg.encode_utf16().collect();
                unsafe {
                    MessageBoxW(
                        hwnd,
                        windows::core::PCWSTR(wide_msg.as_ptr()),
                        w!("EmEditor Terminal Error"),
                        MB_OK,
                    );
                }
            }
        }
    } else {
        log::info!("Session already running. Sending 'dir' command.");
        // Also show output bar when interacting with existing session
        editor::show_output_bar(hwnd);
        if let Some(session) = session_guard.as_mut() {
             let _ = session.send("dir");
        }
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables, clippy::not_unsafe_ptr_arg_deref)]
pub extern "system" fn QueryStatus(hwnd: HWND, pbChecked: *mut BOOL) -> BOOL {
    if !pbChecked.is_null() {
        unsafe {
            let session_arc = get_session();
            let is_running = session_arc.lock().unwrap().is_some();
            *pbChecked = BOOL(if is_running { 1 } else { 0 });
        }
    }
    BOOL(1)
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn OnEvents(hwnd: HWND, nEvent: u32, wParam: WPARAM, lParam: LPARAM) {
    if nEvent == EVENT_CREATE {
        log::info!("OnEvents: EVENT_CREATE");
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn GetMenuTextID() -> u32 {
    0
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn GetStatusMessageID() -> u32 {
    0
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn GetBitmapID() -> u32 {
    0
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn PlugInProc(
    hwnd: HWND,
    nMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    LRESULT(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(EVENT_CREATE, 0x00000400);
        assert_eq!(EVENT_CLOSE, 0x00000800);
    }
}