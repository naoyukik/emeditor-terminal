use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, PostMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, WH_KEYBOARD,
};
use std::cell::RefCell;

/// フックプロシージャから送信されるカスタムメッセージ
/// 0x8000 (WM_APP) + 2 を使用
pub const WM_APP_KEYINPUT: u32 = 0x8002;

thread_local! {
    static KEYBOARD_HOOK: RefCell<Option<HHOOK>> = const { RefCell::new(None) };
    static TARGET_HWND: RefCell<Option<HWND>> = const { RefCell::new(None) };
}

/// Windowsの低レベルキーボードフックを管理する構造体
pub struct KeyboardHook {
    target_hwnd: HWND,
}

impl KeyboardHook {
    /// 新しいフック管理インスタンスを作成する
    pub fn new(target_hwnd: HWND) -> Self {
        Self { target_hwnd }
    }

    /// キーボードフックをインストールする
    pub fn install(&self) {
        TARGET_HWND.with(|h| {
            *h.borrow_mut() = Some(self.target_hwnd);
        });

        KEYBOARD_HOOK.with(|hook| {
            let mut hook_ref = hook.borrow_mut();
            if hook_ref.is_none() {
                unsafe {
                    let h = SetWindowsHookExW(
                        WH_KEYBOARD,
                        Some(keyboard_hook_proc),
                        None,
                        windows::Win32::System::Threading::GetCurrentThreadId(),
                    );
                    match h {
                        Ok(hhook) => {
                            log::info!("Keyboard hook installed successfully (Infra)");
                            *hook_ref = Some(hhook);
                        }
                        Err(e) => {
                            log::error!("Failed to install keyboard hook (Infra): {}", e);
                        }
                    }
                }
            }
        });
    }

    /// キーボードフックを解除する
    pub fn uninstall(&self) {
        KEYBOARD_HOOK.with(|hook| {
            let mut hook_ref = hook.borrow_mut();
            if let Some(hhook) = hook_ref.take() {
                unsafe {
                    let _ = UnhookWindowsHookEx(hhook);
                    log::info!("Keyboard hook uninstalled (Infra)");
                }
            }
        });
        TARGET_HWND.with(|h| {
            *h.borrow_mut() = None;
        });
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        self.uninstall();
    }
}

/// フックプロシージャ
/// 可能な限り軽量に保ち、実際の処理はターゲットウィンドウのメッセージループで行う
extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let vk_code = wparam.0 as u16;
        let key_up = (lparam.0 >> 31) & 1; // bit 31 = transition state (1 = key up)

        // キーダウンイベントのみを通知
        if key_up == 0 {
            if let Some(hwnd) = TARGET_HWND.with(|h| *h.borrow()) {
                unsafe {
                    // ターゲットウィンドウへ通知。wparamに仮想キーコードを乗せる
                    let _ = PostMessageW(hwnd, WM_APP_KEYINPUT, WPARAM(vk_code as usize), lparam);
                }
            }
        }
    }

    // 次のフックへチェーン
    KEYBOARD_HOOK.with(|hook| {
        let hook_ref = hook.borrow();
        if let Some(hhook) = *hook_ref {
            unsafe { CallNextHookEx(hhook, code, wparam, lparam) }
        } else {
            unsafe { CallNextHookEx(None, code, wparam, lparam) }
        }
    })
}
