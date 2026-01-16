use log::LevelFilter;
use simplelog::{Config, WriteLogger};
use std::ffi::c_void;
use std::fs::File;
use std::path::PathBuf;
use windows::core::w;
use windows::Win32::Foundation::{BOOL, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK};

// EmEditor SDK Constants
pub const EVENT_CREATE: u32 = 0x00000400;
#[allow(dead_code)]
pub const EVENT_CLOSE: u32 = 0x00000800;

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
    unsafe {
        MessageBoxW(
            hwnd,
            w!("Hello from Rust! (OnCommand)"),
            w!("EmEditor Plugin"),
            MB_OK,
        );
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables, clippy::not_unsafe_ptr_arg_deref)]
pub extern "system" fn QueryStatus(hwnd: HWND, pbChecked: *mut BOOL) -> BOOL {
    if !pbChecked.is_null() {
        unsafe {
            *pbChecked = BOOL(0);
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