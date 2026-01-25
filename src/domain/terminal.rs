use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Default,
    Ansi(u8),
    Rgb(u8, u8, u8),
}

#[derive(Clone, Copy, PartialEq)]
pub struct Cell {
    pub c: char,
    pub fg_color: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg_color: Color::Default,
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
    lines: VecDeque<Vec<Cell>>,
    width: usize,
    height: usize,
    cursor: Cursor,
    current_fg_color: Color,
    scroll_top: usize,    // 0-based, inclusive
    scroll_bottom: usize, // 0-based, inclusive
    incomplete_sequence: String,
    saved_cursor: Option<(usize, usize)>,
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
            current_fg_color: Color::Default,
            scroll_top: 0,
            scroll_bottom: height.saturating_sub(1),
            incomplete_sequence: String::new(),
            saved_cursor: None,
        }
    }

    pub fn write_string(&mut self, s: &str) {
        let input = if !self.incomplete_sequence.is_empty() {
            let mut combined = std::mem::take(&mut self.incomplete_sequence);
            combined.push_str(s);
            combined
        } else {
            s.to_string()
        };

        let char_vec: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < char_vec.len() {
            let c = char_vec[i];
            if c == '\x1b' {
                if i + 1 >= char_vec.len() {
                    self.incomplete_sequence.push(c);
                    break;
                }

                let next_c = char_vec[i + 1];
                if next_c == '[' {
                    let start_idx = i;
                    let mut current_idx = i + 2;
                    let mut complete = false;

                    while current_idx < char_vec.len() {
                        let ch = char_vec[current_idx];
                        let code = ch as u32 as u8;
                        if (0x30..=0x3F).contains(&code) || (0x20..=0x2F).contains(&code) {
                            current_idx += 1;
                        } else {
                            complete = true;
                            break;
                        }
                    }

                    if complete {
                        let end_idx = current_idx;
                        let cmd = char_vec[end_idx];
                        let inner = &char_vec[(i + 2)..end_idx];
                        let param_str: String = inner
                            .iter()
                            .take_while(|&&c| (0x30..=0x3F).contains(&(c as u32 as u8)))
                            .collect();

                        let intermediate_str: String = inner.iter().skip(param_str.len()).collect();

                        self.handle_csi(cmd, &param_str, &intermediate_str);
                        i = end_idx + 1;
                    } else {
                        self.incomplete_sequence = char_vec[start_idx..].iter().collect();
                        break;
                    }
                } else if next_c == ']' {
                    let start_idx = i;
                    let mut current_idx = i + 2;
                    let mut found_terminator = false;
                    let mut terminator_len = 0;

                    while current_idx < char_vec.len() {
                        let ch = char_vec[current_idx];
                        if ch == '\x07' {
                            found_terminator = true;
                            terminator_len = 1;
                            break;
                        } else if ch == '\x1b' {
                            if current_idx + 1 < char_vec.len() {
                                if char_vec[current_idx + 1] == '\\' {
                                    // Corrected from '\'' to '\\'
                                    found_terminator = true;
                                    terminator_len = 2;
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        current_idx += 1;
                    }

                    if found_terminator {
                        i = current_idx + terminator_len;
                    } else {
                        self.incomplete_sequence = char_vec[start_idx..].iter().collect();
                        break;
                    }
                } else {
                    match next_c {
                        '7' => self.save_cursor(),
                        '8' => self.restore_cursor(),
                        _ => {}
                    }
                    i += 2;
                }
            } else {
                self.process_normal_char(c);
                i += 1;
            }
        }
    }

    fn handle_csi(&mut self, command: char, params: &str, _intermediates: &str) {
        match command {
            'm' => {
                if params.is_empty() {
                    self.current_fg_color = Color::Default;
                } else {
                    let parts: Vec<&str> = params.split(';').collect();
                    let mut i = 0;
                    while i < parts.len() {
                        let p = parts[i].parse::<u8>().unwrap_or(0);
                        match p {
                            0 => self.current_fg_color = Color::Default,
                            30..=37 => self.current_fg_color = Color::Ansi(p - 30),
                            38 => {
                                if i + 1 < parts.len() {
                                    let type_p = parts[i + 1].parse::<u8>().unwrap_or(0);
                                    if type_p == 5 && i + 2 < parts.len() {
                                        let color_idx = parts[i + 2].parse::<u8>().unwrap_or(0);
                                        self.current_fg_color = Color::Ansi(color_idx);
                                        i += 2;
                                    } else if type_p == 2 && i + 4 < parts.len() {
                                        let r = parts[i + 2].parse::<u8>().unwrap_or(0);
                                        let g = parts[i + 3].parse::<u8>().unwrap_or(0);
                                        let b = parts[i + 4].parse::<u8>().unwrap_or(0);
                                        self.current_fg_color = Color::Rgb(r, g, b);
                                        i += 4;
                                    }
                                }
                            }
                            39 => self.current_fg_color = Color::Default,
                            90..=97 => self.current_fg_color = Color::Ansi(p - 90 + 8),
                            _ => {}
                        }
                        i += 1;
                    }
                }
            }
            'A' => {
                let n = self.parse_csi_param(params, 1);
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                if self.cursor.y >= n {
                    self.cursor.y -= n;
                } else {
                    self.cursor.y = 0;
                }
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, current_col);
            }
            'B' => {
                let n = self.parse_csi_param(params, 1);
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                self.cursor.y = std::cmp::min(self.height - 1, self.cursor.y + n);
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, current_col);
            }
            'C' => {
                let n = self.parse_csi_param(params, 1);
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                let target_col = std::cmp::min(self.width - 1, current_col + n);
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, target_col);
            }
            'D' => {
                let n = self.parse_csi_param(params, 1);
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                let target_col = current_col.saturating_sub(n);
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, target_col);
            }
            'K' => {
                let mode = params.parse::<usize>().unwrap_or(0);
                if let Some(line) = self.lines.get_mut(self.cursor.y) {
                    while line.len() < self.width {
                        line.push(Cell::default());
                    }
                    match mode {
                        0 => {
                            if self.cursor.x < line.len() {
                                for cell in line.iter_mut().skip(self.cursor.x) {
                                    *cell = Cell::default();
                                }
                            }
                        }
                        1 => {
                            let end = std::cmp::min(self.cursor.x + 1, line.len());
                            for cell in line.iter_mut().take(end) {
                                *cell = Cell::default();
                            }
                        }
                        2 => {
                            line.fill(Cell::default());
                        }
                        _ => {}
                    }
                }
            }
            'P' => {
                let n = self.parse_csi_param(params, 1);
                if let Some(line) = self.lines.get_mut(self.cursor.y) {
                    if self.cursor.x < line.len() {
                        let end_idx = std::cmp::min(self.cursor.x + n, line.len());
                        let removed_count = end_idx - self.cursor.x;
                        line.drain(self.cursor.x..end_idx);
                        line.extend(std::iter::repeat_n(Cell::default(), removed_count));
                    }
                }
            }
            'X' => {
                let n = self.parse_csi_param(params, 1);
                if let Some(line) = self.lines.get_mut(self.cursor.y) {
                    while line.len() < self.width {
                        line.push(Cell::default());
                    }
                    if self.cursor.x < line.len() {
                        let end_idx = std::cmp::min(self.cursor.x + n, line.len());
                        for cell in line.iter_mut().take(end_idx).skip(self.cursor.x) {
                            *cell = Cell::default();
                        }
                    }
                }
            }
            'H' | 'f' => {
                let parts: Vec<&str> = params.split(';').collect();
                let row = if parts.is_empty() || parts[0].is_empty() {
                    1
                } else {
                    parts[0].parse::<usize>().unwrap_or(1)
                };
                let col = if parts.len() < 2 || parts[1].is_empty() {
                    1
                } else {
                    parts[1].parse::<usize>().unwrap_or(1)
                };
                self.cursor.y = if row > 0 { row - 1 } else { 0 };
                if self.cursor.y >= self.height {
                    self.cursor.y = self.height - 1;
                }
                let target_display_col = if col > 0 { col - 1 } else { 0 };
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, target_display_col);
            }
            'J' => {
                let mode = params.parse::<usize>().unwrap_or(0);
                match mode {
                    0 => {
                        if let Some(line) = self.lines.get_mut(self.cursor.y) {
                            while line.len() < self.width {
                                line.push(Cell::default());
                            }
                            for cell in line.iter_mut().skip(self.cursor.x) {
                                *cell = Cell::default();
                            }
                        }
                        for y in (self.cursor.y + 1)..self.lines.len() {
                            if let Some(line) = self.lines.get_mut(y) {
                                line.fill(Cell::default());
                            }
                        }
                    }
                    1 => {
                        for y in 0..self.cursor.y {
                            if let Some(line) = self.lines.get_mut(y) {
                                line.fill(Cell::default());
                            }
                        }
                        if let Some(line) = self.lines.get_mut(self.cursor.y) {
                            while line.len() < self.width {
                                line.push(Cell::default());
                            }
                            for cell in line.iter_mut().take(self.cursor.x + 1) {
                                *cell = Cell::default();
                            }
                        }
                    }
                    2 | 3 => {
                        for line in self.lines.iter_mut() {
                            line.fill(Cell::default());
                        }
                    }
                    _ => {}
                }
            }
            'G' => {
                let col = self.parse_csi_param(params, 1);
                let target_display_col = if col > 0 { col - 1 } else { 0 };
                let target_display_col =
                    std::cmp::min(target_display_col, self.width.saturating_sub(1));
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, target_display_col);
            }
            'd' => {
                let row = self.parse_csi_param(params, 1);
                let current_display_col =
                    self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                self.cursor.y = if row > 0 { row - 1 } else { 0 };
                if self.cursor.y >= self.height {
                    self.cursor.y = self.height - 1;
                }
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, current_display_col);
            }
            'E' => {
                let n = self.parse_csi_param(params, 1);
                self.cursor.y = std::cmp::min(self.height - 1, self.cursor.y + n);
                self.cursor.x = 0;
            }
            'F' => {
                let n = self.parse_csi_param(params, 1);
                if self.cursor.y >= n {
                    self.cursor.y -= n;
                } else {
                    self.cursor.y = 0;
                }
                self.cursor.x = 0;
            }
            'h' => {
                if params == "?25" {
                    self.cursor.visible = true;
                }
            }
            'l' => {
                if params == "?25" {
                    self.cursor.visible = false;
                }
            }
            'r' => {
                let parts: Vec<&str> = params.split(';').collect();
                let top = if parts.is_empty() || parts[0].is_empty() {
                    1
                } else {
                    parts[0].parse::<usize>().unwrap_or(1)
                };
                let bottom = if parts.len() < 2 || parts[1].is_empty() {
                    self.height
                } else {
                    parts[1].parse::<usize>().unwrap_or(self.height)
                };
                let top_idx = if top > 0 { top - 1 } else { 0 };
                let bottom_idx = if bottom > 0 { bottom - 1 } else { 0 };
                if top_idx < bottom_idx && bottom_idx < self.height {
                    self.scroll_top = top_idx;
                    self.scroll_bottom = bottom_idx;
                    self.cursor.x = 0;
                    self.cursor.y = 0;
                } else {
                    self.scroll_top = 0;
                    self.scroll_bottom = self.height.saturating_sub(1);
                    self.cursor.x = 0;
                    self.cursor.y = 0;
                }
            }
            _ => {}
        }
    }

    fn process_normal_char(&mut self, c: char) {
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

    fn put_char(&mut self, c: char) {
        if let Some(line) = self.lines.get_mut(self.cursor.y) {
            while line.len() < self.width {
                line.push(Cell::default());
            }
            let cell = Cell {
                c,
                fg_color: self.current_fg_color,
            };
            if self.cursor.x < line.len() {
                line[self.cursor.x] = cell;
            } else {
                line.push(cell);
            }
        }
    }

    fn scroll_up(&mut self) {
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

    fn display_col_to_char_index(&self, row: usize, target_display_col: usize) -> usize {
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

    fn parse_csi_param(&self, params: &str, default: usize) -> usize {
        let n = params.parse::<usize>().unwrap_or(default);
        if n == 0 {
            1
        } else {
            n
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
    fn test_sgr_colors() {
        let mut buffer = TerminalBuffer::new(80, 25);
        buffer.write_string("\x1b[31mRed\x1b[39mDefault");
        assert_eq!(buffer.lines[0][0].fg_color, Color::Ansi(1));
        assert_eq!(buffer.lines[0][3].fg_color, Color::Default);
        buffer.write_string("\x1b[38;5;208mOrange");
        assert_eq!(buffer.lines[0][10].fg_color, Color::Ansi(208));
        buffer.write_string("\x1b[38;2;255;100;50mRGB");
        assert_eq!(buffer.lines[0][16].fg_color, Color::Rgb(255, 100, 50));
    }

    #[test]
    fn test_terminal_resize() {
        let mut buffer = TerminalBuffer::new(10, 5);
        buffer.write_string("Hello CJKあいう");
        assert_eq!(buffer.cursor.y, 1);
        assert_eq!(line_to_string(&buffer.lines[0]), "Hello CJK ");
        assert_eq!(line_to_string(&buffer.lines[1]), "あいう       ");
        buffer.resize(20, 10);
        assert_eq!(line_to_string(&buffer.lines[0]), "Hello CJK           ");
        buffer.resize(5, 2);
        assert_eq!(line_to_string(&buffer.lines[0]), "Hello");
        assert_eq!(line_to_string(&buffer.lines[1]), "あい ");
    }
}
