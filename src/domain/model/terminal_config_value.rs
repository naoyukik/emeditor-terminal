use super::color_theme_value::ColorTheme;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TerminalConfig {
    pub theme_type: ThemeType,
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
        };
        let theme = config.get_color_theme();
        assert_eq!(theme, ColorTheme::one_half_dark());
    }

    #[test]
    fn test_terminal_config_one_half_light() {
        let config = TerminalConfig {
            theme_type: ThemeType::OneHalfLight,
        };
        let theme = config.get_color_theme();
        assert_eq!(theme, ColorTheme::one_half_light());
    }
}
