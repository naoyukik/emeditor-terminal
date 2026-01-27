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

    // Scrollback support
    pub(crate) history: VecDeque<Vec<Cell>>,
    pub(crate) viewport_offset: usize, // 0 = at bottom (normal view), >0 = scrolled up
    pub(crate) scrollback_limit: usize,
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
            history: VecDeque::new(),
            viewport_offset: 0,
            scrollback_limit: 10000, // Default 10,000 lines
        }
    }

    pub(crate) fn scroll_up(&mut self) {
        // If scroll region is invalid or full screen
        if self.scroll_top >= self.lines.len()
            || self.scroll_bottom >= self.lines.len()
            || self.scroll_top > self.scroll_bottom
        {
            if let Some(line) = self.lines.pop_front() {
                self.push_to_history(line);
            }
            self.lines.push_back(vec![Cell::default(); self.width]);
            return;
        }

        // If top of scroll region is 0, the line being removed goes to history
        if self.scroll_top == 0 {
            if let Some(line) = self.lines.remove(0) {
                self.push_to_history(line);
            }
        } else {
            // Otherwise it's just deleted (middle of screen scroll)
            self.lines.remove(self.scroll_top);
        }

        self.lines
            .insert(self.scroll_bottom, vec![Cell::default(); self.width]);
    }

    fn push_to_history(&mut self, line: Vec<Cell>) {
        if self.scrollback_limit == 0 {
            return;
        }
        if self.history.len() >= self.scrollback_limit {
            self.history.pop_front();
        }
        self.history.push_back(line);

        // Maintain viewport position relative to content if scrolled up
        if self.viewport_offset > 0 {
            self.viewport_offset += 1;
            if self.viewport_offset > self.history.len() {
                self.viewport_offset = self.history.len();
            }
        }
    }

    pub fn scroll_to(&mut self, offset: usize) {
        let max_scroll = self.history.len();
        self.viewport_offset = std::cmp::min(offset, max_scroll);
    }

    pub fn scroll_lines(&mut self, delta: isize) {
        let current = self.viewport_offset as isize;
        let new_offset = current + delta;
        let max_scroll = self.history.len() as isize;

        if new_offset < 0 {
            self.viewport_offset = 0;
        } else if new_offset > max_scroll {
            self.viewport_offset = max_scroll as usize;
        } else {
            self.viewport_offset = new_offset as usize;
        }
    }

    pub fn reset_viewport(&mut self) {
        self.viewport_offset = 0;
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

    pub fn get_line_at_visual_row(&self, visual_row: usize) -> Option<&Vec<Cell>> {
        // visual_row: 0 is top of screen, height-1 is bottom

        // Distance from the very last line of the active buffer
        let dist_from_bottom = (self.height - 1 - visual_row) + self.viewport_offset;

        if dist_from_bottom < self.lines.len() {
            // It's in the active buffer
            let idx = self.lines.len() - 1 - dist_from_bottom;
            self.lines.get(idx)
        } else {
            // It's in history
            let dist_in_history = dist_from_bottom - self.lines.len();
            if dist_in_history < self.history.len() {
                let idx = self.history.len() - 1 - dist_in_history;
                self.history.get(idx)
            } else {
                None
            }
        }
    }

    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    fn line_to_string(line: &[Cell]) -> String {
        line.iter().map(|cell| cell.c).collect()
    }

    #[test]
    fn test_cursor_initialization() {
        let buffer = TerminalBuffer::new(80, 25);
        assert_eq!(buffer.cursor.x, 0);
        assert_eq!(buffer.cursor.y, 0);
        assert!(buffer.cursor.visible);
    }

    #[test]
    fn test_scrollback_history() {
        let mut buffer = TerminalBuffer::new(10, 3);
        buffer.scrollback_limit = 5;

        // Line 1
        buffer.put_char('1');
        buffer.process_normal_char('\n');
        // Line 2
        buffer.put_char('2');
        buffer.process_normal_char('\n');
        // Line 3
        buffer.put_char('3');
        buffer.process_normal_char('\n');
        // Line 4 (now line 1 is in history)
        buffer.put_char('4');

        assert_eq!(buffer.history.len(), 1);
        assert_eq!(line_to_string(&buffer.history[0]).trim(), "1");
        assert_eq!(line_to_string(&buffer.lines[0]).trim(), "2");

        // Fill more to overflow limit
        for i in 5..=10 {
            buffer.process_normal_char('\n');
            buffer.put_char(std::char::from_digit(i as u32, 16).unwrap_or('X'));
        }

        // History limit is 5.
        assert_eq!(buffer.history.len(), 5);
        // Last line in buffer is 'a' (10).
        // Lines in buffer: ['8', '9', 'a']
        // History should contain: ['3', '4', '5', '6', '7']
        assert_eq!(line_to_string(&buffer.history[4]).trim(), "7");
        assert_eq!(line_to_string(&buffer.lines[0]).trim(), "8");
    }

    #[test]
    fn test_viewport_logic() {
        let mut buffer = TerminalBuffer::new(10, 3);
        buffer.put_char('A');
        buffer.process_normal_char('\n');
        buffer.put_char('B');
        buffer.process_normal_char('\n');
        buffer.put_char('C');
        buffer.process_normal_char('\n'); // 'A' goes to history
        buffer.put_char('D');

        // History: ["A"], Lines: ["B", "C", "D"]

        // Normal view (offset 0)
        assert_eq!(line_to_string(buffer.get_line_at_visual_row(0).unwrap()).trim(), "B");
        assert_eq!(line_to_string(buffer.get_line_at_visual_row(2).unwrap()).trim(), "D");

        // Scroll up 1 (offset 1) -> Should show A, B, C
        buffer.scroll_lines(1);
        assert_eq!(buffer.viewport_offset, 1);
        assert_eq!(line_to_string(buffer.get_line_at_visual_row(0).unwrap()).trim(), "A");
        assert_eq!(line_to_string(buffer.get_line_at_visual_row(2).unwrap()).trim(), "C");

        // Scroll back down
        buffer.scroll_lines(-1);
        assert_eq!(buffer.viewport_offset, 0);
        assert_eq!(line_to_string(buffer.get_line_at_visual_row(0).unwrap()).trim(), "B");
    }
}
