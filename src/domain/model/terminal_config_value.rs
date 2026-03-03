use super::color_theme_value::ColorTheme;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ThemeType {
    Campbell,
    OneHalfDark,
    OneHalfLight,
}

impl Default for ThemeType {
    fn default() -> Self {
        Self::OneHalfDark
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalConfig {
    pub theme_type: ThemeType,
    pub font_face: String,
    pub font_size: i32,
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
            theme_type: ThemeType::OneHalfDark,
            font_face: "Consolas".to_string(),
            font_size: 10,
            shell_path,
        }
    }
}

impl TerminalConfig {
    pub fn get_color_theme(&self) -> ColorTheme {
        match self.theme_type {
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
        assert_eq!(config.theme_type, ThemeType::OneHalfDark);
        let theme = config.get_color_theme();
        assert_eq!(theme, ColorTheme::one_half_dark());
    }

    #[test]
    fn test_terminal_config_one_half_dark() {
        let config = TerminalConfig {
            theme_type: ThemeType::OneHalfDark,
            ..TerminalConfig::default()
        };
        let theme = config.get_color_theme();
        assert_eq!(theme, ColorTheme::one_half_dark());
    }

    #[test]
    fn test_terminal_config_one_half_light() {
        let config = TerminalConfig {
            theme_type: ThemeType::OneHalfLight,
            ..TerminalConfig::default()
        };
        let theme = config.get_color_theme();
        assert_eq!(theme, ColorTheme::one_half_light());
    }
}
