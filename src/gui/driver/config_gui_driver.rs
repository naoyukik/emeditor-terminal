use crate::domain::model::terminal_config_value::{TerminalConfig, ThemeType};
use crate::get_instance_handle;
use crate::gui::common::{pixels_to_points, points_to_pixels};
use crate::gui::driver::resource::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::LOGFONTW;
use windows::Win32::UI::Controls::Dialogs::{
    ChooseFontW, CF_INITTOLOGFONTSTRUCT, CF_SCREENFONTS, CHOOSEFONTW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DialogBoxParamW, EndDialog, SendMessageW, SetDlgItemTextW, CB_ADDSTRING, CB_GETCURSEL,
    CB_SETCURSEL, IDCANCEL, IDOK,
};

static IS_DIALOG_ACTIVE: AtomicBool = AtomicBool::new(false);

// ダイアログ表示中のテンポラリな設定を保持するための Mutex
static TEMP_CONFIG: Mutex<Option<TerminalConfig>> = Mutex::new(None);

/// 設定ダイアログを表示する
///
/// ユーザーが OK をクリックした場合は更新後の TerminalConfig を返し、
/// キャンセルした場合は None を返す。
pub(crate) fn show_settings_dialog(
    view_hwnd: HWND,
    parent_hwnd: HWND,
    initial_config: TerminalConfig,
) -> Option<TerminalConfig> {
    if IS_DIALOG_ACTIVE.swap(true, Ordering::SeqCst) {
        return None;
    }

    let instance = get_instance_handle();

    // 初期設定をセット
    if let Ok(mut lock) = TEMP_CONFIG.lock() {
        *lock = Some(initial_config);
    }

    let mut result_config = None;

    // SAFETY: ダイアログの表示。インスタンスハンドル、リソースID、親ハンドル、
    // およびプロシージャが正しく設定されており、モーダル実行は安全。
    unsafe {
        // モーダルダイアログの表示
        let result = DialogBoxParamW(
            Some(instance),
            windows::core::PCWSTR(IDD_SET_PROPERTIES as usize as *const u16),
            if parent_hwnd.0.is_null() {
                Some(view_hwnd)
            } else {
                Some(parent_hwnd)
            },
            Some(settings_dlg_proc),
            LPARAM(0),
        );

        if result == IDOK.0 as isize {
            if let Ok(lock) = TEMP_CONFIG.lock() {
                result_config = lock.clone();
            }
        } else if result == -1 {
            log::error!(
                "DialogBoxParamW failed. GetLastError={:?}",
                windows::Win32::Foundation::GetLastError()
            );
        }
    }

    if let Ok(mut lock) = TEMP_CONFIG.lock() {
        *lock = None;
    }
    IS_DIALOG_ACTIVE.store(false, Ordering::SeqCst);

    result_config
}

/// ダイアログ内のフォント表示を更新する
unsafe fn update_font_label(hwnd: HWND, config: &TerminalConfig) {
    let mut style_parts = Vec::new();
    if config.font_weight >= 700 {
        style_parts.push("Bold");
    }
    if config.font_italic {
        style_parts.push("Italic");
    }
    let style_str = if style_parts.is_empty() {
        "".to_string()
    } else {
        format!(" ({})", style_parts.join("/"))
    };

    let font_display = format!(
        "Current Font: {}, {}pt{}",
        config.font_face, config.font_size, style_str
    );
    let wide_text: Vec<u16> = font_display
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    
    // SAFETY: 有効な HWND とコントロール ID を使用してテキストを設定する。
    let _ = SetDlgItemTextW(
        hwnd,
        IDC_STATIC_FONT_NAME,
        windows::core::PCWSTR(wide_text.as_ptr()),
    );
}

