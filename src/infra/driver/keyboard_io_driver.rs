use crate::domain::model::input::{InputKey, Modifiers};
use crate::domain::model::window_id_value::WindowId;
use crate::domain::repository::key_translator_repository::KeyTranslatorRepository;
use crate::domain::service::vt_sequence_translator_domain_service::VtSequenceTranslatorDomainService;
use std::cell::RefCell;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL, VK_MENU, VK_SHIFT};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, PostMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, WH_KEYBOARD,
};

/// 描画更新を通知するメッセージ
const WM_APP_REPAINT: u32 = 0x8001;

thread_local! {
    static KEYBOARD_HOOK: RefCell<Option<HHOOK>> = const { RefCell::new(None) };
    static TARGET_HWND: RefCell<Option<HWND>> = const { RefCell::new(None) };
    static HOOK_INSTANCE: RefCell<Option<KeyboardIoDriver >> = const { RefCell::new(None) };
}

/// Windowsの低レベルキーボードフックを管理する構造体
pub struct KeyboardIoDriver {
    target_hwnd: HWND,
}

impl KeyboardIoDriver {
    pub fn new(target_window_id: WindowId) -> Self {
        Self {
            target_hwnd: HWND(target_window_id.0 as _),
        }
    }

    pub fn install_global(window_id: WindowId) {
        HOOK_INSTANCE.with(|instance| {
            let mut instance_ref = instance.borrow_mut();
            if instance_ref.is_none() {
                let hook = KeyboardIoDriver::new(window_id);
                hook.install();
                *instance_ref = Some(hook);
                log::info!("Global keyboard hook installed");
            }
        });
    }

    pub fn uninstall_global() {
        HOOK_INSTANCE.with(|instance| {
            let mut instance_ref = instance.borrow_mut();
            if let Some(hook) = instance_ref.take() {
                hook.uninstall();
                log::info!("Global keyboard hook uninstalled");
            }
        });
    }

    pub fn install(&self) {
        KEYBOARD_HOOK.with(|hook| {
            let mut hook_ref = hook.borrow_mut();
            if hook_ref.is_none() {
                // SAFETY: 自スレッドに対するキーボードフックのインストール。
                // 成功した場合は HHOOK を保持し、Drop 時または明示的にアンインストールされる。
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
                            TARGET_HWND.with(|h| {
                                *h.borrow_mut() = Some(self.target_hwnd);
                            });
                        }
                        Err(e) => {
                            log::error!("Failed to install keyboard hook (Infra): {}", e);
                        }
                    }
                }
            }
        });
    }

    pub fn uninstall(&self) {
        KEYBOARD_HOOK.with(|hook| {
            let mut hook_ref = hook.borrow_mut();
            if let Some(hhook) = hook_ref.take() {
                // SAFETY: 保持していた有効な HHOOK を解除する。
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

impl Drop for KeyboardIoDriver {
    fn drop(&mut self) {
        self.uninstall();
    }
}

extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let vk_code = wparam.0 as u16;
        let key_up = (lparam.0 >> 31) & 1;

        if key_up == 0
            && let Some(hwnd) = TARGET_HWND.with(|h| *h.borrow())
            && !crate::gui::window::is_ime_composing(hwnd)
        {
            // SAFETY: キーの状態（Ctrl/Shift/Alt）を同期的に取得する。
            let is_ctrl_pressed = unsafe { GetKeyState(VK_CONTROL.0 as i32) } < 0;
            let is_shift_pressed = unsafe { GetKeyState(VK_SHIFT.0 as i32) } < 0;
            let is_alt_pressed = unsafe { GetKeyState(VK_MENU.0 as i32) } < 0;

            if !crate::gui::driver::window_gui_driver::WindowGuiDriver::is_system_shortcut(
                vk_code,
                is_alt_pressed,
            ) {
                let translator = VtSequenceTranslatorDomainService::new();
                let input_key = InputKey::new(
                    vk_code,
                    Modifiers {
                        is_ctrl_pressed,
                        is_shift_pressed,
                        is_alt_pressed,
                    },
                );

                if let Some(seq) = translator.translate(input_key) {
                    let data_arc =
                        crate::gui::resolver::terminal_window_resolver::get_terminal_data();
                    let mut window_data = data_arc.lock().unwrap();
                    window_data.service.reset_viewport();
                    let _ = window_data.service.send_input(&seq);
                    drop(window_data);

                    // SAFETY: 有効なウィンドウハンドルに対して描画更新を通知する。
                    // PostMessageW はスレッドセーフである。
                    unsafe {
                        let _ = PostMessageW(Some(hwnd), WM_APP_REPAINT, WPARAM(0), LPARAM(0));
                    }
                    return LRESULT(1);
                }
            }
        }
    }

    // SAFETY: フックチェーンの次のプロシージャを呼び出す。
    KEYBOARD_HOOK.with(|hook| {
        let hook_ref = hook.borrow();
        if let Some(hhook) = *hook_ref {
            unsafe { CallNextHookEx(Some(hhook), code, wparam, lparam) }
        } else {
            unsafe { CallNextHookEx(None, code, wparam, lparam) }
        }
    })
}
