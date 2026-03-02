use crate::domain::model::terminal_config_value::TerminalConfig;
use crate::domain::repository::configuration_repository::ConfigurationRepository;
use crate::gui::resolver::terminal_window_resolver::SendHWND;
use crate::infra::driver::emeditor_io_driver::{
    self, REG_QUERY_VALUE_INFO, REG_SZ, REG_DWORD, EEREG_EMEDITORPLUGIN,
};
use windows::core::w;
use std::mem::size_of;

pub struct EmEditorConfigRepositoryImpl {
    hwnd: SendHWND,
}

impl EmEditorConfigRepositoryImpl {
    pub fn new(hwnd: SendHWND) -> Self {
        Self { hwnd }
    }

    fn query_string(&self, value_name: &str, default: &str) -> String {
        let mut buffer = [0u16; 260];
        let mut cb_data = (buffer.len() * size_of::<u16>()) as u32;
        let value_name_wide: Vec<u16> = value_name.encode_utf16().chain(std::iter::once(0)).collect();

        let mut info = REG_QUERY_VALUE_INFO {
            cbSize: size_of::<REG_QUERY_VALUE_INFO>(),
            dwKey: EEREG_EMEDITORPLUGIN,
            pszConfig: w!("emeditor-terminal"),
            pszValue: windows::core::PCWSTR(value_name_wide.as_ptr()),
            dwType: REG_SZ,
            lpData: buffer.as_mut_ptr() as *mut u8,
            lpcbData: &mut cb_data as *mut u32,
            dwFlags: 0,
        };

        if emeditor_io_driver::reg_query_value(self.hwnd.0, &mut info) == 0 {
            String::from_utf16_lossy(&buffer[..(cb_data as usize / 2).saturating_sub(1)])
        } else {
            default.to_string()
        }
    }

    fn query_dword(&self, value_name: &str, default: i32) -> i32 {
        let mut data: u32 = 0;
        let mut cb_data = size_of::<u32>() as u32;
        let value_name_wide: Vec<u16> = value_name.encode_utf16().chain(std::iter::once(0)).collect();

        let mut info = REG_QUERY_VALUE_INFO {
            cbSize: size_of::<REG_QUERY_VALUE_INFO>(),
            dwKey: EEREG_EMEDITORPLUGIN,
            pszConfig: w!("emeditor-terminal"),
            pszValue: windows::core::PCWSTR(value_name_wide.as_ptr()),
            dwType: REG_DWORD,
            lpData: &mut data as *mut u32 as *mut u8,
            lpcbData: &mut cb_data as *mut u32,
            dwFlags: 0,
        };

        if emeditor_io_driver::reg_query_value(self.hwnd.0, &mut info) == 0 {
            data as i32
        } else {
            default
        }
    }
}

impl ConfigurationRepository for EmEditorConfigRepositoryImpl {
    fn load(&self) -> TerminalConfig {
        let default = TerminalConfig::default();

        // EmEditor本体のウィンドウハンドルが無効な場合はデフォルトを返す
        if self.hwnd.0.0.is_null() {
            return default;
        }

        TerminalConfig {
            theme_type: default.theme_type, // TODO: Phase 2以降で実装
            font_face: self.query_string("FontFaceName", &default.font_face),
            font_size: self.query_dword("FontSize", default.font_size),
            shell_path: self.query_string("ShellPath", &default.shell_path),
        }
    }

    fn save(&self, _config: &TerminalConfig) {
        // TODO: EE_REG_SET_VALUE を使用して保存する
    }

    fn get_terminal_config(&self) -> TerminalConfig {
        self.load()
    }
}
