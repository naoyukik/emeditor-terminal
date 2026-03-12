use crate::domain::model::terminal_config_value::TerminalConfig;

pub trait ConfigurationRepository: Send + Sync {
    /// 構成情報をロードする
    fn load(&self) -> TerminalConfig;
    /// 構成情報を保存する
    fn save(&self, config: &TerminalConfig);

    /// ターミナル構成情報を取得する（キャッシュされた値を返すことが想定される）
    #[allow(dead_code)]
    fn get_terminal_config(&self) -> TerminalConfig;
}
