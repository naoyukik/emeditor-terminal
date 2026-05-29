use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::terminal_buffer_view_entity::{
    normalized_selection_range, SelectionRange, TerminalBufferViewEntity,
    selection_contains,
};
use super::terminal_history_view_entity::TerminalHistoryViewEntity;
use super::terminal_screen_update_entity::TerminalScreenUpdateEntity;
// 基本型を再エクスポートし、外部からアクセス可能にする
use super::terminal_grid_entity::TerminalGridEntity;
use super::terminal_scrollback_entity::TerminalScrollbackEntity;
pub use super::terminal_types_entity::{
    Cell, Cursor, CursorStyle, MouseTrackingMode, TerminalAttribute, TerminalColor,
};

pub struct TerminalBufferEntity {
    grid: TerminalGridEntity,
    scrollback: TerminalScrollbackEntity,
    width: usize,
    height: usize,
    cursor: Cursor,
    current_attribute: TerminalAttribute,
    scroll_top: usize,
    scroll_bottom: usize,
    is_origin_mode: bool,
    last_inverse_render_pos: Option<(usize, usize)>,
    saved_cursor: Option<(usize, usize)>,
    pending_cluster: String,
    mouse_tracking_mode: MouseTrackingMode,
    use_sgr_mouse_encoding: bool,
    last_mouse_pos: Option<(usize, usize)>,
    selection_range: SelectionRange,
}

