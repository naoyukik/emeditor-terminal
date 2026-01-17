use std::ffi::c_void;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{BOOL, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    DialogBoxIndirectParamW, EndDialog, GetDlgItem, GetWindowTextW,
    WM_INITDIALOG, WM_COMMAND, WM_CLOSE, BN_CLICKED, IDOK, IDCANCEL,
    DLGTEMPLATE, DLGITEMTEMPLATE, 
    DS_CENTER, DS_MODALFRAME, DS_SETFONT, 
    WS_CAPTION, WS_POPUP, WS_SYSMENU, WS_CHILD, WS_VISIBLE, WS_TABSTOP, WS_BORDER, 
    ES_AUTOHSCROLL, BS_DEFPUSHBUTTON,
    WS_EX_DLGMODALFRAME,
    GWLP_USERDATA, WINDOW_LONG_PTR_INDEX, SetWindowLongPtrW, GetWindowLongPtrW,
};
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;

// ID definitions for controls
const ID_EDIT: i32 = 1001;

// Helper to align data to DWORD boundaries
fn align_vec(vec: &mut Vec<u8>) {
    let len = vec.len();
    let padding = (4 - (len % 4)) % 4;
    for _ in 0..padding {
        vec.push(0);
    }
}

fn append_str(vec: &mut Vec<u8>, s: &str) {
    for c in s.encode_utf16() {
        vec.extend_from_slice(&c.to_le_bytes());
    }
    vec.push(0); // null terminator (u16)
    vec.push(0);
}

pub fn show_input_dialog(hwnd_parent: HWND) -> Option<String> {
    log::info!("show_input_dialog called");
    // Construct Dialog Template in memory
    let mut template = Vec::new();

    // ... (Template construction logic is same, omitted logging for brevity) ...
    // DLGTEMPLATE
    let style = (DS_CENTER as u32) | (DS_MODALFRAME as u32) | WS_CAPTION.0 | WS_SYSMENU.0 | WS_POPUP.0 | WS_VISIBLE.0 | (DS_SETFONT as u32);
    template.extend_from_slice(&style.to_le_bytes());
    template.extend_from_slice(&WS_EX_DLGMODALFRAME.0.to_le_bytes());
    template.extend_from_slice(&(3u16).to_le_bytes());
    template.extend_from_slice(&(0i16).to_le_bytes());
    template.extend_from_slice(&(0i16).to_le_bytes());
    template.extend_from_slice(&(200i16).to_le_bytes());
    template.extend_from_slice(&(70i16).to_le_bytes());
    template.extend_from_slice(&(0u16).to_le_bytes());
    template.extend_from_slice(&(0u16).to_le_bytes());
    append_str(&mut template, "Send Command");
    template.extend_from_slice(&(9u16).to_le_bytes());
    append_str(&mut template, "Segoe UI");

    align_vec(&mut template);
    let edit_style = WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0 | WS_BORDER.0 | (ES_AUTOHSCROLL as u32);
    template.extend_from_slice(&edit_style.to_le_bytes());
    template.extend_from_slice(&(0u32).to_le_bytes());
    template.extend_from_slice(&(10i16).to_le_bytes());
    template.extend_from_slice(&(10i16).to_le_bytes());
    template.extend_from_slice(&(180i16).to_le_bytes());
    template.extend_from_slice(&(14i16).to_le_bytes());
    template.extend_from_slice(&(ID_EDIT as u16).to_le_bytes());
    template.extend_from_slice(&(0xFFFFu16).to_le_bytes());
    template.extend_from_slice(&(0x0081u16).to_le_bytes());
    template.extend_from_slice(&(0u16).to_le_bytes());
    template.extend_from_slice(&(0u16).to_le_bytes());

    align_vec(&mut template);
    let btn_style = WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0 | (BS_DEFPUSHBUTTON as u32);
    template.extend_from_slice(&btn_style.to_le_bytes());
    template.extend_from_slice(&(0u32).to_le_bytes());
    template.extend_from_slice(&(35i16).to_le_bytes());
    template.extend_from_slice(&(40i16).to_le_bytes());
    template.extend_from_slice(&(50i16).to_le_bytes());
    template.extend_from_slice(&(14i16).to_le_bytes());
    template.extend_from_slice(&(IDOK.0 as u16).to_le_bytes());
    template.extend_from_slice(&(0xFFFFu16).to_le_bytes());
    template.extend_from_slice(&(0x0080u16).to_le_bytes());
    append_str(&mut template, "Send");
    template.extend_from_slice(&(0u16).to_le_bytes());

    align_vec(&mut template);
    let btn_style = WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0;
    template.extend_from_slice(&btn_style.to_le_bytes());
    template.extend_from_slice(&(0u32).to_le_bytes());
    template.extend_from_slice(&(115i16).to_le_bytes());
    template.extend_from_slice(&(40i16).to_le_bytes());
    template.extend_from_slice(&(50i16).to_le_bytes());
    template.extend_from_slice(&(14i16).to_le_bytes());
    template.extend_from_slice(&(IDCANCEL.0 as u16).to_le_bytes());
    template.extend_from_slice(&(0xFFFFu16).to_le_bytes());
    template.extend_from_slice(&(0x0080u16).to_le_bytes());
    append_str(&mut template, "Cancel");
    template.extend_from_slice(&(0u16).to_le_bytes());

    // Show Dialog
    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap();
        let mut result_string: Option<String> = None;
        let result_ptr = &mut result_string as *mut Option<String>;
        
        log::info!("Calling DialogBoxIndirectParamW");
        let result = DialogBoxIndirectParamW(
            hinstance,
            template.as_ptr() as *const DLGTEMPLATE,
            hwnd_parent,
            Some(dlg_proc),
            LPARAM(result_ptr as isize),
        );
        log::info!("DialogBoxIndirectParamW returned: {:?}", result);
        
        result_string
    }
}

extern "system" fn dlg_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> isize {
    unsafe {
        match msg {
            WM_INITDIALOG => {
                log::info!("DlgProc: WM_INITDIALOG");
                let result_ptr = lparam.0 as *mut Option<String>;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, result_ptr as isize);
                
                let _ = SetFocus(GetDlgItem(hwnd, ID_EDIT).unwrap()); 
                return 0; 
            }
            WM_COMMAND => {
                let id = wparam.0 as i32 & 0xFFFF;
                if id == IDOK.0 {
                    log::info!("DlgProc: IDOK clicked");
                    let mut buffer = [0u16; 1024];
                    if let Ok(h_edit) = GetDlgItem(hwnd, ID_EDIT) {
                         let len = GetWindowTextW(h_edit, &mut buffer);
                         log::info!("GetWindowTextW len: {}", len);
                         if len > 0 {
                             let text = String::from_utf16_lossy(&buffer[..len as usize]);
                             log::info!("Captured text: {}", text);
                             
                             let result_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Option<String>;
                             if !result_ptr.is_null() {
                                 log::info!("Writing to result_ptr");
                                 *result_ptr = Some(text);
                             } else {
                                 log::error!("result_ptr is null!");
                             }
                         }
                    } else {
                        log::error!("Failed to get Edit control item");
                    }
                    let _ = EndDialog(hwnd, 1);
                    return 1;
                } else if id == IDCANCEL.0 {
                    log::info!("DlgProc: IDCANCEL clicked");
                    let _ = EndDialog(hwnd, 0);
                    return 1;
                }
            }
            WM_CLOSE => {
                log::info!("DlgProc: WM_CLOSE");
                let _ = EndDialog(hwnd, 0);
                return 1;
            }
            _ => {}
        }
    }
    0
}
