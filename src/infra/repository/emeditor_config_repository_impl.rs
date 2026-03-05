use crate::domain::model::terminal_config_value::TerminalConfig;
use crate::domain::repository::configuration_repository::ConfigurationRepository;
use crate::gui::resolver::terminal_window_resolver::SendHWND;
use crate::infra::driver::emeditor_io_driver::{
    self, EEREG_EMEDITORPLUGIN, REG_DWORD, REG_QUERY_VALUE_INFO, REG_SZ,
};
use std::mem::size_of;
use windows::core::w;

pub struct EmEditorConfigRepositoryImpl {
    hwnd: SendHWND,
}

impl EmEditorConfigRepositoryImpl {
    pub fn new(hwnd: SendHWND) -> Self {
        Self { hwnd }
    }

    fn query_string(&self, value_name: &str, default: &str) -> String {
        let mut buffer: Vec<u16> = vec![0u16; 260];
        let value_name_wide: Vec<u16> = value_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut cb_data = (buffer.len() * size_of::<u16>()) as u32;

        let mut info = REG_QUERY_VALUE_INFO {
            cbSize: size_of::<REG_QUERY_VALUE_INFO>(),
            dwKey: EEREG_EMEDITORPLUGIN,
            pszConfig: w!("emeditor_terminal.dll"),
            pszValue: windows::core::PCWSTR(value_name_wide.as_ptr()),
            dwType: REG_SZ,
            lpData: buffer.as_mut_ptr() as *mut u8,
            lpcbData: &mut cb_data as *mut u32,
            dwFlags: 0,
        };

        // Editor_RegQueryValue returns ERROR_SUCCESS (0) on success.
        let ret = emeditor_io_driver::reg_query_value(self.hwnd.0, &mut info);

        if ret == 0 {
            // cb_data is bytes, convert to number of u16 units.
            let len_with_null = (cb_data as usize / size_of::<u16>()).min(buffer.len());
            let result = String::from_utf16_lossy(&buffer[..len_with_null]);
            let result = result.trim_matches('\0').to_string();
            if result.is_empty() {
                default.to_string()
            } else {
                result
            }
        } else {
            log::warn!("query_string: failed with ret={}, using default", ret);
            default.to_string()
        }
    }

    fn query_dword(&self, value_name: &str, default: i32) -> i32 {
        let mut data: u32 = 0;
        let mut cb_data = size_of::<u32>() as u32;
        let value_name_wide: Vec<u16> = value_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut info = REG_QUERY_VALUE_INFO {
            cbSize: size_of::<REG_QUERY_VALUE_INFO>(),
            dwKey: EEREG_EMEDITORPLUGIN,
            pszConfig: w!("emeditor_terminal.dll"),
            pszValue: windows::core::PCWSTR(value_name_wide.as_ptr()),
            dwType: REG_DWORD,
            lpData: &mut data as *mut u32 as *mut u8,
            lpcbData: &mut cb_data as *mut u32,
            dwFlags: 0,
        };

        let ret = emeditor_io_driver::reg_query_value(self.hwnd.0, &mut info);
        if ret == 0 {
            log::info!("query_dword: success, value={}", data);
            data as i32
        } else {
            log::warn!("query_dword: failed with ret={}, using default={}", ret, default);
            default
        }
    }

    fn set_string(&self, value_name: &str, value: &str) {
        if value.is_empty() && value_name == "ShellPath" {
            return;
        }
        
        let value_wide: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
        let value_name_wide: Vec<u16> = value_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let info = emeditor_io_driver::REG_SET_VALUE_INFO {
            cbSize: size_of::<emeditor_io_driver::REG_SET_VALUE_INFO>(),
            dwKey: EEREG_EMEDITORPLUGIN,
            pszConfig: w!("emeditor_terminal.dll"),
            pszValue: windows::core::PCWSTR(value_name_wide.as_ptr()),
            dwType: emeditor_io_driver::REG_SZ,
            lpData: value_wide.as_ptr() as *const u8,
            cbData: (value_wide.len() * size_of::<u16>()) as u32,
            dwFlags: 0,
        };

        let ret = emeditor_io_driver::reg_set_value(self.hwnd.0, &info);
        if ret != 0 {
            log::error!("set_string: failed with ret={}", ret);
        }
    }

    fn set_dword(&self, value_name: &str, value: i32) {
        let data = value as u32;
        let value_name_wide: Vec<u16> = value_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let info = emeditor_io_driver::REG_SET_VALUE_INFO {
            cbSize: size_of::<emeditor_io_driver::REG_SET_VALUE_INFO>(),
            dwKey: EEREG_EMEDITORPLUGIN,
            pszConfig: w!("emeditor_terminal.dll"),
            pszValue: windows::core::PCWSTR(value_name_wide.as_ptr()),
            dwType: emeditor_io_driver::REG_DWORD,
            lpData: &data as *const u32 as *const u8,
            cbData: size_of::<u32>() as u32,
            dwFlags: 0,
        };

        let ret = emeditor_io_driver::reg_set_value(self.hwnd.0, &info);
        if ret != 0 {
            log::error!("set_dword: failed with ret={}", ret);
        }
    }
}

impl ConfigurationRepository for EmEditorConfigRepositoryImpl {
    fn load(&self) -> TerminalConfig {
        let default = TerminalConfig::default();

        if self.hwnd.0 .0.is_null() {
            return default;
        }

        let font_face = self.query_string("FontFaceName", &default.font_face);
        let font_size = self.query_dword("FontSize", default.font_size);
        let shell_path_raw = self.query_string("ShellPath", &default.shell_path);

        let shell_path = {
            let path = std::path::Path::new(&shell_path_raw);
            if path.is_absolute() && path.exists() {
                shell_path_raw
            } else {
                which::which(&shell_path_raw)
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or(shell_path_raw)
            }
        };

        let config = TerminalConfig {
            theme_type: default.theme_type,
            font_face,
            font_size,
            shell_path,
        };
        log::info!("EmEditorConfigRepositoryImpl::load: final face={}, size={}", config.font_face, config.font_size);
        config
    }

    fn save(&self, config: &TerminalConfig) {
        log::info!("EmEditorConfigRepositoryImpl::save: face={}, size={}", config.font_face, config.font_size);
        self.set_string("FontFaceName", &config.font_face);
        self.set_dword("FontSize", config.font_size);
        if !config.shell_path.is_empty() {
            self.set_string("ShellPath", &config.shell_path);
        }
    }

    fn get_terminal_config(&self) -> TerminalConfig {
        self.load()
    }
}
