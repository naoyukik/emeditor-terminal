use crate::domain::model::terminal_config_value::{TerminalConfig, ThemeType};
use crate::domain::model::window_id_value::WindowId;
use crate::domain::repository::configuration_repository::{ConfigError, ConfigurationRepository};
use crate::infra::driver::emeditor_io_driver;

pub struct EmEditorConfigRepositoryImpl {
    hwnd: WindowId,
}

impl EmEditorConfigRepositoryImpl {
    pub fn new(hwnd: WindowId) -> Self {
        Self { hwnd }
    }

    fn query_string(&self, value_name: &str, default: &str) -> String {
        emeditor_io_driver::emeditor_query_string(self.hwnd, value_name, default)
    }

    fn query_dword(&self, value_name: &str, default: i32) -> i32 {
        emeditor_io_driver::emeditor_query_u32(self.hwnd, value_name, default as u32) as i32
    }

    fn set_string(&self, value_name: &str, value: &str) -> i32 {
        emeditor_io_driver::emeditor_set_string(self.hwnd, value_name, value)
    }

    fn set_dword(&self, value_name: &str, value: i32) -> i32 {
        emeditor_io_driver::emeditor_set_u32(self.hwnd, value_name, value as u32)
    }
}

impl ConfigurationRepository for EmEditorConfigRepositoryImpl {
    fn load(&self) -> TerminalConfig {
        let default = TerminalConfig::default();

        if self.hwnd.0 == 0 {
            return default;
        }

        let theme_type_val = self.query_dword("ColorTheme", 0); // Default: SystemDefault
        let theme_type = ThemeType::from_index(theme_type_val);

        let font_face = self.query_string("FontFaceName", &default.font_face);
        let font_size = self.query_dword("FontSize", default.font_size);
        let font_weight = self.query_dword("FontWeight", default.font_weight);
        let font_italic =
            self.query_dword("FontItalic", if default.font_italic { 1 } else { 0 }) != 0;
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

        TerminalConfig {
            theme_type,
            font_face,
            font_size,
            font_weight,
            font_italic,
            shell_path,
        }
    }

    fn save(&self, config: &TerminalConfig) -> Result<(), ConfigError> {
        if self.hwnd.0 == 0 {
            log::warn!("EmEditorConfigRepositoryImpl: HWND is NULL, skipping save.");
            return Err(ConfigError::SaveFailed("HWND is NULL".to_string()));
        }
        // Always save to allow explicit clearing of settings
        log::info!(
            "EmEditorConfigRepositoryImpl: Saving config: theme={:?}, font_face={}, font_size={}, weight={}, italic={}",
            config.theme_type,
            config.font_face,
            config.font_size,
            config.font_weight,
            config.font_italic
        );

        let mut results = Vec::new();

        results.push(self.set_dword("ColorTheme", config.theme_type.to_index()));
        results.push(self.set_string("FontFaceName", &config.font_face));
        results.push(self.set_dword("FontSize", config.font_size));
        results.push(self.set_dword("FontWeight", config.font_weight));
        results.push(self.set_dword("FontItalic", if config.font_italic { 1 } else { 0 }));
        results.push(self.set_string("ShellPath", &config.shell_path));

        if results.iter().any(|&r| r != 0) {
            let err_msg = format!(
                "One or more settings failed to save. Return codes: {:?}",
                results
            );
            log::error!("{}", err_msg);
            return Err(ConfigError::SaveFailed(err_msg));
        }

        Ok(())
    }

    fn get_terminal_config(&self) -> TerminalConfig {
        self.load()
    }
}
