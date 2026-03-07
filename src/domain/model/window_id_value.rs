/// ウィンドウを識別するための ID (Win32 HWND の値を保持するスレッドセーフなラッパー)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowId(pub isize);

// SAFETY:
// - WindowId は Win32 HWND の値を整数として保持する。
// - ハンドルの利用（API 呼び出し）は、この ID を HWND に戻す際に
//   適切なレイヤー（Driver 等）で安全性を担保して行うこと。
unsafe impl Send for WindowId {}
unsafe impl Sync for WindowId {}

impl WindowId {
    /// 0 を示すデフォルトの ID
    pub fn default() -> Self {
        Self(0)
    }
}
