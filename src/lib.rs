use windows::Win32::Foundation::{BOOL, HINSTANCE, HWND};
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK};
use windows::core::w;
use std::ffi::c_void;

// EmEditor SDK Types (Draft)
#[allow(non_camel_case_types)]
pub type ET_COMMAND = u32;

// Placeholder for LOAD_INFO if needed. 
// Note: Exact definition of LOAD_INFO is not yet confirmed. 
// Usually passed in OnEvents(EVENT_CREATE) or similar if applicable.
#[repr(C)]
#[allow(non_snake_case)]
pub struct LOAD_INFO {
    pub cbSize: u32,
    // Add fields as discovered
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
            // Initialize
        }
        DLL_PROCESS_DETACH => {
            // Cleanup
        }
        _ => {}
    }
    BOOL(1)
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn OnCommand(hwnd: HWND) {
    unsafe {
        MessageBoxW(
            hwnd,
            w!("Hello from Rust!"),
            w!("EmEditor Plugin"),
            MB_OK,
        );
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn QueryStatus(hwnd: HWND, pbChecked: *mut BOOL) -> BOOL {
    // Return TRUE (1) to enable the command
    BOOL(1)
}