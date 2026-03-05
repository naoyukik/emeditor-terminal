use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{DialogBoxParamW, EndDialog, IDOK, IDCANCEL};
use crate::get_instance_handle;
use std::sync::atomic::{AtomicBool, Ordering};

// リソース ID (emeditor-terminal.rc と合わせる)
const IDD_SET_PROPERTIES: u16 = 101;

static IS_DIALOG_ACTIVE: AtomicBool = AtomicBool::new(false);

/// 設定ダイアログを表示する
pub(crate) fn show_settings_dialog(parent_hwnd: HWND) {
    if IS_DIALOG_ACTIVE.swap(true, Ordering::SeqCst) {
        // すでに表示中の場合は何もしない
        return;
    }

    let instance = get_instance_handle();

    unsafe {
        // モーダルダイアログの表示
        // MAKEINTRESOURCEW(IDD_SET_PROPERTIES) 相当
        let _result = DialogBoxParamW(
            instance,
            windows::core::PCWSTR(IDD_SET_PROPERTIES as usize as *const u16),
            parent_hwnd,
            Some(settings_dlg_proc),
            LPARAM(0),
        );
    }

    IS_DIALOG_ACTIVE.store(false, Ordering::SeqCst);
}

/// ダイアログプロシージャ
///
/// Safety: Win32 API コールバックとしての署名を維持。
unsafe extern "system" fn settings_dlg_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    _l_param: LPARAM,
) -> isize {
    match msg {
        windows::Win32::UI::WindowsAndMessaging::WM_INITDIALOG => {
            log::info!("WM_INITDIALOG: Settings dialog initialized.");
            1 // TRUE: フォーカスをデフォルトコントロールに設定
        }
        windows::Win32::UI::WindowsAndMessaging::WM_COMMAND => {
            let control_id = (w_param.0 & 0xFFFF) as i32;
            match control_id {
                id if id == IDOK.0 as i32 => {
                    log::info!("Settings dialog: OK button clicked.");
                    EndDialog(hwnd, IDOK.0 as isize).expect("EndDialog failed");
                    1
                }
                id if id == IDCANCEL.0 as i32 => {
                    log::info!("Settings dialog: Cancel button clicked.");
                    EndDialog(hwnd, IDCANCEL.0 as isize).expect("EndDialog failed");
                    1
                }
                _ => 0,
            }
        }
        _ => 0,
    }
}
