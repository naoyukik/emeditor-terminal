use super::terminal_types_entity::{Cell, CursorStyle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectionPoint {
    pub x: usize,
    pub logical_row: usize,
}

pub type SelectionRange = Option<(SelectionPoint, SelectionPoint)>;

pub trait TerminalBufferViewEntity {
    fn get_line_at_visual_row(&self, visual_row: usize) -> Option<&Vec<Cell>>;
    fn visual_row_to_logical_row(&self, visual_row: usize) -> usize;
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;
    fn get_viewport_offset(&self) -> usize;
    fn get_ime_anchor_pos(&self) -> (usize, usize);
    fn get_cursor_pos(&self) -> (usize, usize);
    fn get_selection_range(&self) -> SelectionRange;
    fn is_cursor_visible(&self) -> bool;
    fn get_cursor_style(&self) -> CursorStyle;
}
