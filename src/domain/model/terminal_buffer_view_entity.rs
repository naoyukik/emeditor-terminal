use super::terminal_types_entity::{Cell, CursorStyle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectionPoint {
    pub x: usize,
    pub logical_row: usize,
}

pub type SelectionRange = Option<(SelectionPoint, SelectionPoint)>;

pub fn normalized_selection_range(
    range: SelectionRange,
) -> Option<(SelectionPoint, SelectionPoint)> {
    let (start, end) = range?;
    if start.logical_row < end.logical_row
        || (start.logical_row == end.logical_row && start.x <= end.x)
    {
        Some((start, end))
    } else {
        Some((end, start))
    }
}

pub fn selection_contains(range: SelectionRange, x: usize, logical_row: usize) -> bool {
    let (start, end) = match normalized_selection_range(range) {
        Some(range) => range,
        None => return false,
    };

    if start == end {
        return false;
    }
    if logical_row < start.logical_row || logical_row > end.logical_row {
        return false;
    }
    if logical_row > start.logical_row && logical_row < end.logical_row {
        return true;
    }
    if start.logical_row == end.logical_row {
        return logical_row == start.logical_row && x >= start.x && x <= end.x;
    }
    if logical_row == start.logical_row {
        return x >= start.x;
    }
    if logical_row == end.logical_row {
        return x <= end.x;
    }
    false
}

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
