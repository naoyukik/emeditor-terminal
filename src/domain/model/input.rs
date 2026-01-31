/// 修飾キーの状態を表す構造体
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Modifiers {
    #[allow(dead_code)]
    pub fn none() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
        }
    }
}

/// ターミナルへの入力キーイベントを表すドメインモデル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputKey {
    pub vk_code: u16,
    pub modifiers: Modifiers,
}

impl InputKey {
    pub fn new(vk_code: u16, modifiers: Modifiers) -> Self {
        Self { vk_code, modifiers }
    }
}
