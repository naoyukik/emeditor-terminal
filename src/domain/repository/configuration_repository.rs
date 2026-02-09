pub trait ConfigurationRepository: Send + Sync {
    /// フォント名を取得する
    fn get_font_face(&self) -> String;
    /// フォントサイズを取得する
    fn get_font_size(&self) -> i32;
    /// 背景色を取得する
    fn get_background_color(&self) -> u32;
    /// 前景色を取得する
    fn get_foreground_color(&self) -> u32;
}