impl TerminalBufferEntity {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: TerminalGridEntity::new(width, height),
            scrollback: TerminalScrollbackEntity::new(10000),
            width,
            height,
            cursor: Cursor::default(),
            current_attribute: TerminalAttribute::default(),
            scroll_top: 0,
            scroll_bottom: height.saturating_sub(1),
            is_origin_mode: false,
            last_inverse_render_pos: None,
            saved_cursor: None,
            pending_cluster: String::new(),
            mouse_tracking_mode: MouseTrackingMode::None,
            use_sgr_mouse_encoding: false,
            last_mouse_pos: None,
            selection_range: None,
        }
    }

    fn get_empty_cell(&self) -> Cell {
        Cell {
            text: " ".to_string(),
            attribute: TerminalAttribute {
                fg: TerminalColor::Default,
                bg: self.current_attribute.bg.clone(),
                ..TerminalAttribute::default()
            },
            is_wide_continuation: false,
        }
    }

    #[allow(dead_code)]
    pub fn print_cell(&mut self, c: char) {
        if self.cursor.x < self.width
            && self.cursor.y < self.height
            && self.current_attribute.is_inverse
        {
            self.last_inverse_render_pos = Some((self.cursor.x, self.cursor.y));
        }
        if c.is_control() && !"\r\n\t\x08".contains(c) {
            return;
        }
        self.pending_cluster.push(c);
        self.process_graphemes();
    }

    pub fn print_string(&mut self, s: &str) {
        // 制御文字は個別に処理されるべきだが、一括書き込み内では無視するかフィルタリングする
        // vte から渡される文字列は基本的に印字可能文字のみのはず
        self.pending_cluster.push_str(s);
        self.process_graphemes();
    }

    fn process_graphemes(&mut self) {
        let mut clusters: Vec<String> = self
            .pending_cluster
            .graphemes(true)
            .map(|s| s.to_string())
            .collect();
        if clusters.len() > 1 {
            let last = clusters.pop().unwrap();
            for cluster in clusters {
                if self.cursor.x < self.width
                    && self.cursor.y < self.height
                    && self.current_attribute.is_inverse
                {
                    self.last_inverse_render_pos = Some((self.cursor.x, self.cursor.y));
                }

                let w = cluster.width().clamp(1, 2);
                if self.cursor.x + w > self.width {
                    self.cursor.x = 0;
                    self.index();
                }
                self.grid.put_cell(
                    self.cursor.x,
                    self.cursor.y,
                    Cell {
                        text: cluster,
                        attribute: self.current_attribute.clone(),
                        is_wide_continuation: false,
                    },
                    w,
                    &self.current_attribute.bg,
                );
                self.cursor.x += w;
            }
            self.pending_cluster = last;
        }
    }

    pub fn flush_pending_cluster(&mut self) {
        if self.pending_cluster.is_empty() {
            return;
        }
        let cluster = std::mem::take(&mut self.pending_cluster);
        if self.cursor.x < self.width
            && self.cursor.y < self.height
            && self.current_attribute.is_inverse
        {
            self.last_inverse_render_pos = Some((self.cursor.x, self.cursor.y));
        }
        let w = cluster.width().clamp(1, 2);
        if self.cursor.x + w > self.width {
            self.cursor.x = 0;
            self.index();
        }
        self.grid.put_cell(
            self.cursor.x,
            self.cursor.y,
            Cell {
                text: cluster,
                attribute: self.current_attribute.clone(),
                is_wide_continuation: false,
            },
            w,
            &self.current_attribute.bg,
        );
        self.cursor.x += w;
    }

    pub fn scroll_up(&mut self) {
        let empty = self.get_empty_cell();
        if self.scroll_top == 0 && self.scroll_bottom == self.height.saturating_sub(1) {
            if let Some(line) = self.grid.lines_mut().pop_front() {
                self.scrollback.push(line);
            }
            self.grid.lines_mut().push_back(vec![empty; self.width]);
        } else {
            self.grid.delete_lines(
                self.scroll_top,
                1,
                self.scroll_bottom,
                vec![empty; self.width],
            );
        }
    }

    pub fn scroll_down(&mut self) {
        let empty = self.get_empty_cell();
        self.grid.insert_lines(
            self.scroll_top,
            1,
            self.scroll_bottom,
            vec![empty; self.width],
        );
    }

    pub fn index(&mut self) {
        if self.cursor.y == self.scroll_bottom {
            self.scroll_up();
        } else if self.cursor.y < self.height - 1 {
            self.cursor.y += 1;
        }
    }

    pub fn reverse_index(&mut self) {
        if self.cursor.y == self.scroll_top {
            self.scroll_down();
        } else if self.cursor.y > 0 {
            self.cursor.y -= 1;
        }
    }

    pub fn move_cursor_to_pos(&mut self, r: usize, c: usize) {
        let y = if self.is_origin_mode {
            (self.scroll_top + r).saturating_sub(1)
        } else {
            r.saturating_sub(1)
        };
        self.cursor.y = y.min(self.height.saturating_sub(1));
        self.cursor.x = c.saturating_sub(1).min(self.width.saturating_sub(1));
    }

    pub fn move_cursor_to_col(&mut self, col: usize) {
        self.cursor.x = col.min(self.width.saturating_sub(1));
    }
    pub fn move_cursor_to_row(&mut self, row: usize) {
        self.cursor.y = row.min(self.height.saturating_sub(1));
    }
    pub fn move_cursor_up(&mut self, n: usize) {
        self.cursor.y = self.cursor.y.saturating_sub(n).max(self.scroll_top);
    }
    pub fn move_cursor_down(&mut self, n: usize) {
        self.cursor.y = (self.cursor.y + n).min(self.scroll_bottom);
    }
    pub fn move_cursor_forward(&mut self, n: usize) {
        self.cursor.x = (self.cursor.x + n).min(self.width.saturating_sub(1));
    }
    pub fn move_cursor_backward(&mut self, n: usize) {
        for _ in 0..n {
            if self.cursor.x == 0 {
                break;
            }
            self.cursor.x -= 1;
            if self.grid.lines()[self.cursor.y][self.cursor.x].is_wide_continuation
                && self.cursor.x > 0
            {
                self.cursor.x -= 1;
            }
        }
    }

    pub fn handle_tab(&mut self) {
        let nx = (self.cursor.x / 8 + 1) * 8;
        if nx >= self.width {
            self.cursor.x = 0;
            self.index();
        } else {
            while self.cursor.x < nx {
                let empty = self.get_empty_cell();
                self.grid.put_cell(
                    self.cursor.x,
                    self.cursor.y,
                    empty,
                    1,
                    &self.current_attribute.bg,
                );
                self.cursor.x += 1;
            }
        }
    }

    pub fn insert_lines(&mut self, n: usize) {
        if self.cursor.y < self.scroll_top || self.cursor.y > self.scroll_bottom {
            return;
        }
        let n = n.min(self.scroll_bottom - self.cursor.y + 1);
        let empty = self.get_empty_cell();
        self.grid.insert_lines(
            self.cursor.y,
            n,
            self.scroll_bottom,
            vec![empty; self.width],
        );
    }

    pub fn delete_lines(&mut self, n: usize) {
        if self.cursor.y < self.scroll_top || self.cursor.y > self.scroll_bottom {
            return;
        }
        let n = n.min(self.scroll_bottom - self.cursor.y + 1);
        let empty = self.get_empty_cell();
        self.grid.delete_lines(
            self.cursor.y,
            n,
            self.scroll_bottom,
            vec![empty; self.width],
        );
    }

    pub fn insert_cells(&mut self, n: usize) {
        let n = n.min(self.width - self.cursor.x);
        if n > 0 {
            self.grid.ensure_safe_boundary(
                self.cursor.y,
                self.cursor.x,
                &self.current_attribute.bg,
            );
            let empty = self.get_empty_cell();
            let line = &mut self.grid.lines_mut()[self.cursor.y];
            for _ in 0..n {
                line.insert(self.cursor.x, empty.clone());
                line.pop();
            }
        }
    }

    pub fn delete_cells(&mut self, n: usize) {
        let n = n.min(self.width - self.cursor.x);
        if n == 0 {
            return;
        }
        self.grid
            .ensure_safe_boundary(self.cursor.y, self.cursor.x, &self.current_attribute.bg);
        self.grid.ensure_safe_boundary(
            self.cursor.y,
            (self.cursor.x + n).min(self.width - 1),
            &self.current_attribute.bg,
        );
        let empty = self.get_empty_cell();
        let line = &mut self.grid.lines_mut()[self.cursor.y];
        line.drain(self.cursor.x..(self.cursor.x + n));
        line.extend(std::iter::repeat_n(empty, n));
    }

    pub fn erase_cells(&mut self, n: usize) {
        let empty = self.get_empty_cell();
        self.grid
            .fill_line(self.cursor.y, self.cursor.x, self.cursor.x + n, empty);
    }
    pub fn erase_in_line(&mut self, mode: u8) {
        let empty = self.get_empty_cell();
        match mode {
            0 => self
                .grid
                .fill_line(self.cursor.y, self.cursor.x, self.width, empty),
            1 => self
                .grid
                .fill_line(self.cursor.y, 0, self.cursor.x + 1, empty),
            2 => self.grid.fill_line(self.cursor.y, 0, self.width, empty),
            _ => {}
        }
    }

    pub fn erase_in_display(&mut self, mode: u8) {
        let empty = self.get_empty_cell();
        match mode {
            0 => {
                self.erase_in_line(0);
                for y in (self.cursor.y + 1)..self.height {
                    self.grid.fill_line(y, 0, self.width, empty.clone());
                }
            }
            1 => {
                for y in 0..self.cursor.y {
                    self.grid.fill_line(y, 0, self.width, empty.clone());
                }
                self.erase_in_line(1);
            }
            2 | 3 => {
                for y in 0..self.height {
                    self.grid.fill_line(y, 0, self.width, empty.clone());
                }
            }
            _ => {}
        }
    }

    pub fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        let t = top.saturating_sub(1).min(self.height.saturating_sub(1));
        let b = bottom.saturating_sub(1).min(self.height.saturating_sub(1));
        if t < b {
            self.scroll_top = t;
            self.scroll_bottom = b;
        } else {
            self.scroll_top = 0;
            self.scroll_bottom = self.height.saturating_sub(1);
        }
        self.cursor.y = if self.is_origin_mode {
            self.scroll_top
        } else {
            0
        };
        self.cursor.x = 0;
    }

    pub fn set_origin_mode(&mut self, on: bool) {
        self.is_origin_mode = on;
        self.cursor.y = if on { self.scroll_top } else { 0 };
        self.cursor.x = 0;
    }
    pub fn set_cursor_visible(&mut self, v: bool) {
        self.cursor.is_visible = v;
    }
    pub fn set_cursor_style(&mut self, s: CursorStyle) {
        self.cursor.style = s;
    }
    pub fn set_attribute(&mut self, a: TerminalAttribute) {
        self.current_attribute = a;
    }
    pub fn get_current_attribute(&self) -> &TerminalAttribute {
        &self.current_attribute
    }
    pub fn save_cursor(&mut self) {
        self.saved_cursor = Some((self.cursor.x, self.cursor.y));
    }
    pub fn restore_cursor(&mut self) {
        if let Some((x, y)) = self.saved_cursor {
            self.cursor.y = y.min(self.height.saturating_sub(1));
            self.cursor.x = x.min(self.width.saturating_sub(1));
        }
    }
    pub fn resize(&mut self, w: usize, h: usize) {
        self.grid.resize(w, h);
        self.width = w;
        self.height = h;
        self.scroll_top = 0;
        self.scroll_bottom = h.saturating_sub(1);
        self.cursor.y = self.cursor.y.min(h.saturating_sub(1));
        self.cursor.x = self.cursor.x.min(w.saturating_sub(1));
    }

    pub fn get_line_at_visual_row(&self, visual_row: usize) -> Option<&Vec<Cell>> {
        TerminalHistoryViewEntity::resolve_visual_row_with_logical_row(
            visual_row,
            self.height,
            self.grid.lines(),
            &self.scrollback,
        )
        .map(|(_, line)| line)
    }

    pub fn visual_row_to_logical_row(&self, visual_row: usize) -> usize {
        TerminalHistoryViewEntity::resolve_visual_row_with_logical_row(
            visual_row,
            self.height,
            self.grid.lines(),
            &self.scrollback,
        )
        .map(|(logical_row, _)| logical_row)
        .unwrap_or_else(|| self.get_history_len().saturating_add(visual_row))
    }

    pub fn get_line_at_logical_row(&self, logical_row: usize) -> Option<&Vec<Cell>> {
        let history_len = self.get_history_len();
        if logical_row < history_len {
            self.scrollback.history().get(logical_row)
        } else {
            self.grid
                .lines()
                .get(logical_row.saturating_sub(history_len))
        }
    }

    pub fn get_width(&self) -> usize {
        self.width
    }
    pub fn get_height(&self) -> usize {
        self.height
    }
    pub fn get_history_len(&self) -> usize {
        TerminalHistoryViewEntity::history_len(&self.scrollback)
    }
    pub fn get_viewport_offset(&self) -> usize {
        TerminalHistoryViewEntity::viewport_offset(&self.scrollback)
    }
    pub fn is_cursor_visible(&self) -> bool {
        self.cursor.is_visible
    }
    pub fn get_cursor_pos(&self) -> (usize, usize) {
        (self.cursor.x, self.cursor.y)
    }
    pub fn get_cursor_style(&self) -> CursorStyle {
        self.cursor.style
    }
    pub fn get_ime_anchor_pos(&self) -> (usize, usize) {
        let (x, y) = if self.cursor.is_visible {
            (self.cursor.x, self.cursor.y)
        } else {
            self.last_inverse_render_pos
                .unwrap_or((self.cursor.x, self.cursor.y))
        };
        (
            x.min(self.width.saturating_sub(1)),
            y.min(self.height.saturating_sub(1)),
        )
    }
    pub fn scroll_to(&mut self, o: usize) {
        TerminalHistoryViewEntity::scroll_to(&mut self.scrollback, o);
    }
    pub fn scroll_lines(&mut self, d: isize) {
        TerminalHistoryViewEntity::scroll_lines(&mut self.scrollback, d);
    }
    pub fn reset_viewport(&mut self) {
        TerminalHistoryViewEntity::reset_viewport(&mut self.scrollback);
    }

    pub fn get_mouse_tracking_mode(&self) -> MouseTrackingMode {
        self.mouse_tracking_mode
    }
    pub fn set_mouse_tracking_mode(&mut self, mode: MouseTrackingMode) {
        self.mouse_tracking_mode = mode;
    }
    pub fn is_sgr_mouse_encoding_enabled(&self) -> bool {
        self.use_sgr_mouse_encoding
    }
    pub fn set_sgr_mouse_encoding(&mut self, enabled: bool) {
        self.use_sgr_mouse_encoding = enabled;
    }

    pub fn get_last_mouse_pos(&self) -> Option<(usize, usize)> {
        self.last_mouse_pos
    }
    pub fn set_last_mouse_pos(&mut self, pos: Option<(usize, usize)>) {
        self.last_mouse_pos = pos;
    }

    pub fn get_selection_range(&self) -> SelectionRange {
        self.selection_range
    }
    pub fn set_selection_range(&mut self, range: SelectionRange) {
        self.selection_range = range;
    }

    pub fn get_selected_text(&self) -> String {
        let (start, end) = match normalized_selection_range(self.selection_range) {
            Some(r) => r,
            None => return String::new(),
        };

        let mut selected_text = String::new();
        for logical_row in start.logical_row..=end.logical_row {
            if let Some(line) = self.get_line_at_logical_row(logical_row) {
                for x in 0..self.width {
                    if selection_contains(self.selection_range, x, logical_row)
                        && let Some(cell) = line.get(x)
                        && !cell.is_wide_continuation
                    {
                        selected_text.push_str(&cell.text);
                    }
                }
                if logical_row < end.logical_row {
                    selected_text.push('\n');
                }
            }
        }
        selected_text
    }
}

