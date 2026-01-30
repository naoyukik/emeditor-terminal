pub use crate::domain::model::input::{InputKey, Modifiers};

/// キーボード入力の翻訳を行うトレイト
pub trait KeyTranslator {
    /// キーイベントを、ターミナルに送信するバイトシーケンスに翻訳する
    fn translate(&self, key: InputKey) -> Option<Vec<u8>>;
}

/// VTシーケンス（ANSIエスケープシーケンス）への翻訳を行う実装
pub struct VtSequenceTranslator;

impl VtSequenceTranslator {
    pub fn new() -> Self {
        Self
    }
}

impl KeyTranslator for VtSequenceTranslator {
    fn translate(&self, key: InputKey) -> Option<Vec<u8>> {
        let vk_code = key.vk_code;
        let ctrl = key.modifiers.ctrl;
        let shift = key.modifiers.shift;
        let alt = key.modifiers.alt;

        // Win32 Virtual-Key Codes (一部抜粋、実装に必要なもの)
        const VK_BACK: u16 = 0x08;
        const VK_TAB: u16 = 0x09;
        const VK_RETURN: u16 = 0x0D;
        const VK_ESCAPE: u16 = 0x1B;
        const VK_PRIOR: u16 = 0x21; // Page Up
        const VK_NEXT: u16 = 0x22;  // Page Down
        const VK_END: u16 = 0x23;
        const VK_HOME: u16 = 0x24;
        const VK_LEFT: u16 = 0x25;
        const VK_UP: u16 = 0x26;
        const VK_RIGHT: u16 = 0x27;
        const VK_DOWN: u16 = 0x28;
        const VK_INSERT: u16 = 0x2D;
        const VK_DELETE: u16 = 0x2E;
        const VK_F1: u16 = 0x70;
        const VK_F2: u16 = 0x71;
        const VK_F3: u16 = 0x72;
        const VK_F4: u16 = 0x73;
        const VK_F5: u16 = 0x74;
        const VK_F6: u16 = 0x75;
        const VK_F7: u16 = 0x76;
        const VK_F8: u16 = 0x77;
        const VK_F9: u16 = 0x78;
        const VK_F10: u16 = 0x79;
        const VK_F11: u16 = 0x7A;
        const VK_F12: u16 = 0x7B;

        // Ctrl+ combinations (A-Z)
        if ctrl && !alt {
            if (0x41..=0x5A).contains(&vk_code) {
                let ctrl_char = (vk_code - 0x40) as u8;
                return Some(vec![ctrl_char]);
            }
        }

        // Alt + Letter/Number (Meta key)
        if alt && !ctrl {
            if (0x30..=0x39).contains(&vk_code) {
                // VK_0..VK_9 は 0x30..0x39 であり、結果として ASCII '0'..'9' と一致する。
                // 偶然の一致に依存しているように見えないよう、明示的に ASCII 数字へ変換する。
                let char_to_send = b'0' + (vk_code - 0x30) as u8;
                return Some(vec![0x1B, char_to_send]);
            } else if (0x41..=0x5A).contains(&vk_code) {
                let mut char_to_send = vk_code as u8;
                if !shift {
                    char_to_send = (vk_code + 0x20) as u8; // To lowercase
                }
                return Some(vec![0x1B, char_to_send]);
            }
        }

        // Special keys with modifiers
        let seq: Option<&[u8]> = match vk_code {
            VK_UP => {
                if ctrl { Some(b"\x1b[1;5A") }
                else if shift { Some(b"\x1b[1;2A") }
                else if alt { Some(b"\x1b[1;3A") }
                else { Some(b"\x1b[A") }
            }
            VK_DOWN => {
                if ctrl { Some(b"\x1b[1;5B") }
                else if shift { Some(b"\x1b[1;2B") }
                else if alt { Some(b"\x1b[1;3B") }
                else { Some(b"\x1b[B") }
            }
            VK_RIGHT => {
                if ctrl { Some(b"\x1b[1;5C") }
                else if shift { Some(b"\x1b[1;2C") }
                else if alt { Some(b"\x1b[1;3C") }
                else { Some(b"\x1b[C") }
            }
            VK_LEFT => {
                if ctrl { Some(b"\x1b[1;5D") }
                else if shift { Some(b"\x1b[1;2D") }
                else if alt { Some(b"\x1b[1;3D") }
                else { Some(b"\x1b[D") }
            }
            VK_HOME => {
                if ctrl { Some(b"\x1b[1;5H") }
                else if shift { Some(b"\x1b[1;2H") }
                else { Some(b"\x1b[H") }
            }
            VK_END => {
                if ctrl { Some(b"\x1b[1;5F") }
                else if shift { Some(b"\x1b[1;2F") }
                else { Some(b"\x1b[F") }
            }
            VK_DELETE => Some(b"\x1b[3~"),
            VK_INSERT => Some(b"\x1b[2~"),
            VK_PRIOR => Some(b"\x1b[5~"), // Page Up
            VK_NEXT => Some(b"\x1b[6~"),  // Page Down
            VK_BACK => Some(b"\x7f"),     // Backspace (DEL)
            VK_RETURN => Some(b"\r"),     // Enter
            VK_TAB => Some(b"\t"),        // Tab
            VK_ESCAPE => Some(b"\x1b"),   // Escape
            VK_F1 => Some(b"\x1bOP"),
            VK_F2 => Some(b"\x1bOQ"),
            VK_F3 => Some(b"\x1bOR"),
            VK_F4 => Some(b"\x1bOS"),
            VK_F5 => Some(b"\x1b[15~"),
            VK_F6 => Some(b"\x1b[17~"),
            VK_F7 => Some(b"\x1b[18~"),
            VK_F8 => Some(b"\x1b[19~"),
            VK_F9 => Some(b"\x1b[20~"),
            VK_F10 => Some(b"\x1b[21~"),
            VK_F11 => Some(b"\x1b[23~"),
            VK_F12 => Some(b"\x1b[24~"),
            _ => None,
        };

        seq.map(|s| s.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_combinations() {
        let translator = VtSequenceTranslator::new();
        // Ctrl+A -> \x01
        let key_a = InputKey::new(0x41, Modifiers { ctrl: true, shift: false, alt: false });
        assert_eq!(translator.translate(key_a), Some(vec![1]));

        // Ctrl+C -> \x03
        let key_c = InputKey::new(0x43, Modifiers { ctrl: true, shift: false, alt: false });
        assert_eq!(translator.translate(key_c), Some(vec![3]));
    }

    #[test]
    fn test_ctrl_alt_combinations() {
        let translator = VtSequenceTranslator::new();
        // Ctrl+Alt+A -> Should be None (or distinct if supported)
        let key_ctrl_alt_a = InputKey::new(0x41, Modifiers { ctrl: true, shift: false, alt: true });
        assert_eq!(translator.translate(key_ctrl_alt_a), None);
    }

    #[test]
    fn test_meta_keys() {
        let translator = VtSequenceTranslator::new();
        // Alt+A -> ESC + a
        let key_alt_a = InputKey::new(0x41, Modifiers { ctrl: false, shift: false, alt: true });
        assert_eq!(translator.translate(key_alt_a), Some(vec![0x1B, b'a']));

        // Alt+Shift+A -> ESC + A
        let key_alt_shift_a = InputKey::new(0x41, Modifiers { ctrl: false, shift: true, alt: true });
        assert_eq!(translator.translate(key_alt_shift_a), Some(vec![0x1B, b'A']));

        // Alt+1 -> ESC + 1
        let key_alt_1 = InputKey::new(0x31, Modifiers { ctrl: false, shift: false, alt: true });
        assert_eq!(translator.translate(key_alt_1), Some(vec![0x1B, b'1']));
    }

    #[test]
    fn test_function_keys() {
        let translator = VtSequenceTranslator::new();
        // F1 -> \x1bOP
        let f1 = InputKey::new(0x70, Modifiers::none());
        assert_eq!(translator.translate(f1), Some(b"\x1bOP".to_vec()));

        // F5 -> \x1b[15~
        let f5 = InputKey::new(0x74, Modifiers::none());
        assert_eq!(translator.translate(f5), Some(b"\x1b[15~".to_vec()));

        // F12 -> \x1b[24~
        let f12 = InputKey::new(0x7B, Modifiers::none());
        assert_eq!(translator.translate(f12), Some(b"\x1b[24~".to_vec()));
    }

    #[test]
    fn test_special_keys() {
        let translator = VtSequenceTranslator::new();
        // Up Arrow
        let up = InputKey::new(0x26, Modifiers::none());
        assert_eq!(translator.translate(up), Some(b"\x1b[A".to_vec()));

        // Ctrl+Up Arrow
        let ctrl_up = InputKey::new(0x26, Modifiers { ctrl: true, shift: false, alt: false });
        assert_eq!(translator.translate(ctrl_up), Some(b"\x1b[1;5A".to_vec()));

        // Alt+Up Arrow
        let alt_up = InputKey::new(0x26, Modifiers { ctrl: false, shift: false, alt: true });
        assert_eq!(translator.translate(alt_up), Some(b"\x1b[1;3A".to_vec()));

        // Backspace
        let backspace = InputKey::new(0x08, Modifiers::none());
        assert_eq!(translator.translate(backspace), Some(vec![0x7f]));

        // Enter
        let enter = InputKey::new(0x0D, Modifiers::none());
        assert_eq!(translator.translate(enter), Some(vec![0x0D]));

        // Tab
        let tab = InputKey::new(0x09, Modifiers::none());
        assert_eq!(translator.translate(tab), Some(vec![0x09]));

        // Escape
        let esc = InputKey::new(0x1B, Modifiers::none());
        assert_eq!(translator.translate(esc), Some(vec![0x1B]));

        // Page Up
        let pgup = InputKey::new(0x21, Modifiers::none());
        assert_eq!(translator.translate(pgup), Some(b"\x1b[5~".to_vec()));

        // Space
        let space = InputKey::new(0x20, Modifiers::none());
        assert_eq!(translator.translate(space), None);
    }

    #[test]
    fn test_ignored_keys() {
        let translator = VtSequenceTranslator::new();
        // Shift only
        let shift = InputKey::new(0x10, Modifiers { ctrl: false, shift: true, alt: false });
        assert_eq!(translator.translate(shift), None);
    }
}
