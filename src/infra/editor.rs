use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::SendMessageW;

#[allow(dead_code)]
const WM_USER: u32 = 0x0400;
#[allow(dead_code)]
const EE_FIRST: u32 = WM_USER + 0x400;
#[allow(dead_code)]
const EE_OUTPUT_STRING: u32 = EE_FIRST + 90;
#[allow(dead_code)]
const EE_EXEC_COMMAND: u32 = EE_FIRST + 22;
#[allow(dead_code)]
const EEID_VIEW_OUTPUT: u32 = 4420;

#[allow(dead_code)]
pub fn output_string(hwnd: HWND, text: &str) {
    // Convert Rust String (UTF-8) to Windows Wide String (UTF-16)
    let wide_text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let _ = SendMessageW(
            hwnd,
            EE_OUTPUT_STRING,
            WPARAM(0),
            LPARAM(wide_text.as_ptr() as isize),
        );
    }
}

#[allow(dead_code)]
pub fn show_output_bar(hwnd: HWND) {
    unsafe {
        let _ = SendMessageW(
            hwnd,
            EE_EXEC_COMMAND,
            WPARAM(EEID_VIEW_OUTPUT as usize),
            LPARAM(0),
        );
    }
}
