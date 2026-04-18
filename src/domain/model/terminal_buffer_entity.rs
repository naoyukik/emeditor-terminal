use std::collections::VecDeque;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

// 基本型を再エクスポートし、外部モジュールからアクセス可能にする
pub use super::terminal_types_entity::{Cell, Cursor, CursorStyle, TerminalAttribute, TerminalColor};

/// ターミナルの文字グリッドおよび状態を管理するドメイン実体
pub struct TerminalBufferEntity {
    lines: VecDeque<Vec<Cell>>,
    width: usize,
    height: usize,
    cursor: Cursor,
    current_attribute: TerminalAttribute,
    scroll_top: usize,
    scroll_bottom: usize,
    is_origin_mode: bool,
    last_inverse_render_pos: Option<(usize, usize)>,
    saved_cursor: Option<(usize, usize)>,

    history: VecDeque<Vec<Cell>>,
    viewport_offset: usize,
    scrollback_limit: usize,
    pending_cluster: String,
}

impl TerminalBufferEntity {
    pub fn new(width: usize, height: usize) -> Self {
        let mut lines = VecDeque::with_capacity(height);
        for _ in 0..height {
            lines.push_back(vec![Cell::default(); width]);
        }
        Self {
            lines, width, height,
            cursor: Cursor::default(),
            current_attribute: TerminalAttribute::default(),
            scroll_top: 0,
            scroll_bottom: height.saturating_sub(1),
            is_origin_mode: false,
            last_inverse_render_pos: None,
            saved_cursor: None,
            history: VecDeque::new(),
            viewport_offset: 0,
            scrollback_limit: 10000,
            pending_cluster: String::new(),
        }
    }

    // --- インターナル ---
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

    fn ensure_safe_boundary(&mut self, y: usize, x: usize) {
        if y >= self.lines.len() || x >= self.width { return; }
        let empty_cell = self.get_empty_cell();
        let line = &mut self.lines[y];
        if line[x].is_wide_continuation {
            if x > 0 { line[x - 1] = empty_cell.clone(); }
            line[x] = empty_cell.clone();
        }
        if line[x].text.width() > 1 && !line[x].is_wide_continuation {
            if x + 1 < line.len() { line[x + 1] = empty_cell.clone(); }
            line[x] = empty_cell.clone();
        }
    }

    fn put_char_at_cursor(&mut self, text: &str, display_width: usize) {
        let (x, y) = (self.cursor.x, self.cursor.y);
        if y >= self.lines.len() || x >= self.width { return; }
        self.ensure_safe_boundary(y, x);
        if display_width == 2 && x + 1 < self.width { self.ensure_safe_boundary(y, x + 1); }
        if let Some(line) = self.lines.get_mut(y) {
            line[x] = Cell { text: text.to_string(), attribute: self.current_attribute.clone(), is_wide_continuation: false };
            if display_width == 2 && x + 1 < line.len() {
                line[x + 1] = Cell { text: " ".to_string(), attribute: self.current_attribute.clone(), is_wide_continuation: true };
            }
        }
    }

    fn push_to_history(&mut self, line: Vec<Cell>) {
        if self.scrollback_limit == 0 { return; }
        if self.history.len() >= self.scrollback_limit { self.history.pop_front(); }
        self.history.push_back(line);
        if self.viewport_offset > 0 {
            self.viewport_offset = (self.viewport_offset + 1).min(self.history.len());
        }
    }

    // --- パブリック API ---
    pub fn print_cell(&mut self, c: char) {
        if self.cursor.x < self.width && self.cursor.y < self.height && self.current_attribute.is_inverse {
            self.last_inverse_render_pos = Some((self.cursor.x, self.cursor.y));
        }
        if c.is_control() && !"\r\n\t\x08".contains(c) { return; }
        self.pending_cluster.push(c);
        let mut clusters: Vec<String> = self.pending_cluster.graphemes(true).map(|s| s.to_string()).collect();
        if clusters.len() > 1 {
            let last = clusters.pop().unwrap();
            for cluster in clusters {
                let display_width = cluster.width().clamp(1, 2);
                if self.cursor.x + display_width > self.width { self.cursor.x = 0; self.index(); }
                self.put_char_at_cursor(&cluster, display_width);
                self.cursor.x += display_width;
            }
            self.pending_cluster = last;
        }
    }

    pub fn flush_pending_cluster(&mut self) {
        if self.pending_cluster.is_empty() { return; }
        let cluster = std::mem::take(&mut self.pending_cluster);
        let display_width = cluster.width().clamp(1, 2);
        if self.cursor.x + display_width > self.width { self.cursor.x = 0; self.index(); }
        self.put_char_at_cursor(&cluster, display_width);
        self.cursor.x += display_width;
    }

    pub fn scroll_up(&mut self) {
        let empty_cell = self.get_empty_cell();
        if self.scroll_top == 0 && self.scroll_bottom == self.height.saturating_sub(1) {
            if let Some(line) = self.lines.pop_front() { self.push_to_history(line); }
            self.lines.push_back(vec![empty_cell; self.width]);
        } else if self.scroll_top < self.lines.len() {
            self.lines.remove(self.scroll_top);
            self.lines.insert(self.scroll_bottom, vec![empty_cell; self.width]);
        }
    }

