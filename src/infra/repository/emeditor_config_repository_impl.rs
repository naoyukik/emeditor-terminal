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
                pszValue: windows::core::PCWSTR(value_name_wide.as_ptr()),
                dwType: REG_SZ,
                lpData: buffer.as_mut_ptr() as *mut u8,
                lpcbData: &mut cb_data as *mut u32,
                dwFlags: 0,
            };

            let ret = emeditor_io_driver::reg_query_value(self.hwnd.0, &mut info);

            if ret == 0 {
                // cb_data はバイト数なので UTF-16 コード単位数に変換
                let max_u16_len = cb_data as usize / size_of::<u16>();
                // バッファに書き込まれた実際の長さを超えないようにクランプ
                let max_u16_len = max_u16_len.min(buffer.len());

                // ヌル終端を探す
                let len = buffer
                    .iter()
                    .take(max_u16_len)
                    .position(|&c| c == 0)
                    .unwrap_or(max_u16_len);

                let result = String::from_utf16_lossy(&buffer[..len]);
                if result.trim().is_empty() {
                    return default.to_string();
                } else {
                    return result;
                }
            } else {
                // 失敗時に cb_data が現在のバッファより大きければ再確保してリトライ
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
            pszConfig: w!("Terminal"),
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

    fn set_string(&self, value_name: &str, value: &str) {
        let value_wide: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
        let value_name_wide: Vec<u16> = value_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let info = emeditor_io_driver::REG_SET_VALUE_INFO {
            cbSize: size_of::<emeditor_io_driver::REG_SET_VALUE_INFO>(),
            dwKey: EEREG_EMEDITORPLUGIN,
            pszConfig: w!("Terminal"),
            pszValue: windows::core::PCWSTR(value_name_wide.as_ptr()),
            dwType: emeditor_io_driver::REG_SZ,
            lpData: value_wide.as_ptr() as *const u8,
            cbData: (value_wide.len() * size_of::<u16>()) as u32,
            dwFlags: 0,
        };

        emeditor_io_driver::reg_set_value(self.hwnd.0, &info);
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
            pszConfig: w!("Terminal"),
            pszValue: windows::core::PCWSTR(value_name_wide.as_ptr()),
            dwType: emeditor_io_driver::REG_DWORD,
            lpData: std::ptr::addr_of!(data) as *const u8,
            cbData: size_of::<u32>() as u32,
            dwFlags: 0,
        };

        emeditor_io_driver::reg_set_value(self.hwnd.0, &info);
    }
}

impl ConfigurationRepository for EmEditorConfigRepositoryImpl {
    fn load(&self) -> TerminalConfig {
        let default = TerminalConfig::default();

        // EmEditor本体のウィンドウハンドルが無効な場合はデフォルトを返す
        if self.hwnd.0 .0.is_null() {
            return default;
        }

        let font_face = self.query_string("FontFaceName", &default.font_face);
        let font_size = self.query_dword("FontSize", default.font_size);
        let shell_path_raw = self.query_string("ShellPath", &default.shell_path);

        // ロードされたパスが絶対パスかつ存在する場合のみ採用、それ以外は再解決を試みる
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

        TerminalConfig {
            theme_type: default.theme_type,
            font_face,
            font_size,
            shell_path,
        }
    }

    fn save(&self, config: &TerminalConfig) {
        self.set_string("FontFaceName", &config.font_face);
        self.set_dword("FontSize", config.font_size);
        self.set_string("ShellPath", &config.shell_path);
    }

    fn get_terminal_config(&self) -> TerminalConfig {
        self.load()
    }
}