impl TerminalBufferViewEntity for TerminalBufferEntity {
    fn get_line_at_visual_row(&self, visual_row: usize) -> Option<&Vec<Cell>> {
        TerminalBufferEntity::get_line_at_visual_row(self, visual_row)
    }

    fn visual_row_to_logical_row(&self, visual_row: usize) -> usize {
        TerminalBufferEntity::visual_row_to_logical_row(self, visual_row)
    }

    fn get_width(&self) -> usize {
        TerminalBufferEntity::get_width(self)
    }

    fn get_height(&self) -> usize {
        TerminalBufferEntity::get_height(self)
    }

    fn get_viewport_offset(&self) -> usize {
        TerminalBufferEntity::get_viewport_offset(self)
    }

    fn get_ime_anchor_pos(&self) -> (usize, usize) {
        TerminalBufferEntity::get_ime_anchor_pos(self)
    }

    fn get_cursor_pos(&self) -> (usize, usize) {
        TerminalBufferEntity::get_cursor_pos(self)
    }

    fn get_selection_range(&self) -> SelectionRange {
        TerminalBufferEntity::get_selection_range(self)
    }

    fn is_cursor_visible(&self) -> bool {
        TerminalBufferEntity::is_cursor_visible(self)
    }

    fn get_cursor_style(&self) -> CursorStyle {
        TerminalBufferEntity::get_cursor_style(self)
    }
}

