use crate::domain::model::terminal_config_value::{TerminalConfig, ThemeType};
use crate::domain::repository::configuration_repository::ConfigurationRepository;

#[allow(dead_code)]
pub struct EmEditorConfigRepositoryImpl;

#[allow(dead_code)]
impl EmEditorConfigRepositoryImpl {
    pub fn new() -> Self {
        Self
    }
}

impl ConfigurationRepository for EmEditorConfigRepositoryImpl {
    fn get_font_face(&self) -> String {
        // TODO: EmEditor SDKを介して実際のフォント名を取得する
        "Consolas".to_string()
    }

    fn get_font_size(&self) -> i32 {
        // TODO: EmEditor SDKを介して実際のフォントサイズを取得する
        12
    }

    fn get_background_color(&self) -> u32 {
        0x00000000 // Black
    }

    fn get_foreground_color(&self) -> u32 {
        0x00FFFFFF // White
    }

    fn get_terminal_config(&self) -> TerminalConfig {
        // TODO: EE_REG_QUERY_VALUE などのEmEditor APIを使用して実際のテーマ設定を読み込む。
        // 現在はOne Half Darkをハードコードして使用する（Phase 2要件）。
        TerminalConfig {
            theme_type: ThemeType::OneHalfDark,
        }
    }
}
