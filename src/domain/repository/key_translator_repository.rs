pub use crate::domain::model::input_value::{InputKey, MouseEvent};

/// キーボードおよびマウス入力の翻訳を行うトレイト
pub trait KeyTranslatorRepository: Send + Sync {
    /// キーイベントを、ターミナルに送信するバイトシーケンスに翻訳する
    fn translate(&self, key: InputKey) -> Option<Vec<u8>>;

    /// マウスイベントを、ターミナルに送信するバイトシーケンスに翻訳する
    fn translate_mouse(&self, event: MouseEvent) -> Option<Vec<u8>>;
}
