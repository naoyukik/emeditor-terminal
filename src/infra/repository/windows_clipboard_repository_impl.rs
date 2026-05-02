use crate::domain::repository::clipboard_repository::ClipboardRepository;
use windows::Win32::Foundation::HGLOBAL;
use windows::Win32::System::DataExchange::{
    CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
use windows::Win32::System::Ole::CF_UNICODETEXT;

pub struct WindowsClipboardRepositoryImpl;

impl ClipboardRepository for WindowsClipboardRepositoryImpl {
    fn get_text(&self) -> Result<String, String> {
        // SAFETY: Win32 クリップボード API の標準的な使用手順に従う。
        unsafe {
            if OpenClipboard(None).is_err() {
                return Err("Failed to open clipboard".to_string());
            }

            let result = if IsClipboardFormatAvailable(CF_UNICODETEXT.0 as u32).is_ok() {
                let h_data = GetClipboardData(CF_UNICODETEXT.0 as u32);
                if let Ok(handle) = h_data {
                    let h_global = HGLOBAL(handle.0);
                    let ptr = GlobalLock(h_global);
                    if !ptr.is_null() {
                        let mut len = 0;
                        while *(ptr as *const u16).add(len) != 0 {
                            len += 1;
                        }
                        let wide_slice = std::slice::from_raw_parts(ptr as *const u16, len);
                        let text = String::from_utf16_lossy(wide_slice);
                        let _ = GlobalUnlock(h_global);
                        Ok(text)
                    } else {
                        Err("Failed to lock clipboard memory".to_string())
                    }
                } else {
                    Err("Failed to get clipboard data".to_string())
                }
            } else {
                Ok(String::new())
            };

            let _ = CloseClipboard();
            result
        }
    }
}
