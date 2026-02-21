use super::color_theme_value::ColorTheme;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeType {
    Campbell,
    SolarizedDark,
    SolarizedLight,
}

impl Default for ThemeType {
    fn default() -> Self {
        Self::Campbell
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
            ThemeType::SolarizedDark => ColorTheme::solarized_dark(),
            ThemeType::SolarizedLight => ColorTheme::solarized_light(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_config_default_theme() {
        let config = TerminalConfig::default();
        assert_eq!(config.theme_type, ThemeType::Campbell);
        let theme = config.get_color_theme();
        assert_eq!(theme, ColorTheme::campbell());
    }

    #[test]
    fn test_terminal_config_solarized_dark() {
        let config = TerminalConfig {
            theme_type: ThemeType::SolarizedDark,
        };
        let theme = config.get_color_theme();
        assert_eq!(theme, ColorTheme::solarized_dark());
    }

    #[test]
    fn test_terminal_config_solarized_light() {
        let config = TerminalConfig {
            theme_type: ThemeType::SolarizedLight,
        };
        let theme = config.get_color_theme();
        assert_eq!(theme, ColorTheme::solarized_light());
    }
}
