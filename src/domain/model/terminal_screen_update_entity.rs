use super::terminal_types_entity::{
    CursorStyle, MouseTrackingMode, TerminalAttribute,
};

pub trait TerminalScreenUpdateEntity {
    fn print_string(&mut self, text: &str);
    fn flush_pending_cluster(&mut self);

    fn move_cursor_up(&mut self, n: usize);
    fn move_cursor_down(&mut self, n: usize);
    fn move_cursor_forward(&mut self, n: usize);
    fn move_cursor_backward(&mut self, n: usize);
    fn move_cursor_to_pos(&mut self, row: usize, col: usize);
    fn move_cursor_to_col(&mut self, col: usize);
    fn move_cursor_to_row(&mut self, row: usize);

    fn handle_tab(&mut self);
    fn index(&mut self);
    fn reverse_index(&mut self);

    fn insert_lines(&mut self, n: usize);
    fn delete_lines(&mut self, n: usize);
    fn insert_cells(&mut self, n: usize);
    fn delete_cells(&mut self, n: usize);
    fn erase_cells(&mut self, n: usize);
    fn erase_in_line(&mut self, mode: u8);
    fn erase_in_display(&mut self, mode: u8);

    fn set_scroll_region(&mut self, top: usize, bottom: usize);
    fn scroll_up(&mut self);
    fn scroll_down(&mut self);
    fn get_height(&self) -> usize;

    fn set_origin_mode(&mut self, on: bool);
    fn set_cursor_visible(&mut self, visible: bool);
    fn set_cursor_style(&mut self, style: CursorStyle);

    fn set_attribute(&mut self, attribute: TerminalAttribute);
    fn get_current_attribute(&self) -> &TerminalAttribute;

    fn save_cursor(&mut self);
    fn restore_cursor(&mut self);

    fn set_mouse_tracking_mode(&mut self, mode: MouseTrackingMode);
    fn set_sgr_mouse_encoding(&mut self, enabled: bool);
}