    pub fn scroll_down(&mut self) {
        let empty_cell = self.get_empty_cell();
        if self.scroll_bottom < self.lines.len() {
            self.lines.remove(self.scroll_bottom);
            self.lines.insert(self.scroll_top, vec![empty_cell; self.width]);
        }
    }

    pub fn index(&mut self) {
        if self.cursor.y == self.scroll_bottom { self.scroll_up(); }
        else if self.cursor.y < self.height - 1 { self.cursor.y += 1; }
    }

    pub fn reverse_index(&mut self) {
        if self.cursor.y == self.scroll_top { self.scroll_down(); }
        else if self.cursor.y > 0 { self.cursor.y -= 1; }
    }

    pub fn move_cursor_to_pos(&mut self, row: usize, col: usize) {
        let row_idx = if self.is_origin_mode { (self.scroll_top + row).saturating_sub(1) } else { row.saturating_sub(1) };
        self.cursor.y = row_idx.min(self.height.saturating_sub(1));
        self.cursor.x = col.saturating_sub(1).min(self.width.saturating_sub(1));
    }

    pub fn move_cursor_to_col(&mut self, col: usize) { self.cursor.x = col.min(self.width.saturating_sub(1)); }
    pub fn move_cursor_to_row(&mut self, row: usize) { self.cursor.y = row.min(self.height.saturating_sub(1)); }
    pub fn move_cursor_up(&mut self, n: usize) { self.cursor.y = self.cursor.y.saturating_sub(n).max(self.scroll_top); }
    pub fn move_cursor_down(&mut self, n: usize) { self.cursor.y = (self.cursor.y + n).min(self.scroll_bottom); }
    pub fn move_cursor_forward(&mut self, n: usize) { self.cursor.x = (self.cursor.x + n).min(self.width.saturating_sub(1)); }
    pub fn move_cursor_backward(&mut self, n: usize) {
        for _ in 0..n {
            if self.cursor.x == 0 { break; }
            self.cursor.x -= 1;
            if self.lines[self.cursor.y][self.cursor.x].is_wide_continuation && self.cursor.x > 0 { self.cursor.x -= 1; }
        }
    }

    pub fn handle_tab(&mut self) {
        let next_x = (self.cursor.x / 8 + 1) * 8;
        if next_x >= self.width { self.cursor.x = 0; self.index(); }
        else { while self.cursor.x < next_x { self.put_char_at_cursor(" ", 1); self.cursor.x += 1; } }
    }

    pub fn insert_lines(&mut self, n: usize) {
        if self.cursor.y < self.scroll_top || self.cursor.y > self.scroll_bottom { return; }
        let n = n.min(self.scroll_bottom - self.cursor.y + 1);
        let empty_cell = self.get_empty_cell();
        for _ in 0..n { self.lines.remove(self.scroll_bottom); self.lines.insert(self.cursor.y, vec![empty_cell.clone(); self.width]); }
    }

    pub fn delete_lines(&mut self, n: usize) {
        if self.cursor.y < self.scroll_top || self.cursor.y > self.scroll_bottom { return; }
        let n = n.min(self.scroll_bottom - self.cursor.y + 1);
        let empty_cell = self.get_empty_cell();
        for _ in 0..n { self.lines.remove(self.cursor.y); self.lines.insert(self.scroll_bottom, vec![empty_cell.clone(); self.width]); }
    }

    pub fn insert_cells(&mut self, n: usize) {
        let (x, y) = (self.cursor.x, self.cursor.y);
        let n = n.min(self.width - x);
        if n == 0 || y >= self.lines.len() { return; }
        self.ensure_safe_boundary(y, x);
        let empty_cell = self.get_empty_cell();
        let line = &mut self.lines[y];
        for _ in 0..n { line.insert(x, empty_cell.clone()); line.pop(); }
        if line[self.width-1].text.width() > 1 && !line[self.width-1].is_wide_continuation { line[self.width-1] = empty_cell; }
    }

    pub fn delete_cells(&mut self, n: usize) {
        let (x, y) = (self.cursor.x, self.cursor.y);
        let n = n.min(self.width - x);
        if n == 0 || y >= self.lines.len() { return; }
        self.ensure_safe_boundary(y, x);
        self.ensure_safe_boundary(y, (x + n).min(self.width - 1));
        let empty_cell = self.get_empty_cell();
        let line = &mut self.lines[y];
        line.drain(x..(x + n));
        line.extend(std::iter::repeat_n(empty_cell, n));
    }

    pub fn erase_cells(&mut self, n: usize) {
        let (x, y) = (self.cursor.x, self.cursor.y);
        let empty_cell = self.get_empty_cell();
        if let Some(line) = self.lines.get_mut(y) {
            let end = (x + n).min(self.width);
            for cell in line.iter_mut().take(end).skip(x) { *cell = empty_cell.clone(); }
        }
    }