/// ダイアログプロシージャ
unsafe extern "system" fn settings_dlg_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    _l_param: LPARAM,
) -> isize {
    // SAFETY: ダイアログプロシージャ内での Win32 API 呼び出し。
    // メッセージ、パラメータ、およびハンドルの整合性は Windows 側によって担保される。
    match msg {
        windows::Win32::UI::WindowsAndMessaging::WM_INITDIALOG => {
            log::info!("WM_INITDIALOG: Initializing settings dialog.");

            // テーマコンボボックスの初期化
            if let Ok(combo_hwnd) =
                windows::Win32::UI::WindowsAndMessaging::GetDlgItem(Some(hwnd), IDC_COMBO_THEME)
            {
                if !combo_hwnd.0.is_null() {
                    for theme in ThemeType::all() {
                        let display_name = theme.get_display_name();
                        let wide_name: Vec<u16> = display_name
                            .encode_utf16()
                            .chain(std::iter::once(0))
                            .collect();
                        let item_idx = SendMessageW(
                            combo_hwnd,
                            CB_ADDSTRING,
                            Some(WPARAM(0)),
                            Some(LPARAM(wide_name.as_ptr() as isize)),
                        );
                        // インデックス値を明示的に紐付ける
                        if item_idx.0 >= 0 {
                            SendMessageW(
                                combo_hwnd,
                                windows::Win32::UI::WindowsAndMessaging::CB_SETITEMDATA,
                                Some(WPARAM(item_idx.0 as usize)),
                                Some(LPARAM(theme.to_index() as isize)),
                            );
                        }
                    }

                    if let Ok(lock) = TEMP_CONFIG.lock() {
                        if let Some(config) = lock.as_ref() {
                            let target_val = config.theme_type.to_index();
                            let count = SendMessageW(
                                combo_hwnd,
                                windows::Win32::UI::WindowsAndMessaging::CB_GETCOUNT,
                                Some(WPARAM(0)),
                                Some(LPARAM(0)),
                            )
                            .0 as i32;

                            for i in 0..count {
                                let item_data = SendMessageW(
                                    combo_hwnd,
                                    windows::Win32::UI::WindowsAndMessaging::CB_GETITEMDATA,
                                    Some(WPARAM(i as usize)),
                                    Some(LPARAM(0)),
                                )
                                .0 as i32;
                                if item_data == target_val {
                                    SendMessageW(
                                        combo_hwnd,
                                        CB_SETCURSEL,
                                        Some(WPARAM(i as usize)),
                                        Some(LPARAM(0)),
                                    );
                                    break;
                                }
                            }
                            update_font_label(hwnd, config);
                        }
                    }
                }
            }
            1 // TRUE
        }
        windows::Win32::UI::WindowsAndMessaging::WM_COMMAND => {
            let control_id = (w_param.0 & 0xFFFF) as i32;
            match control_id {
                id if id == IDOK.0 => {
                    log::info!("Settings dialog: OK clicked.");

                    // コンボボックスからテーマを取得
                    if let Ok(combo_hwnd) = windows::Win32::UI::WindowsAndMessaging::GetDlgItem(
                        Some(hwnd),
                        IDC_COMBO_THEME,
                    ) {
                        if !combo_hwnd.0.is_null() {
                            let sel_idx = SendMessageW(
                                combo_hwnd,
                                CB_GETCURSEL,
                                Some(WPARAM(0)),
                                Some(LPARAM(0)),
                            )
                            .0;
                            if sel_idx != windows::Win32::UI::WindowsAndMessaging::CB_ERR as isize {
                                let theme_val = SendMessageW(
                                    combo_hwnd,
                                    windows::Win32::UI::WindowsAndMessaging::CB_GETITEMDATA,
                                    Some(WPARAM(sel_idx as usize)),
                                    Some(LPARAM(0)),
                                )
                                .0 as i32;

                                if let Ok(mut lock) = TEMP_CONFIG.lock() {
                                    if let Some(config) = lock.as_mut() {
                                        config.theme_type = ThemeType::from_index(theme_val);
                                    }
                                }
                            }
                        }
                    }

                    if let Err(e) = EndDialog(hwnd, IDOK.0 as isize) {
                        log::error!("EndDialog(IDOK) failed: {:?}", e);
                    }
                    1
                }
                id if id == IDCANCEL.0 => {
                    if let Err(e) = EndDialog(hwnd, IDCANCEL.0 as isize) {
                        log::error!("EndDialog(IDCANCEL) failed: {:?}", e);
                    }
                    1
                }
                IDC_BTN_CHANGE_FONT => {
                    let mut lf = LOGFONTW::default();

                    if let Ok(lock) = TEMP_CONFIG.lock() {
                        if let Some(config) = lock.as_ref() {
                            let face_name_units: Vec<u16> =
                                config.font_face.encode_utf16().collect();
                            let len = face_name_units.len().min(lf.lfFaceName.len() - 1);
                            lf.lfFaceName[..len].copy_from_slice(&face_name_units[..len]);
                            lf.lfHeight = points_to_pixels(hwnd, config.font_size);
                            lf.lfWeight = config.font_weight;
                            lf.lfItalic = if config.font_italic { 1 } else { 0 };
                        }
                    }

                    let mut cf = CHOOSEFONTW {
                        lStructSize: std::mem::size_of::<CHOOSEFONTW>() as u32,
                        hwndOwner: hwnd,
                        lpLogFont: &mut lf,
                        Flags: CF_SCREENFONTS | CF_INITTOLOGFONTSTRUCT,
                        ..Default::default()
                    };

                    if ChooseFontW(&mut cf).as_bool() {
                        let len = lf
                            .lfFaceName
                            .iter()
                            .position(|&c| c == 0)
                            .unwrap_or(lf.lfFaceName.len());
                        let selected_face = String::from_utf16_lossy(&lf.lfFaceName[..len]);
                        let selected_size = pixels_to_points(hwnd, lf.lfHeight);
                        let selected_weight = lf.lfWeight;
                        let selected_italic = lf.lfItalic != 0;

                        if let Ok(mut lock) = TEMP_CONFIG.lock() {
                            if let Some(config) = lock.as_mut() {
                                config.font_face = selected_face;
                                config.font_size = selected_size;
                                config.font_weight = selected_weight;
                                config.font_italic = selected_italic;

                                update_font_label(hwnd, config);
                            }
                        }
                    }
                    1
                }
                _ => 0,
            }
        }
        _ => 0,
    }
}
