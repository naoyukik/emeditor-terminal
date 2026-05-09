use crate::domain::repository::clipboard_repository::ClipboardRepository;
use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
    SetClipboardData,
};
use windows::Win32::System::Memory::{
    GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock,
};
use windows::Win32::System::Ole::CF_UNICODETEXT;

pub struct WindowsClipboardRepositoryImpl;

impl ClipboardRepository for WindowsClipboardRepositoryImpl {
    fn get_text(&self) -> Result<String, String> {
        // SAFETY: Win32 クリップボード API の標準的な使用手順に従う。
        // - OpenClipboard / CloseClipboard を確実にペアで呼び出し、他プロセスとの排他制御を管理する。
        // - GetClipboardData で取得したハンドルはシステムが所有しており、GlobalFree してはならない。
        // - GlobalLock / GlobalUnlock により、取得したグローバルメモリへのアクセスを安全に行う。
        // - バッファサイズは GlobalSize に基づき、ヌル終端を考慮して適切に制限されている。
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
                        let size_in_bytes = GlobalSize(h_global);
                        let max_len = size_in_bytes / 2;
                        let mut len = 0;
                        while len < max_len && *(ptr as *const u16).add(len) != 0 {
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

    fn set_text(&self, text: &str) -> Result<(), String> {
        let wide_text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let size = wide_text.len() * std::mem::size_of::<u16>();

        // SAFETY: Win32 クリップボード API の標準的な書き込み手順。
        // - GlobalAlloc で割り当てたメモリの所有権は、SetClipboardData が成功した時点でシステムに移転する。
        // - SetClipboardData が失敗した、あるいはそれ以前の段階でエラーとなった場合は、
        //   GlobalFree を呼び出してリソースリークを防ぐ必要がある。
        // - すべての操作は Open/CloseClipboard のペア内で行われ、EmptyClipboard により所有権を確立している。
        unsafe {
            if OpenClipboard(None).is_err() {
                return Err("Failed to open clipboard".to_string());
            }

            let _ = EmptyClipboard();

            let h_mem = GlobalAlloc(GMEM_MOVEABLE, size);
            if let Ok(handle) = h_mem {
                let ptr = GlobalLock(handle);
                if ptr.is_null() {
                    let _ = GlobalFree(Some(handle));
                    let _ = CloseClipboard();
                    return Err("Failed to lock global memory".to_string());
                }

                std::ptr::copy_nonoverlapping(wide_text.as_ptr(), ptr as *mut u16, wide_text.len());

                let _ = GlobalUnlock(handle);

                if SetClipboardData(CF_UNICODETEXT.0 as u32, Some(HANDLE(handle.0))).is_err() {
                    let _ = GlobalFree(Some(handle));
                    let _ = CloseClipboard();
                    return Err("Failed to set clipboard data".to_string());
                }
            } else {
                let _ = CloseClipboard();
                return Err("Failed to allocate global memory".to_string());
            }

            let _ = CloseClipboard();
            Ok(())
        }
    }
}
