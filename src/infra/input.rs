use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, PostMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, WH_KEYBOARD,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VK_CONTROL, VK_MENU, VK_SHIFT,
};
use std::cell::RefCell;
use crate::domain::model::input::{InputKey, Modifiers};
use crate::domain::input::{KeyTranslator, VtSequenceTranslator};

/// 描画更新を通知するメッセージ
/// 0x8000 (WM_APP) + 1 は WM_APP_REPAINT として custom_bar.rs で定義されている
const WM_APP_REPAINT: u32 = 0x8001;

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
        log::debug!("Attempting to install keyboard hook (Infra)");
        KEYBOARD_HOOK.with(|hook| {
            let mut hook_ref = hook.borrow_mut();
            if hook_ref.is_none() {
                unsafe {
                    log::debug!("Calling SetWindowsHookExW...");
                    let h = SetWindowsHookExW(
                        WH_KEYBOARD,
                        Some(keyboard_hook_proc),
                        None,
                        windows::Win32::System::Threading::GetCurrentThreadId(),
                    );
                    match h {
                        Ok(hhook) => {
                            log::info!("Keyboard hook installed successfully (Infra): {:?}", hhook);
                            *hook_ref = Some(hhook);
                            // フックが成功した場合のみターゲットウィンドウを設定
                            TARGET_HWND.with(|h| {
                                *h.borrow_mut() = Some(self.target_hwnd);
                            });
                        }
                        Err(e) => {
                            log::error!("Failed to install keyboard hook (Infra): {}", e);
                        }
                    }
                }
            } else {
                log::debug!("Keyboard hook already installed (Infra)");
            }
        });
    }

    /// キーボードフックを解除する
    pub fn uninstall(&self) {
        log::debug!("Uninstalling keyboard hook (Infra)");
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
extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // log::trace!("keyboard_hook_proc: code={}, wparam={:?}", code, wparam);
    if code >= 0 {
        let vk_code = wparam.0 as u16;
        let key_up = (lparam.0 >> 31) & 1; // bit 31 = transition state (1 = key up)

        // キーダウンイベントのみを処理
        if key_up == 0 {
            if let Some(hwnd) = TARGET_HWND.with(|h| *h.borrow()) {
                // IMEの状態チェック
                if !crate::gui::custom_bar::is_ime_composing(hwnd) {
                    let ctrl_pressed = unsafe { GetKeyState(VK_CONTROL.0 as i32) } < 0;
                    let shift_pressed = unsafe { GetKeyState(VK_SHIFT.0 as i32) } < 0;
                    let alt_pressed = unsafe { GetKeyState(VK_MENU.0 as i32) } < 0;

                    // システムショートカットの除外
                    if !crate::gui::custom_bar::is_system_shortcut(vk_code, alt_pressed) {
                        let translator = VtSequenceTranslator::new();
                        let input_key = InputKey::new(
                            vk_code,
                            Modifiers {
                                ctrl: ctrl_pressed,
                                shift: shift_pressed,
                                alt: alt_pressed,
                            },
                        );

                        if let Some(seq) = translator.translate(input_key) {
                            // 直接ターミナルデータに書き込む
                            let data_arc = crate::gui::custom_bar::get_terminal_data();
                            let mut data = data_arc.lock().unwrap();
                            data.service.reset_viewport();
                            let _ = data.service.send_input(&seq);
                            drop(data);

                            // 描画更新を通知（これは安全なPostMessage）
                            unsafe { let _ = PostMessageW(hwnd, WM_APP_REPAINT, WPARAM(0), LPARAM(0)); }

                            // キーを処理したことを示す
                            return LRESULT(1);
                        }
                    }
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