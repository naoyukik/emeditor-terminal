use crate::gui::driver::config_gui_driver;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};

/// EmEditor SDK からの PlugInProc メッセージを解釈し、適切な処理に振り分ける
pub(crate) fn handle_plugin_proc(
    hwnd: HWND,
    n_msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match n_msg {
        crate::EP_QUERY_PROPERTIES => {
            log::info!(
                "EP_QUERY_PROPERTIES: Plugin has properties. hwnd={:?}",
                hwnd
            );
            LRESULT(1) // TRUE
        }
        crate::EP_SET_PROPERTIES => {
            // 親ウィンドウを取得
            // 通常は wParam に親 HWND が入るが、環境によっては lParam に入るケースもある。
            let raw_parent = if w_param.0 != 0 {
                w_param.0
            } else if l_param.0 != 0 {
                l_param.0 as usize
            } else {
                hwnd.0 as usize
            };
            let parent_hwnd = HWND(raw_parent as *mut std::ffi::c_void);

            log::info!(
                "EP_SET_PROPERTIES: Request to show settings dialog. parent={:?}",
                parent_hwnd
            );

            // 設定の保存には View HWND (hwnd) が必要
            config_gui_driver::show_settings_dialog(hwnd, parent_hwnd);
            LRESULT(1) // TRUE
        }
        crate::EP_PRE_TRANSLATE_MSG => LRESULT(0),
        crate::EP_GET_INFO => LRESULT(0),
        _ => LRESULT(0),
    }
}
