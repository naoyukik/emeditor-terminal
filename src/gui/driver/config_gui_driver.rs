use crate::domain::model::terminal_config_value::TerminalConfig;
use crate::domain::repository::configuration_repository::ConfigurationRepository;
use crate::get_instance_handle;
use crate::gui::resolver::terminal_window_resolver::SendHWND;
use crate::infra::repository::emeditor_config_repository_impl::EmEditorConfigRepositoryImpl;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{LOGFONTW, GetDC, ReleaseDC, GetDeviceCaps, LOGPIXELSY};
use windows::Win32::UI::Controls::Dialogs::{
    ChooseFontW, CF_INITTOLOGFONTSTRUCT, CF_SCREENFONTS, CHOOSEFONTW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DialogBoxParamW, EndDialog, SetDlgItemTextW, IDCANCEL, IDOK,
};

// リソース ID (emeditor-terminal.rc と合わせる)
const IDD_SET_PROPERTIES: u16 = 101;
const IDC_STATIC_FONT_NAME: i32 = 1001;
const IDC_BTN_CHANGE_FONT: i32 = 1002;

static IS_DIALOG_ACTIVE: AtomicBool = AtomicBool::new(false);

// ダイアログ表示中のテンポラリな設定を保持するための Mutex
static TEMP_CONFIG: Mutex<Option<TerminalConfig>> = Mutex::new(None);
// EmEditor の View HWND を保持 (設定の保存に必要)
static VIEW_HWND: Mutex<Option<SendHWND>> = Mutex::new(None);

/// ピクセル単位の高さからポイントサイズへ変換する
unsafe fn pixels_to_points(hwnd: HWND, lf_height: i32) -> i32 {
    let hdc = GetDC(hwnd);
    if hdc.is_invalid() { return 10; }
    let dpi_y = GetDeviceCaps(hdc, LOGPIXELSY);
    ReleaseDC(hwnd, hdc);
    
    if dpi_y == 0 { return 10; }
    (lf_height.abs() * 72 + dpi_y / 2) / dpi_y
}

/// ポイントサイズからピクセル単位の高さへ変換する (LOGFONT用)
unsafe fn points_to_pixels(hwnd: HWND, points: i32) -> i32 {
    let hdc = GetDC(hwnd);
    if hdc.is_invalid() { return -13; }
    let dpi_y = GetDeviceCaps(hdc, LOGPIXELSY);
    ReleaseDC(hwnd, hdc);
    
    if dpi_y == 0 { return -13; }
    -(points * dpi_y / 72)
}

/// 設定ダイアログを表示する
pub(crate) fn show_settings_dialog(view_hwnd: HWND, parent_hwnd: HWND) {
    if IS_DIALOG_ACTIVE.swap(true, Ordering::SeqCst) {
        return;
    }

    let instance = get_instance_handle();
    
    if let Ok(mut lock) = VIEW_HWND.lock() {
        *lock = Some(SendHWND(view_hwnd));
    }

    unsafe {
        let _result = DialogBoxParamW(
            instance,
            windows::core::PCWSTR(IDD_SET_PROPERTIES as usize as *const u16),
            if parent_hwnd.0.is_null() { view_hwnd } else { parent_hwnd },
            Some(settings_dlg_proc),
            LPARAM(0),
        );
    }

    if let Ok(mut lock) = TEMP_CONFIG.lock() { *lock = None; }
    if let Ok(mut lock) = VIEW_HWND.lock() { *lock = None; }
    IS_DIALOG_ACTIVE.store(false, Ordering::SeqCst);
}

/// ダイアログ内のフォント表示を更新する
unsafe fn update_font_label(hwnd: HWND, font_face: &str, font_size: i32) {
    let font_display = format!("Current Font: {}, {}pt", font_face, font_size);
    let wide_text: Vec<u16> = font_display
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
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
    match msg {
        windows::Win32::UI::WindowsAndMessaging::WM_INITDIALOG => {
            let view_hwnd = VIEW_HWND.lock().ok().and_then(|lock| lock.clone());

            if let Some(h) = view_hwnd {
                let repo = EmEditorConfigRepositoryImpl::new(h);
                let config = repo.get_terminal_config();

                if let Ok(mut lock) = TEMP_CONFIG.lock() {
                    *lock = Some(config.clone());
                }

                update_font_label(hwnd, &config.font_face, config.font_size);
            }
            1 // TRUE
        }
        windows::Win32::UI::WindowsAndMessaging::WM_COMMAND => {
            let control_id = (w_param.0 & 0xFFFF) as i32;
            match control_id {
                id if id == IDOK.0 as i32 => {
                    let config_to_save = TEMP_CONFIG.lock().ok().and_then(|lock| lock.clone());
                    let view_hwnd = VIEW_HWND.lock().ok().and_then(|lock| lock.clone());

                    if let (Some(config), Some(h_view)) = (config_to_save, view_hwnd) {
                        let repo = EmEditorConfigRepositoryImpl::new(h_view);
                        repo.save(&config);
                    }

                    EndDialog(hwnd, IDOK.0 as isize).expect("EndDialog failed");
                    1
                }
                id if id == IDCANCEL.0 as i32 => {
                    EndDialog(hwnd, IDCANCEL.0 as isize).expect("EndDialog failed");
                    1
                }
                IDC_BTN_CHANGE_FONT => {
                    let mut lf = LOGFONTW::default();
                    
                    if let Ok(lock) = TEMP_CONFIG.lock() {
                        if let Some(config) = &*lock {
                            let face_name_units: Vec<u16> = config.font_face.encode_utf16().collect();
                            let len = face_name_units.len().min(lf.lfFaceName.len() - 1);
                            lf.lfFaceName[..len].copy_from_slice(&face_name_units[..len]);
                            lf.lfHeight = points_to_pixels(hwnd, config.font_size);
                        }
                    }

                    let mut cf = CHOOSEFONTW::default();
                    cf.lStructSize = std::mem::size_of::<CHOOSEFONTW>() as u32;
                    cf.hwndOwner = hwnd;
                    cf.lpLogFont = &mut lf;
                    cf.Flags = CF_SCREENFONTS | CF_INITTOLOGFONTSTRUCT;

                    if ChooseFontW(&mut cf).as_bool() {
                        let selected_face = String::from_utf16_lossy(&lf.lfFaceName);
                        let selected_face = selected_face.trim_matches('\0').to_string();
                        let selected_size = pixels_to_points(hwnd, lf.lfHeight);
                        
                        if let Ok(mut lock) = TEMP_CONFIG.lock() {
                            if let Some(config) = lock.as_mut() {
                                config.font_face = selected_face.clone();
                                config.font_size = selected_size;
                            }
                        }
                        update_font_label(hwnd, &selected_face, selected_size);
                    }
                    1
                }
                _ => 0,
            }
        }
        _ => 0,
    }
}
