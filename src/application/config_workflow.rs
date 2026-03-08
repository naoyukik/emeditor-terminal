use crate::domain::model::terminal_config_value::TerminalConfig;
use crate::domain::repository::configuration_repository::ConfigurationRepository;

/// 設定関連のビジネスロジックを担当するワークフロー
pub struct ConfigWorkflow {
    config_repository: Box<dyn ConfigurationRepository>,
}

impl ConfigWorkflow {
    pub fn new(config_repository: Box<dyn ConfigurationRepository>) -> Self {
        Self { config_repository }
    }

    /// 現在の設定をロードする
    pub fn load_config(&self) -> TerminalConfig {
        self.config_repository.load()
    }

    /// 指定された設定を保存する
    pub fn save_config(&self, config: TerminalConfig) {
        self.config_repository.save(&config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::terminal_config_value::ThemeType;

    struct MockConfigRepository {
        config: std::sync::Mutex<TerminalConfig>,
    }

    impl ConfigurationRepository for MockConfigRepository {
        fn load(&self) -> TerminalConfig {
            self.config.lock().unwrap().clone()
        }
        fn save(&self, config: &TerminalConfig) {
            *self.config.lock().unwrap() = config.clone();
        }
        fn get_terminal_config(&self) -> TerminalConfig {
            self.load()
        }
    }

    #[test]
    fn test_load_and_save_config() {
        let initial_config = TerminalConfig::default();
        let repository = Box::new(MockConfigRepository {
            config: std::sync::Mutex::new(initial_config.clone()),
        });
        let workflow = ConfigWorkflow::new(repository);

        // Load
        let loaded = workflow.load_config();
        assert_eq!(loaded, initial_config);

        // Update
        let mut new_config = initial_config.clone();
        new_config.font_size = 14;
        new_config.theme_type = ThemeType::OneHalfLight;

        // Save
        workflow.save_config(new_config.clone());

        // Reload and verify
        let reloaded = workflow.load_config();
        assert_eq!(reloaded, new_config);
        assert_eq!(reloaded.font_size, 14);
    }
}
