/// クリップボード操作を抽象化するリポジトリ
pub trait ClipboardRepository: Send + Sync {
    /// クリップボードからテキストを取得する
    fn get_text(&self) -> Result<String, String>;
}
