use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::SendMessageW;

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

pub const EE_REG_QUERY_VALUE: u32 = EE_FIRST + 86; // 2134
pub const EE_REG_SET_VALUE: u32 = EE_FIRST + 85; // 2133

pub const EEREG_EMEDITORPLUGIN: u32 = 0x7fffff30;

pub const REG_SZ: u32 = 1;
pub const REG_DWORD: u32 = 4;

#[repr(C)]
#[allow(non_snake_case)]
pub struct REG_QUERY_VALUE_INFO {
    pub cbSize: usize,
    pub dwKey: u32,
    pub pszConfig: PCWSTR,
    pub pszValue: PCWSTR,
    pub dwType: u32,
    pub lpData: *mut u8,
    pub lpcbData: *mut u32,
    pub dwFlags: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct REG_SET_VALUE_INFO {
    pub cbSize: usize,
    pub dwKey: u32,
    pub pszConfig: PCWSTR,
    pub pszValue: PCWSTR,
    pub dwType: u32,
    pub lpData: *const u8,
    pub cbData: u32,
    pub dwFlags: u32,
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

#[allow(dead_code)]
pub fn output_string(hwnd: HWND, text: &str) {
    let wide_text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let _ = SendMessageW(
            hwnd,
            EE_OUTPUT_STRING,
            WPARAM(0),
            LPARAM(wide_text.as_ptr() as isize),
        );
    }
}

pub fn reg_query_value(hwnd: HWND, info: &mut REG_QUERY_VALUE_INFO) -> i32 {
    unsafe {
        let result = SendMessageW(
            hwnd,
            EE_REG_QUERY_VALUE,
            WPARAM(0),
            LPARAM(info as *mut _ as isize),
        );
        let ret = result.0 as i32;
        log::debug!(
            "reg_query_value: ret={}, value_name={}",
            ret,
            info.pszValue.display()
        );
        ret
    }
}

pub fn is_very_dark(_hwnd: HWND) -> bool {
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER, KEY_READ,
    };

    let mut h_key = windows::Win32::System::Registry::HKEY::default();
    let sub_key =
        windows::core::w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");

    unsafe {
        if RegOpenKeyExW(HKEY_CURRENT_USER, sub_key, 0, KEY_READ, &mut h_key).is_ok() {
            let mut data: u32 = 0;
            let mut cb_data = std::mem::size_of::<u32>() as u32;
            let value_name = windows::core::w!("AppsUseLightTheme");

            let result = RegQueryValueExW(
                h_key,
                value_name,
                None,
                None,
                Some(&mut data as *mut u32 as *mut u8),
                Some(&mut cb_data),
            );

            let _ = RegCloseKey(h_key);

            if result.is_ok() {
                return data == 0; // 0 means Dark Mode
            }
        }
    }

    // Default to Dark if detection fails (project preference)
    true
}

pub fn reg_set_value(hwnd: HWND, info: &REG_SET_VALUE_INFO) -> i32 {
    unsafe {
        let result = SendMessageW(
            hwnd,
            EE_REG_SET_VALUE,
            WPARAM(0),
            LPARAM(info as *const _ as isize),
        );
        let ret = result.0 as i32;
        log::debug!(
            "reg_set_value: ret={}, value_name={}",
            ret,
            info.pszValue.display()
        );
        ret
    }
}
