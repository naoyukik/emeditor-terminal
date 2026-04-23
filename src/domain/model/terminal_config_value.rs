use super::color_theme_value::ColorTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum ThemeType {
    #[default]
    SystemDefault,
    OneHalfDark,
    OneHalfLight,
}

impl ThemeType {
    /// 全てのテーマタイプと表示名、インデックスの定義
    const THEMES: &[(Self, &'static str, i32)] = &[
        (Self::SystemDefault, "System Default (Auto)", 0),
        (Self::OneHalfDark, "One Half Dark", 1),
        (Self::OneHalfLight, "One Half Light", 2),
    ];

    pub fn from_index(index: i32) -> Self {
        Self::THEMES
            .iter()
            .find(|(_, _, i)| *i == index)
            .map(|(t, _, _)| *t)
            .unwrap_or(Self::SystemDefault)
    }

    pub fn to_index(self) -> i32 {
        Self::THEMES
            .iter()
            .find(|(t, _, _)| *t == self)
            .map(|(_, _, i)| *i)
            .expect("ThemeType is missing from ThemeType::THEMES mapping")
    }

    pub fn get_display_name(self) -> &'static str {
        Self::THEMES
            .iter()
            .find(|(t, _, _)| *t == self)
            .map(|(_, n, _)| *n)
            .unwrap_or_else(|| {
                panic!(
                    "ThemeType::{:?} has no display name mapping defined in THEMES",
                    self
                )
            })
    }

    pub fn all() -> Vec<Self> {
        Self::THEMES.iter().map(|(t, _, _)| *t).collect()
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

    #[test]
    fn test_theme_type_display_name() {
        assert_eq!(
            ThemeType::SystemDefault.get_display_name(),
            "System Default (Auto)"
        );
        assert_eq!(ThemeType::OneHalfDark.get_display_name(), "One Half Dark");
        assert_eq!(ThemeType::OneHalfLight.get_display_name(), "One Half Light");
    }

    #[test]
    fn test_theme_type_all() {
        let all = ThemeType::all();
        // 要素数と集合としての包含関係
        assert_eq!(all.len(), 3);
        assert!(all.contains(&ThemeType::SystemDefault));
        assert!(all.contains(&ThemeType::OneHalfDark));
        assert!(all.contains(&ThemeType::OneHalfLight));

        // 並び順が期待通りであること（ComboBoxの挿入順に影響するため重要）
        assert_eq!(all[0], ThemeType::SystemDefault);
        assert_eq!(all[1], ThemeType::OneHalfDark);
        assert_eq!(all[2], ThemeType::OneHalfLight);

        // 表示名に重複がないこと
        let names: Vec<&str> = all.iter().map(|t| t.get_display_name()).collect();
        let mut unique_names = names.clone();
        unique_names.sort();
        unique_names.dedup();
        assert_eq!(names.len(), unique_names.len());

        // 各位置と to_index()/from_index() の対応が一致していること (Round-trip)
        for (idx, theme) in all.iter().enumerate() {
            let idx_i32 = idx as i32;
            assert_eq!(theme.to_index(), idx_i32);
            assert_eq!(ThemeType::from_index(idx_i32), *theme);
        }
    }
}
