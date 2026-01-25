use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerminalColor {
    Default,
    Ansi(u8),
    Xterm(u8),
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerminalAttribute {
    pub fg: TerminalColor,
    pub bg: TerminalColor,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
    pub strikethrough: bool,
}

impl Default for TerminalAttribute {
    fn default() -> Self {
        Self {
            fg: TerminalColor::Default,
            bg: TerminalColor::Default,
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            inverse: false,
            strikethrough: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Cell {
    pub c: char,
    pub attribute: TerminalAttribute,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            attribute: TerminalAttribute::default(),
        }
    }
}

pub struct Cursor {
    pub x: usize,
    pub y: usize,
    pub visible: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            visible: true,
        }
    }
}

pub struct TerminalBuffer {
    pub(crate) lines: VecDeque<Vec<Cell>>,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) cursor: Cursor,
    pub(crate) current_attribute: TerminalAttribute,
    pub(crate) scroll_top: usize,    // 0-based, inclusive
    pub(crate) scroll_bottom: usize, // 0-based, inclusive
    pub(crate) saved_cursor: Option<(usize, usize)>,
}

impl TerminalBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut lines = VecDeque::with_capacity(height);
        for _ in 0..height {
            lines.push_back(vec![Cell::default(); width]);
        }
        Self {
            lines,
            width,
            height,
            cursor: Cursor::default(),
            current_attribute: TerminalAttribute::default(),
            scroll_top: 0,
            scroll_bottom: height.saturating_sub(1),
            saved_cursor: None,
        }
    }

    pub(crate) fn scroll_up(&mut self) {
        if self.scroll_top >= self.lines.len() 
            || self.scroll_bottom >= self.lines.len()
            || self.scroll_top > self.scroll_bottom
        {
            self.lines.pop_front();
            self.lines.push_back(vec![Cell::default(); self.width]);
            return;
        }
        self.lines.remove(self.scroll_top);
        self.lines
            .insert(self.scroll_bottom, vec![Cell::default(); self.width]);
    }

    pub fn process_normal_char(&mut self, c: char) {
        match c {
            '\r' => self.cursor.x = 0,
            '\n' => {
                self.cursor.x = 0;
                if self.cursor.y == self.scroll_bottom {
                    self.scroll_up();
                } else if self.cursor.y < self.height - 1 {
                    self.cursor.y += 1;
                }
            }
            '\x08' => {
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                let target_col = if current_col > 0 { current_col - 1 } else { 0 };
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, target_col);
            }
            '\t' => {
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                let next_col = (current_col / 8 + 1) * 8;
                if next_col >= self.width {
                    self.cursor.x = 0;
                    if self.cursor.y == self.scroll_bottom {
                        self.scroll_up();
                    } else if self.cursor.y < self.height - 1 {
                        self.cursor.y += 1;
                    }
                } else {
                    for _ in 0..(next_col - current_col) {
                        self.put_char(' ');
                        self.cursor.x += 1;
                    }
                }
            }
            _ => {
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                let char_width = Self::char_display_width(c);
                if current_col + char_width > self.width {
                    self.cursor.x = 0;
                    if self.cursor.y == self.scroll_bottom {
                        self.scroll_up();
                    } else if self.cursor.y < self.height - 1 {
                        self.cursor.y += 1;
                    }
                }
                self.put_char(c);
                self.cursor.x += 1;
            }
        }
    }

    pub(crate) fn put_char(&mut self, c: char) {
        if let Some(line) = self.lines.get_mut(self.cursor.y) {
            while line.len() < self.width {
                line.push(Cell::default());
            }
            let cell = Cell {
                c,
                attribute: self.current_attribute,
            };
            if self.cursor.x < line.len() {
                line[self.cursor.x] = cell;
            } else {
                line.push(cell);
            }
        }
    }

    pub fn char_display_width(c: char) -> usize {
        let code = c as u32;
        if (0x1100..=0x115F).contains(&code)
            || (0x2E80..=0x9FFF).contains(&code)
            || (0xAC00..=0xD7A3).contains(&code)
            || (0xF900..=0xFAFF).contains(&code)
            || (0xFE10..=0xFE1F).contains(&code)
            || (0xFE30..=0xFE6F).contains(&code)
            || (0xFF00..=0xFF60).contains(&code)
            || (0xFFE0..=0xFFE6).contains(&code)
            || (0x20000..=0x2FFFF).contains(&code)
            || (0x30000..=0x3FFFF).contains(&code)
        {
            2
        } else {
            1
        }
    }

    pub fn get_display_width_up_to(&self, row: usize, char_index: usize) -> usize {
        if let Some(line) = self.lines.get(row) {
            let mut width = 0;
            for (i, cell) in line.iter().enumerate() {
                if i >= char_index {
                    break;
                }
                width += Self::char_display_width(cell.c);
            }
            width
        } else {
            0
        }
    }

    pub(crate) fn display_col_to_char_index(&self, row: usize, target_display_col: usize) -> usize {
        if let Some(line) = self.lines.get(row) {
            let mut display_col = 0;
            for (char_idx, cell) in line.iter().enumerate() {
                if display_col >= target_display_col {
                    return char_idx;
                }
                display_col += Self::char_display_width(cell.c);
            }
            line.len()
        } else {
            target_display_col
        }
    }

    pub fn get_lines(&self) -> &VecDeque<Vec<Cell>> {
        &self.lines
    }
    pub fn is_cursor_visible(&self) -> bool {
        self.cursor.visible
    }
    pub fn get_cursor_pos(&self) -> (usize, usize) {
        (self.cursor.x, self.cursor.y)
    }
    pub fn save_cursor(&mut self) {
        self.saved_cursor = Some((self.cursor.x, self.cursor.y));
    }
    pub fn restore_cursor(&mut self) {
        if let Some((x, y)) = self.saved_cursor {
            self.cursor.y = std::cmp::min(y, self.height.saturating_sub(1));
            if let Some(line) = self.lines.get(self.cursor.y) {
                self.cursor.x = std::cmp::min(x, line.len());
            } else {
                self.cursor.x = 0;
            }
        }
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        for line in &mut self.lines {
            let mut current_width = 0;
            let mut new_cells = Vec::new();
            for cell in line.iter() {
                let w = Self::char_display_width(cell.c);
                if current_width + w > new_width {
                    break;
                }
                current_width += w;
                new_cells.push(*cell);
            }
            while current_width < new_width {
                new_cells.push(Cell::default());
                current_width += 1;
            }
            *line = new_cells;
        }
        self.width = new_width;
        if new_height > self.height {
            for _ in 0..(new_height - self.height) {
                self.lines.push_back(vec![Cell::default(); new_width]);
            }
        } else if new_height < self.height {
            self.lines.truncate(new_height);
        }
        self.height = new_height;
        self.scroll_top = 0;
        self.scroll_bottom = self.height.saturating_sub(1);
        if self.height > 0 {
            self.cursor.y = std::cmp::min(self.cursor.y, self.height.saturating_sub(1));
            if let Some(line) = self.lines.get(self.cursor.y) {
                self.cursor.x = std::cmp::min(self.cursor.x, line.len());
            }
        } else {
            self.cursor.x = 0;
            self.cursor.y = 0;
        }
    }
}
