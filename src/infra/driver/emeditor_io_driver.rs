use crate::domain::model::window_id_value::WindowId;
use std::mem::size_of;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
pub use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, SendMessageW, MB_ICONERROR, MB_OK};

#[allow(dead_code)]
const WM_USER: u32 = 0x0400;
#[allow(dead_code)]
pub const EE_FIRST: u32 = WM_USER + 0x400; // 2048
#[allow(dead_code)]
pub const EE_OUTPUT_STRING: u32 = EE_FIRST + 90;
#[allow(dead_code)]
pub const EE_EXEC_COMMAND: u32 = EE_FIRST + 22;
#[allow(dead_code)]
pub const EE_CUSTOM_BAR_OPEN: u32 = EE_FIRST + 73;

const EE_REG_QUERY_VALUE: u32 = EE_FIRST + 86; // 2134
const EE_REG_SET_VALUE: u32 = EE_FIRST + 85; // 2133

const EEREG_EMEDITORPLUGIN: u32 = 0x7fffff30;

const REG_SZ: u32 = 1;
const REG_DWORD: u32 = 4;

#[repr(C)]
#[allow(non_snake_case)]
struct REG_QUERY_VALUE_INFO {
    cbSize: usize,
    dwKey: u32,
    pszConfig: PCWSTR,
    pszValue: PCWSTR,
    dwType: u32,
    lpData: *mut u8,
    lpcbData: *mut u32,
    dwFlags: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct REG_SET_VALUE_INFO {
    cbSize: usize,
    dwKey: u32,
    pszConfig: PCWSTR,
    pszValue: PCWSTR,
    dwType: u32,
    lpData: *const u8,
    cbData: u32,
    dwFlags: u32,
}

pub const CUSTOM_BAR_BOTTOM: i32 = 3;

#[repr(C)]
#[allow(non_snake_case)]
pub struct CUSTOM_BAR_INFO {
    pub cbSize: usize,
    pub hwndCustomBar: HWND,
    pub hwndClient: HWND,
    pub pszTitle: PCWSTR,
    pub iPos: i32,
}

/// EmEditor の設定から文字列を取得する
pub fn emeditor_query_string(window_id: WindowId, value_name: &str, default: &str) -> String {
    let hwnd = HWND(window_id.0 as _);
    let mut buffer: Vec<u16> = vec![0u16; 260];
    let value_name_wide: Vec<u16> = value_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let mut attempt = 0;
    const MAX_ATTEMPTS: usize = 4;

    loop {
        if attempt >= MAX_ATTEMPTS {
            return default.to_string();
        }
        attempt += 1;

        let mut cb_data = (buffer.len() * size_of::<u16>()) as u32;

        let mut info = REG_QUERY_VALUE_INFO {
            cbSize: size_of::<REG_QUERY_VALUE_INFO>(),
            dwKey: EEREG_EMEDITORPLUGIN,
            pszConfig: w!("Terminal"),
            pszValue: PCWSTR(value_name_wide.as_ptr()),
            dwType: REG_SZ,
            lpData: buffer.as_mut_ptr() as *mut u8,
            lpcbData: &mut cb_data as *mut u32,
            dwFlags: 0,
        };

        // SAFETY: HWND は有効な EmEditor ウィンドウハンドルであることを前提とし、
        // 取得されるデータサイズに合わせてバッファを管理している。
        let result = unsafe {
            SendMessageW(
                hwnd,
                EE_REG_QUERY_VALUE,
                Some(WPARAM(0)),
                Some(LPARAM(&mut info as *mut _ as isize)),
            )
        };

        if result.0 == 0 {
            let len = (cb_data as usize / size_of::<u16>()).min(buffer.len());
            let result_str = String::from_utf16_lossy(&buffer[..len]);
            let result_str = result_str.trim_matches('\0').to_string();
            return if result_str.is_empty() {
                default.to_string()
            } else {
                result_str
            };
        } else {
            let required_bytes = cb_data as usize;
            let current_bytes = buffer.len() * size_of::<u16>();

            if required_bytes > current_bytes && required_bytes > 0 {
                let required_u16 = (required_bytes + 1) / size_of::<u16>();
                let new_len = required_u16.max(buffer.len().saturating_mul(2));
                buffer = vec![0u16; new_len];
                continue;
            } else {
                return default.to_string();
            }
        }
    }
}

/// EmEditor の設定から数値 (DWORD) を取得する
pub fn emeditor_query_u32(window_id: WindowId, value_name: &str, default: u32) -> u32 {
    let hwnd = HWND(window_id.0 as _);
    let mut data: u32 = default;
    let mut cb_data = size_of::<u32>() as u32;
    let value_name_wide: Vec<u16> = value_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let mut info = REG_QUERY_VALUE_INFO {
        cbSize: size_of::<REG_QUERY_VALUE_INFO>(),
        dwKey: EEREG_EMEDITORPLUGIN,
        pszConfig: w!("Terminal"),
        pszValue: PCWSTR(value_name_wide.as_ptr()),
        dwType: REG_DWORD,
        lpData: &mut data as *mut u32 as *mut u8,
        lpcbData: &mut cb_data as *mut u32,
        dwFlags: 0,
    };

    // SAFETY: メッセージ経由での設定取得は、指定したメモリ領域への書き込みのみを行う。
    let result = unsafe {
        SendMessageW(
            hwnd,
            EE_REG_QUERY_VALUE,
            Some(WPARAM(0)),
            Some(LPARAM(&mut info as *mut _ as isize)),
        )
    };

    if result.0 == 0 {
        data
    } else {
        default
    }
}

/// EmEditor の設定に文字列を保存する
pub fn emeditor_set_string(window_id: WindowId, value_name: &str, value: &str) -> i32 {
    let hwnd = HWND(window_id.0 as _);
    let value_wide: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
    let value_name_wide: Vec<u16> = value_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let info = REG_SET_VALUE_INFO {
        cbSize: size_of::<REG_SET_VALUE_INFO>(),
        dwKey: EEREG_EMEDITORPLUGIN,
        pszConfig: w!("Terminal"),
        pszValue: PCWSTR(value_name_wide.as_ptr()),
        dwType: REG_SZ,
        lpData: value_wide.as_ptr() as *const u8,
        cbData: (value_wide.len() * size_of::<u16>()) as u32,
        dwFlags: 0,
    };

    // SAFETY: メッセージ経由での設定保存は、読み取り専用のバッファポインタを渡すため安全。
    unsafe {
        SendMessageW(
            hwnd,
            EE_REG_SET_VALUE,
            Some(WPARAM(0)),
            Some(LPARAM(&info as *const _ as isize)),
        )
        .0 as i32
    }
}

/// EmEditor の設定に数値 (DWORD) を保存する
pub fn emeditor_set_u32(window_id: WindowId, value_name: &str, value: u32) -> i32 {
    let hwnd = HWND(window_id.0 as _);
    let value_name_wide: Vec<u16> = value_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let info = REG_SET_VALUE_INFO {
        cbSize: size_of::<REG_SET_VALUE_INFO>(),
        dwKey: EEREG_EMEDITORPLUGIN,
        pszConfig: w!("Terminal"),
        pszValue: PCWSTR(value_name_wide.as_ptr()),
        dwType: REG_DWORD,
        lpData: &value as *const u32 as *const u8,
        cbData: size_of::<u32>() as u32,
        dwFlags: 0,
    };

    // SAFETY: メッセージ経由での設定保存は、読み取り専用のバッファポインタを渡すため安全。
    unsafe {
        SendMessageW(
            hwnd,
            EE_REG_SET_VALUE,
            Some(WPARAM(0)),
            Some(LPARAM(&info as *const _ as isize)),
        )
        .0 as i32
    }
}

#[allow(dead_code)]
pub fn output_string(window_id: WindowId, text: &str) {
    let hwnd = HWND(window_id.0 as _);
    let wide_text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: メッセージ経由での文字列出力は安全。
    unsafe {
        let _ = SendMessageW(
            hwnd,
            EE_OUTPUT_STRING,
            Some(WPARAM(0)),
            Some(LPARAM(wide_text.as_ptr() as isize)),
        );
    }
}

pub fn is_system_dark_mode() -> bool {
    windows_registry::CURRENT_USER
        .open("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")
        .and_then(|key| key.get_u32("AppsUseLightTheme"))
        .map(|val| val == 0)
        .unwrap_or(false)
}

pub fn show_message_box(window_id: WindowId, text: &str, caption: &str, u_type: u32) -> i32 {
    let hwnd = HWND(window_id.0 as _);
    let wide_text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let wide_caption: Vec<u16> = caption.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: メッセージボックスの表示は、有効な HWND とヌル終端文字列ポインタを使用するため安全。
    unsafe {
        MessageBoxW(
            Some(hwnd),
            PCWSTR(wide_text.as_ptr()),
            PCWSTR(wide_caption.as_ptr()),
            windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_STYLE(u_type),
        )
        .0
    }
}