    pub fn erase_in_line(&mut self, mode: u8) {
        let empty_cell = self.get_empty_cell();
        if let Some(line) = self.lines.get_mut(self.cursor.y) {
            match mode {
                0 => for cell in line.iter_mut().skip(self.cursor.x) { *cell = empty_cell.clone(); },
                1 => for cell in line.iter_mut().take(self.cursor.x + 1) { *cell = empty_cell.clone(); },
                2 => line.fill(empty_cell),
                _ => {}
            }
        }
    }

    pub fn erase_in_display(&mut self, mode: u8) {
        let empty_cell = self.get_empty_cell();
        match mode {
            0 => { self.erase_in_line(0); for y in (self.cursor.y + 1)..self.height { self.lines[y].fill(empty_cell.clone()); } }
            1 => { for y in 0..self.cursor.y { self.lines[y].fill(empty_cell.clone()); } self.erase_in_line(1); }
            2 | 3 => { for line in self.lines.iter_mut() { line.fill(empty_cell.clone()); } }
            _ => {}
        }
    }

    pub fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        let t = top.saturating_sub(1).min(self.height.saturating_sub(1));
        let b = bottom.saturating_sub(1).min(self.height.saturating_sub(1));
        if t < b { self.scroll_top = t; self.scroll_bottom = b; }
        else { self.scroll_top = 0; self.scroll_bottom = self.height.saturating_sub(1); }
        self.cursor.y = if self.is_origin_mode { self.scroll_top } else { 0 };
        self.cursor.x = 0;
    }

    pub fn set_origin_mode(&mut self, on: bool) {
        self.is_origin_mode = on;
        self.cursor.y = if on { self.scroll_top } else { 0 };
        self.cursor.x = 0;
    }

    pub fn set_cursor_visible(&mut self, visible: bool) { self.cursor.is_visible = visible; }
    pub fn set_cursor_style(&mut self, style: CursorStyle) { self.cursor.style = style; }
    pub fn set_attribute(&mut self, attr: TerminalAttribute) { self.current_attribute = attr; }
    pub fn get_current_attribute(&self) -> &TerminalAttribute { &self.current_attribute }
    pub fn save_cursor(&mut self) { self.saved_cursor = Some((self.cursor.x, self.cursor.y)); }
    pub fn restore_cursor(&mut self) {
        if let Some((x, y)) = self.saved_cursor {
            self.cursor.y = y.min(self.height.saturating_sub(1));
            self.cursor.x = x.min(self.width.saturating_sub(1));
        }
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        for line in &mut self.lines { line.resize(new_width, Cell::default()); }
        self.width = new_width;
        if new_height > self.height { for _ in 0..(new_height - self.height) { self.lines.push_back(vec![Cell::default(); new_width]); } }
        else if new_height < self.height { self.lines.truncate(new_height); }
        self.height = new_height;
        self.scroll_top = 0; self.scroll_bottom = self.height.saturating_sub(1);
        self.cursor.y = self.cursor.y.min(self.height.saturating_sub(1));
        self.cursor.x = self.cursor.x.min(self.width.saturating_sub(1));
    }

    pub fn get_line_at_visual_row(&self, visual_row: usize) -> Option<&Vec<Cell>> {
        let dist_from_bottom = (self.height - 1 - visual_row) + self.viewport_offset;
        if dist_from_bottom < self.lines.len() {
            let idx = self.lines.len() - 1 - dist_from_bottom;
            self.lines.get(idx)
        } else {
            let dist_in_history = dist_from_bottom - self.lines.len();
            if dist_in_history < self.history.len() {
                let idx = self.history.len() - 1 - dist_in_history;
                self.history.get(idx)
            } else { None }
        }
    }

    pub fn get_width(&self) -> usize { self.width }
    pub fn get_height(&self) -> usize { self.height }
    pub fn get_history_len(&self) -> usize { self.history.len() }
    pub fn get_viewport_offset(&self) -> usize { self.viewport_offset }
    pub fn is_cursor_visible(&self) -> bool { self.cursor.is_visible }
    pub fn get_cursor_pos(&self) -> (usize, usize) { (self.cursor.x, self.cursor.y) }
    pub fn get_cursor_style(&self) -> CursorStyle { self.cursor.style }
    pub fn get_ime_anchor_pos(&self) -> (usize, usize) {
        let (x, y) = if self.cursor.is_visible { (self.cursor.x, self.cursor.y) }
        else { self.last_inverse_render_pos.unwrap_or((self.cursor.x, self.cursor.y)) };
        (x.min(self.width.saturating_sub(1)), y.min(self.height.saturating_sub(1)))
    }
    pub fn scroll_to(&mut self, offset: usize) { self.viewport_offset = offset.min(self.history.len()); }
    pub fn scroll_lines(&mut self, delta: isize) {
        let new_offset = (self.viewport_offset as isize + delta).max(0).min(self.history.len() as isize);
        self.viewport_offset = new_offset as usize;
    }
    pub fn reset_viewport(&mut self) { self.viewport_offset = 0; }
}
