use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::UpdateWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::{DestroyWindow, IsWindow};

/// Win32 ウィンドウ操作をカプセル化するドライバ
pub(crate) struct WindowGuiDriver;

impl WindowGuiDriver {
    /// 既存のウィンドウが存在し有効な場合、フォーカスを当てる。
    ///
    /// # Returns
    /// ウィンドウが存在し、フォーカスを試行した場合は `true` を返す。
    pub(crate) fn focus_existing_window(hwnd: HWND) -> bool {
        // SAFETY: HWND が有効であることを IsWindow で確認してから操作を行う。
        unsafe {
            if IsWindow(hwnd).as_bool() {
                let prev_focus = SetFocus(hwnd);
                log::debug!(
                    "focus_existing_window: SetFocus called for HWND {:?}. Previous focus was {:?}",
                    hwnd,
                    prev_focus
                );
                true
            } else {
                false
            }
        }
    }

    /// 指定されたウィンドウを安全に破棄する。
    pub(crate) fn destroy_window(hwnd: HWND) {
        // SAFETY: IsWindow で存在を確認してから DestroyWindow を呼び出す。
        unsafe {
            if IsWindow(hwnd).as_bool() {
                let _ = DestroyWindow(hwnd);
            }
        }
    }

    /// 指定されたウィンドウの更新を強制する（WM_PAINTメッセージを即座に処理させる）。
    pub(crate) fn update_window(hwnd: HWND) {
        // SAFETY: IsWindow で存在を確認してから UpdateWindow を呼び出す。
        unsafe {
            if IsWindow(hwnd).as_bool() {
                let _ = UpdateWindow(hwnd);
            }
        }
    }
}