impl TerminalScreenUpdateEntity for TerminalBufferEntity {
    fn print_string(&mut self, text: &str) {
        TerminalBufferEntity::print_string(self, text);
    }

    fn flush_pending_cluster(&mut self) {
        TerminalBufferEntity::flush_pending_cluster(self);
    }

    fn move_cursor_up(&mut self, n: usize) {
        TerminalBufferEntity::move_cursor_up(self, n);
    }

    fn move_cursor_down(&mut self, n: usize) {
        TerminalBufferEntity::move_cursor_down(self, n);
    }

    fn move_cursor_forward(&mut self, n: usize) {
        TerminalBufferEntity::move_cursor_forward(self, n);
    }

    fn move_cursor_backward(&mut self, n: usize) {
        TerminalBufferEntity::move_cursor_backward(self, n);
    }

    fn move_cursor_to_pos(&mut self, row: usize, col: usize) {
        TerminalBufferEntity::move_cursor_to_pos(self, row, col);
    }

    fn move_cursor_to_col(&mut self, col: usize) {
        TerminalBufferEntity::move_cursor_to_col(self, col);
    }

    fn move_cursor_to_row(&mut self, row: usize) {
        TerminalBufferEntity::move_cursor_to_row(self, row);
    }

    fn handle_tab(&mut self) {
        TerminalBufferEntity::handle_tab(self);
    }

