/// ウィンドウを識別するための ID (Win32 HWND の値を保持するスレッドセーフなラッパー)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WindowId(pub isize);

impl WindowId {
    /// 0 を示すデフォルトの ID
    #[allow(dead_code)]
    pub fn new(hwnd: isize) -> Self {
        Self(hwnd)
    }
}
