use super::terminal_types_entity::{Cell, CursorStyle};

pub type SelectionRange = Option<((usize, usize), (usize, usize))>;

pub trait TerminalBufferViewEntity {
    fn get_line_at_visual_row(&self, visual_row: usize) -> Option<&Vec<Cell>>;
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;
    fn get_viewport_offset(&self) -> usize;
    fn get_ime_anchor_pos(&self) -> (usize, usize);
    fn get_cursor_pos(&self) -> (usize, usize);
    fn get_selection_range(&self) -> SelectionRange;
    fn is_cursor_visible(&self) -> bool;
    fn get_cursor_style(&self) -> CursorStyle;
}
