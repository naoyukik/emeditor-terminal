use crate::domain::repository::configuration_repository::ConfigurationRepository;

pub struct EmEditorConfigRepositoryImpl;

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
}