    fn index(&mut self) {
        TerminalBufferEntity::index(self);
    }

    fn reverse_index(&mut self) {
        TerminalBufferEntity::reverse_index(self);
    }

    fn insert_lines(&mut self, n: usize) {
        TerminalBufferEntity::insert_lines(self, n);
    }

    fn delete_lines(&mut self, n: usize) {
        TerminalBufferEntity::delete_lines(self, n);
    }

    fn insert_cells(&mut self, n: usize) {
        TerminalBufferEntity::insert_cells(self, n);
    }

    fn delete_cells(&mut self, n: usize) {
        TerminalBufferEntity::delete_cells(self, n);
    }

    fn erase_cells(&mut self, n: usize) {
        TerminalBufferEntity::erase_cells(self, n);
    }

    fn erase_in_line(&mut self, mode: u8) {
        TerminalBufferEntity::erase_in_line(self, mode);
    }

    fn erase_in_display(&mut self, mode: u8) {
        TerminalBufferEntity::erase_in_display(self, mode);
    }

    fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        TerminalBufferEntity::set_scroll_region(self, top, bottom);
    }

    fn scroll_up(&mut self) {
        TerminalBufferEntity::scroll_up(self);
    }

    fn scroll_down(&mut self) {
        TerminalBufferEntity::scroll_down(self);
    }

    fn get_height(&self) -> usize {
        TerminalBufferEntity::get_height(self)
    }

    fn set_origin_mode(&mut self, on: bool) {
        TerminalBufferEntity::set_origin_mode(self, on);
    }

    fn set_cursor_visible(&mut self, visible: bool) {
        TerminalBufferEntity::set_cursor_visible(self, visible);
    }

    fn set_cursor_style(&mut self, style: CursorStyle) {
        TerminalBufferEntity::set_cursor_style(self, style);
    }

    fn set_attribute(&mut self, attribute: TerminalAttribute) {
        TerminalBufferEntity::set_attribute(self, attribute);
    }

    fn get_current_attribute(&self) -> &TerminalAttribute {
        TerminalBufferEntity::get_current_attribute(self)
    }

    fn save_cursor(&mut self) {
        TerminalBufferEntity::save_cursor(self);
    }

    fn restore_cursor(&mut self) {
        TerminalBufferEntity::restore_cursor(self);
    }

    fn set_mouse_tracking_mode(&mut self, mode: MouseTrackingMode) {
        TerminalBufferEntity::set_mouse_tracking_mode(self, mode);
    }

    fn set_sgr_mouse_encoding(&mut self, enabled: bool) {
        TerminalBufferEntity::set_sgr_mouse_encoding(self, enabled);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::terminal_buffer_view_entity::SelectionPoint;

    fn line_text(line: &Vec<Cell>) -> String {
        line.iter()
            .filter(|cell| !cell.is_wide_continuation)
            .map(|cell| cell.text.as_str())
            .collect()
    }

    #[test]
    fn test_print_string_basic() {
        let mut buffer = TerminalBufferEntity::new(10, 5);
        buffer.print_string("Hello");
        buffer.flush_pending_cluster();
        let (x, y) = buffer.get_cursor_pos();
        assert_eq!(x, 5);
        assert_eq!(y, 0);

        let line = buffer.get_line_at_visual_row(0).unwrap();
        let text: String = line.iter().take(5).map(|c| c.text.clone()).collect();
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_print_string_wrap() {
        let mut buffer = TerminalBufferEntity::new(5, 5);
        buffer.print_string("HelloWorld");
        buffer.flush_pending_cluster();
        let (x, y) = buffer.get_cursor_pos();
        assert_eq!(x, 5);
        assert_eq!(y, 1); // "Hello" (5) at row 0, cursor at end. "World" (5) at row 1.
        // Wait, current logic:
        // row 0: H, e, l, l, o (x=5)
        // Next 'W': x+1 > 5? Yes. x=0, index(). row 1: W...
        // After "World", x=5. y=1.

        let line0 = buffer.get_line_at_visual_row(0).unwrap();
        let text0: String = line0.iter().map(|c| c.text.clone()).collect();
        assert_eq!(text0, "Hello");

        let line1 = buffer.get_line_at_visual_row(1).unwrap();
        let text1: String = line1.iter().map(|c| c.text.clone()).collect();
        assert_eq!(text1, "World");
    }

    #[test]
    fn test_print_string_graphemes() {
        let mut buffer = TerminalBufferEntity::new(10, 5);
        // "家族" (Family) emoji is often multiple code points
        buffer.print_string("👨‍👩‍👧‍👦");
        buffer.flush_pending_cluster();
        let (x, y) = buffer.get_cursor_pos();
        assert_eq!(x, 2); // Should be width 2
        assert_eq!(y, 0);

        let line = buffer.get_line_at_visual_row(0).unwrap();
        assert_eq!(line[0].text, "👨‍👩‍👧‍👦");
        assert_eq!(line[1].text, " ");
        assert!(line[1].is_wide_continuation);
    }

    #[test]
    fn test_visual_and_logical_row_mapping_share_viewport_interpretation() {
        let mut buffer = TerminalBufferEntity::new(2, 2);
        buffer.print_string("ABCDEFGH");
        buffer.flush_pending_cluster();

        let expected_rows = [
            (0, ["EF", "GH"]),
            (1, ["CD", "EF"]),
            (2, ["AB", "CD"]),
        ];

        for (offset, expected) in expected_rows {
            buffer.scroll_to(offset);
            for visual_row in 0..buffer.get_height() {
                let visual = buffer.get_line_at_visual_row(visual_row).unwrap();
                let logical_row = buffer.visual_row_to_logical_row(visual_row);
                let logical = buffer.get_line_at_logical_row(logical_row).unwrap();
                assert_eq!(
                    line_text(visual),
                    line_text(logical),
                    "viewport_offset={offset}, visual_row={visual_row}"
                );
                assert_eq!(line_text(visual), expected[visual_row]);
            }
        }
    }

    #[test]
    fn test_selection_range_is_stable_across_scroll_operations() {
        let mut buffer = TerminalBufferEntity::new(2, 2);
        buffer.print_string("ABCDEFGH");
        buffer.flush_pending_cluster();
        buffer.scroll_to(1);

        let start = SelectionPoint {
            x: 0,
            logical_row: buffer.visual_row_to_logical_row(0),
        };
        let end = SelectionPoint {
            x: 1,
            logical_row: buffer.visual_row_to_logical_row(1),
        };
        buffer.set_selection_range(Some((start, end)));

        let selected_before = buffer.get_selected_text();

        buffer.scroll_lines(1);
        assert_eq!(buffer.get_selection_range(), Some((start, end)));
        assert_eq!(buffer.get_selected_text(), selected_before);

        buffer.scroll_to(1);
        assert_eq!(buffer.get_selection_range(), Some((start, end)));
        assert_eq!(buffer.get_selected_text(), selected_before);
    }

    #[test]
    fn test_buffer_view_trait_delegation() {
        let mut buffer = TerminalBufferEntity::new(10, 3);
        buffer.print_string("AB");
        buffer.flush_pending_cluster();

        let view: &dyn TerminalBufferViewEntity = &buffer;
        assert_eq!(view.get_width(), 10);
        assert_eq!(view.get_height(), 3);
        assert_eq!(view.get_cursor_pos(), (2, 0));

        let line = view.get_line_at_visual_row(0).unwrap();
        assert_eq!(line[0].text, "A");
        assert_eq!(line[1].text, "B");
    }
}
