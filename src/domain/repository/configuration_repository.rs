use crate::domain::model::terminal_config_value::TerminalConfig;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    SaveFailed(String),
    #[allow(dead_code)]
    LoadFailed(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::SaveFailed(msg) => write!(f, "Save failed: {}", msg),
            ConfigError::LoadFailed(msg) => write!(f, "Load failed: {}", msg),
        }
    }
}

pub trait ConfigurationRepository: Send + Sync {
    /// 構成情報をロードする
    fn load(&self) -> TerminalConfig;
    /// 構成情報を保存する
    fn save(&self, config: &TerminalConfig) -> Result<(), ConfigError>;

    /// ターミナル構成情報を取得する（キャッシュされた値を返すことが想定される）
    #[allow(dead_code)]
    fn get_terminal_config(&self) -> TerminalConfig;
}
