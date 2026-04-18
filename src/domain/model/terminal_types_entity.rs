use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, PartialEq)]
pub enum TerminalColor {
    Default,
    Ansi(u8),
    Xterm(u8),
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TerminalAttribute {
    pub fg: TerminalColor,
    pub bg: TerminalColor,
    pub is_bold: bool,
    pub is_dim: bool,
    pub is_italic: bool,
    pub is_underline: bool,
    pub is_inverse: bool,
    pub is_strikethrough: bool,
}

impl Default for TerminalAttribute {
    fn default() -> Self {
        Self {
            fg: TerminalColor::Default,
            bg: TerminalColor::Default,
            is_bold: false,
            is_dim: false,
            is_italic: false,
            is_underline: false,
            is_inverse: false,
            is_strikethrough: false,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Cell {
    pub text: String,
    pub attribute: TerminalAttribute,
    pub is_wide_continuation: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            text: " ".to_string(),
            attribute: TerminalAttribute::default(),
            is_wide_continuation: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorStyle {
    BlinkingBlock,
    SteadyBlock,
    BlinkingUnderline,
    SteadyUnderline,
    BlinkingBar,
    SteadyBar,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self::BlinkingBar
    }
}

pub struct Cursor {
    pub x: usize,
    pub y: usize,
    pub is_visible: bool,
    pub style: CursorStyle,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            is_visible: true,
            style: CursorStyle::default(),
        }
    }
}
