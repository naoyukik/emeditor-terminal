use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use crate::gui::driver::config_gui_driver;

/// EmEditor SDK からの PlugInProc メッセージを解釈し、適切な処理に振り分ける
pub(crate) fn handle_plugin_proc(
    hwnd: HWND,
    n_msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match n_msg {
        crate::EP_QUERY_PROPERTIES => {
            log::info!("EP_QUERY_PROPERTIES: Plugin has properties. hwnd={:?}", hwnd);
            LRESULT(1) // TRUE
        }
        crate::EP_SET_PROPERTIES => {
            log::info!("EP_SET_PROPERTIES: Request to show settings dialog. hwnd={:?}, wParam={:?}, lParam={:?}", hwnd, w_param.0, l_param.0);
            
            // 親ウィンドウを取得 (wParam がダイアログの親 HWND)
            let parent_hwnd = HWND(w_param.0 as *mut std::ffi::c_void);
            
            // 設定の保存には View HWND (hwnd) が必要
            config_gui_driver::show_settings_dialog(hwnd, parent_hwnd);
            LRESULT(1) // TRUE
        }
        crate::EP_PRE_TRANSLATE_MSG => {
            LRESULT(0)
        }
        crate::EP_GET_INFO => {
            LRESULT(0)
        }
        _ => {
            LRESULT(0)
        }
    }
}
