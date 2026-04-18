use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

// 基本型を再エクスポートし、外部からアクセス可能にする
use super::terminal_grid_entity::TerminalGridEntity;
use super::terminal_scrollback_entity::TerminalScrollbackEntity;
pub use super::terminal_types_entity::{
    Cell, Cursor, CursorStyle, TerminalAttribute, TerminalColor,
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
        let mut clusters: Vec<String> = self
            .pending_cluster
            .graphemes(true)
            .map(|s| s.to_string())
            .collect();
        if clusters.len() > 1 {
            let last = clusters.pop().unwrap();
            for cluster in clusters {
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
        let dist = (self.height.saturating_sub(1).saturating_sub(visual_row))
            + self.scrollback.viewport_offset();
        if dist < self.grid.lines().len() {
            self.grid.lines().get(
                self.grid
                    .lines()
                    .len()
                    .saturating_sub(1)
                    .saturating_sub(dist),
            )
        } else {
            self.scrollback.history().get(
                self.scrollback
                    .history()
                    .len()
                    .saturating_sub(1)
                    .saturating_sub(dist.saturating_sub(self.grid.lines().len())),
            )
        }
    }

    pub fn get_width(&self) -> usize {
        self.width
    }
    pub fn get_height(&self) -> usize {
        self.height
    }
    pub fn get_history_len(&self) -> usize {
        self.scrollback.history().len()
    }
    pub fn get_viewport_offset(&self) -> usize {
        self.scrollback.viewport_offset()
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
        self.scrollback.scroll_to(o);
    }
    pub fn scroll_lines(&mut self, d: isize) {
        self.scrollback.scroll_lines(d);
    }
    pub fn reset_viewport(&mut self) {
        self.scrollback.reset_viewport();
    }
}
