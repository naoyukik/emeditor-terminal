use super::color_theme_value::ColorTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ThemeType {
    SystemDefault,
    Campbell,
    OneHalfDark,
    OneHalfLight,
}

impl ThemeType {
    pub fn from_index(index: i32) -> Self {
        match index {
            1 => Self::OneHalfDark,
            2 => Self::OneHalfLight,
            _ => Self::SystemDefault,
        }
    }

    pub fn to_index(self) -> i32 {
        match self {
            Self::SystemDefault => 0,
            Self::OneHalfDark => 1,
            Self::OneHalfLight => 2,
            Self::Campbell => 0, // 暫定的に SystemDefault と同じ
        }
    }
}

impl Default for ThemeType {
    fn default() -> Self {
        Self::SystemDefault
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalConfig {
    pub theme_type: ThemeType,
    pub font_face: String,
    pub font_size: i32,
    pub font_weight: i32,
    pub font_italic: bool,
    pub shell_path: String,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        // pwsh.exe -> powershell.exe -> cmd.exe の順で絶対パスを取得する
        let shell_path = which::which("pwsh.exe")
            .or_else(|_| which::which("powershell.exe"))
            .or_else(|_| which::which("cmd.exe"))
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "cmd.exe".to_string());

        Self {
            theme_type: ThemeType::default(),
            font_face: "Consolas".to_string(),
            font_size: 10,
            font_weight: 400, // FW_NORMAL
            font_italic: false,
            shell_path,
        }
    }
}

impl TerminalConfig {
    pub fn get_color_theme(&self, is_dark: bool) -> ColorTheme {
        match self.theme_type {
            ThemeType::SystemDefault => {
                if is_dark {
                    ColorTheme::one_half_dark()
                } else {
                    ColorTheme::one_half_light()
                }
            }
            ThemeType::Campbell => ColorTheme::campbell(),
            ThemeType::OneHalfDark => ColorTheme::one_half_dark(),
            ThemeType::OneHalfLight => ColorTheme::one_half_light(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_config_default_theme() {
        let config = TerminalConfig::default();
        assert_eq!(config.theme_type, ThemeType::SystemDefault);
        assert_eq!(config.font_weight, 400);
        assert!(!config.font_italic);
        assert_eq!(config.get_color_theme(true), ColorTheme::one_half_dark());
        assert_eq!(config.get_color_theme(false), ColorTheme::one_half_light());
    }

    #[test]
    fn test_terminal_config_one_half_dark() {
        let config = TerminalConfig {
            theme_type: ThemeType::OneHalfDark,
            font_weight: 700,
            font_italic: true,
            ..TerminalConfig::default()
        };
        assert_eq!(config.font_weight, 700);
        assert!(config.font_italic);
        let theme = config.get_color_theme(true);
        assert_eq!(theme, ColorTheme::one_half_dark());
    }

    #[test]
    fn test_terminal_config_one_half_light() {
        let config = TerminalConfig {
            theme_type: ThemeType::OneHalfLight,
            ..TerminalConfig::default()
        };
        let theme = config.get_color_theme(true);
        assert_eq!(theme, ColorTheme::one_half_light());
    }

    #[test]
    fn test_theme_type_index_mapping() {
        assert_eq!(ThemeType::from_index(0), ThemeType::SystemDefault);
        assert_eq!(ThemeType::from_index(1), ThemeType::OneHalfDark);
        assert_eq!(ThemeType::from_index(2), ThemeType::OneHalfLight);
        assert_eq!(ThemeType::from_index(99), ThemeType::SystemDefault);

        assert_eq!(ThemeType::SystemDefault.to_index(), 0);
        assert_eq!(ThemeType::OneHalfDark.to_index(), 1);
        assert_eq!(ThemeType::OneHalfLight.to_index(), 2);
    }
}
