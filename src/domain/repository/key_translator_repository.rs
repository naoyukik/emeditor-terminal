pub use crate::domain::model::input::InputKey;

/// キーボード入力の翻訳を行うトレイト
pub trait KeyTranslatorRepository: Send + Sync {
    /// キーイベントを、ターミナルに送信するバイトシーケンスに翻訳する
    fn translate(&self, key: InputKey) -> Option<Vec<u8>>;
}
