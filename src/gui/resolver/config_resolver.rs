use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use crate::gui::driver::config_gui_driver;

/// EmEditor SDK からの PlugInProc メッセージを解釈し、適切な処理に振り分ける
pub(crate) fn handle_plugin_proc(
    _hwnd: HWND,
    n_msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match n_msg {
        crate::EP_QUERY_PROPERTIES => {
            log::info!("EP_QUERY_PROPERTIES: Plugin has properties.");
            LRESULT(1) // TRUE
        }
        crate::EP_SET_PROPERTIES => {
            log::info!("EP_SET_PROPERTIES: Request to show settings dialog.");
            let raw_hwnd = if w_param.0 != 0 { w_param.0 } else { l_param.0 as usize };
            let parent_hwnd = HWND(raw_hwnd as *mut std::ffi::c_void);
            config_gui_driver::show_settings_dialog(parent_hwnd);
            LRESULT(1) // TRUE
        }
        crate::EP_PRE_TRANSLATE_MSG => {
            // 高頻度メッセージのためログは出さない
            LRESULT(0)
        }
        crate::EP_GET_INFO => {
            // 必要に応じて情報を返すが、今は無視
            LRESULT(0)
        }
        _ => {
            // 未知のメッセージのみログを出す
            log::debug!("PlugInProc: Unknown nMsg={}, wParam={}, lParam={}", n_msg, w_param.0, l_param.0);
            LRESULT(0)
        }
    }
}
