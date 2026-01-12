use windows::Win32::Foundation::{BOOL, HINSTANCE, HWND, WPARAM, LPARAM, LRESULT};
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK};
use windows::core::w;
use std::ffi::c_void;

// EmEditor SDK Constants
pub const EVENT_CREATE: u32 = 0x00000400;
pub const EVENT_CLOSE: u32 = 0x00000800;

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
pub extern "system" fn OnCommand(hwnd: HWND) {
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
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn QueryStatus(hwnd: HWND, pbChecked: *mut BOOL) -> BOOL {
    // Return TRUE (1) to enable the command
    BOOL(1)
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn OnEvents(hwnd: HWND, nEvent: u32, wParam: WPARAM, lParam: LPARAM) {
    if nEvent == EVENT_CREATE {
        // Plugin is loaded/initialized
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn GetMenuTextID() -> u32 {
    0 // Return 0 if no resource ID
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn GetStatusMessageID() -> u32 {
    0 // Return 0 if no resource ID
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn GetBitmapID() -> u32 {
    0 // Return 0 if no resource ID
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn PlugInProc(hwnd: HWND, nMsg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    LRESULT(0)
}