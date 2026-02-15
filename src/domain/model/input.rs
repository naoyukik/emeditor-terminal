/// 修飾キーの状態を表す構造体
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    pub is_ctrl_pressed: bool,
    pub is_shift_pressed: bool,
    pub is_alt_pressed: bool,
}

impl Modifiers {
    #[allow(dead_code)]
    pub fn none() -> Self {
        Self {
            is_ctrl_pressed: false,
            is_shift_pressed: false,
            is_alt_pressed: false,
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
