use log::LevelFilter;
use simplelog::{ConfigBuilder, WriteLogger};
use std::ffi::c_void;
use std::fs::File;
use std::sync::OnceLock;
use windows::Win32::Foundation::{BOOL, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

mod domain;
mod gui;
mod infra;

use gui::custom_bar;

// EmEditor SDK Constants
pub const EVENT_CREATE: u32 = 0x00000400;
#[allow(dead_code)]
pub const EVENT_CLOSE: u32 = 0x00000800;

static INSTANCE_HANDLE: OnceLock<usize> = OnceLock::new();

pub fn get_instance_handle() -> HINSTANCE {
    let ptr = *INSTANCE_HANDLE.get().unwrap_or(&0) as *mut c_void;
    HINSTANCE(ptr)
}

fn init_logger() {
    let mut path = std::env::temp_dir();
    path.push("emeditor_terminal.log");

    // Set offset to +09:00:00 for JST
    let config = ConfigBuilder::new()
        .set_time_offset(time::UtcOffset::from_hms(9, 0, 0).expect("Valid JST offset"))
        .build();

    match File::create(&path) {
        Ok(file) => match WriteLogger::init(LevelFilter::Debug, config, file) {
            Ok(_) => log::info!("Logger initialized"),
            Err(e) => eprintln!("Failed to initialize logger: {}", e),
        },
        Err(e) => eprintln!("Failed to create log file '{}': {}", path.display(), e),
    }
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
            let _ = INSTANCE_HANDLE.set(dll_module.0 as usize);
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

    // Show custom bar
    if custom_bar::open_custom_bar(hwnd) {
        log::info!("Terminal bar opened and pwsh.exe started");
    } else {
        log::info!("Custom bar already open, focusing only");
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables, clippy::not_unsafe_ptr_arg_deref)]
pub extern "system" fn QueryStatus(hwnd: HWND, pbChecked: *mut BOOL) -> BOOL {
    if !pbChecked.is_null() {
        // Always return checked for now if it exists, or based on custom bar state
        // For now, let's keep it simple.
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
    } else if nEvent == EVENT_CLOSE {
        log::info!("OnEvents: EVENT_CLOSE - cleaning up plugin resources");
        custom_bar::cleanup_terminal();
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
