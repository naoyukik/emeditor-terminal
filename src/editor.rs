use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::SendMessageW;

const WM_USER: u32 = 0x0400;
const EE_FIRST: u32 = WM_USER + 0x400;
const EE_OUTPUT_STRING: u32 = EE_FIRST + 90;

pub fn output_string(hwnd: HWND, text: &str) {
    log::info!("Sending to EmEditor (len={}): {}", text.len(), text.trim());
    
    // Convert Rust String (UTF-8) to Windows Wide String (UTF-16)
    let wide_text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let result = SendMessageW(
            hwnd,
            EE_OUTPUT_STRING,
            WPARAM(0),
            LPARAM(wide_text.as_ptr() as isize),
        );
        log::info!("SendMessageW result: {:?}", result);
    }
}