use std::collections::VecDeque;

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
    lines: VecDeque<String>,
    max_lines: usize,
    width: usize,
    height: usize,
    cursor: Cursor,
}

impl TerminalBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut lines = VecDeque::with_capacity(height);
        for _ in 0..height {
            lines.push_back(" ".repeat(width));
        }
        Self {
            lines,
            max_lines: 1000, // スクロールバック用
            width,
            height,
            cursor: Cursor::default(),
        }
    }

    pub fn write_string(&mut self, s: &str) {
        log::debug!("TerminalBuffer::write_string input: {:?}", s);
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                if let Some(&'[') = chars.peek() {
                    chars.next(); // consume '['
                    let mut param_str = String::new();
                    loop {
                        match chars.peek() {
                            Some(&p) if (0x30..=0x3F).contains(&(p as u8)) => {
                                param_str.push(p);
                                chars.next();
                            }
                            _ => break,
                        }
                    }
                    if let Some(cmd) = chars.next() {
                         self.handle_csi(cmd, &param_str);
                    }
                }
            } else {
                self.process_normal_char(c);
            }
        }
    }

    fn handle_csi(&mut self, command: char, params: &str) {
        match command {
            'D' => { // Cursor Back
                let n = params.parse::<usize>().unwrap_or(1);
                if self.cursor.x >= n {
                    self.cursor.x -= n;
                } else {
                    self.cursor.x = 0;
                }
            },
            'C' => { // Cursor Forward
                let n = params.parse::<usize>().unwrap_or(1);
                self.cursor.x = std::cmp::min(self.width - 1, self.cursor.x + n);
            },
            'K' => { // Erase in Line
                 let mode = params.parse::<usize>().unwrap_or(0);
                 if let Some(line) = self.lines.get_mut(self.cursor.y) {
                    let mut chars: Vec<char> = line.chars().collect();
                    // Pad if necessary
                    while chars.len() < self.width {
                        chars.push(' ');
                    }

                    match mode {
                        0 => { // Cursor to end
                            if self.cursor.x < chars.len() {
                                for i in self.cursor.x..chars.len() {
                                    chars[i] = ' ';
                                }
                            }
                        },
                        1 => { // Start to cursor
                            let end = std::cmp::min(self.cursor.x + 1, chars.len());
                            for i in 0..end {
                                chars[i] = ' ';
                            }
                        },
                        2 => { // Whole line
                            for i in 0..chars.len() {
                                chars[i] = ' ';
                            }
                        },
                        _ => {}
                    }
                    *line = chars.into_iter().collect();
                 }
            },
            'P' => { // Delete Character (Shift Left)
                let n = params.parse::<usize>().unwrap_or(1);
                if let Some(line) = self.lines.get_mut(self.cursor.y) {
                     let mut chars: Vec<char> = line.chars().collect();
                     if self.cursor.x < chars.len() {
                         let end_idx = std::cmp::min(self.cursor.x + n, chars.len());
                         chars.drain(self.cursor.x..end_idx);
                         // Pad with spaces at the end
                         for _ in 0..(end_idx - self.cursor.x) {
                             chars.push(' ');
                         }
                         *line = chars.into_iter().collect();
                     }
                }
            },
            'X' => { // Erase Character (Replace with Space)
                let n = params.parse::<usize>().unwrap_or(1);
                if let Some(line) = self.lines.get_mut(self.cursor.y) {
                     let mut chars: Vec<char> = line.chars().collect();
                     // Pad if necessary
                     while chars.len() < self.width {
                         chars.push(' ');
                     }
                     if self.cursor.x < chars.len() {
                         let end_idx = std::cmp::min(self.cursor.x + n, chars.len());
                         for i in self.cursor.x..end_idx {
                             chars[i] = ' ';
                         }
                         *line = chars.into_iter().collect();
                     }
                }
            },
            'H' | 'f' => { // Cursor Position (CUP) - ESC[row;colH or ESC[row;colf
                // Parse row;col format, default to 1;1 if not specified
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
                // Convert from 1-based to 0-based indexing
                self.cursor.y = if row > 0 { row - 1 } else { 0 };
                // Clamp row to valid range
                if self.cursor.y >= self.height {
                    self.cursor.y = self.height - 1;
                }
                // Convert display column to character index (accounting for wide characters)
                let target_display_col = if col > 0 { col - 1 } else { 0 };
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, target_display_col);
                log::debug!("CSI H: params={}, row={}, col={}, target_display_col={}, cursor now at ({}, {})", 
                    params, row, col, target_display_col, self.cursor.x, self.cursor.y);
            },
            'J' => { // Erase in Display
                let mode = params.parse::<usize>().unwrap_or(0);
                match mode {
                    0 => { // Cursor to end of screen
                        // Clear from cursor to end of current line
                        if let Some(line) = self.lines.get_mut(self.cursor.y) {
                            let mut chars: Vec<char> = line.chars().collect();
                            while chars.len() < self.width {
                                chars.push(' ');
                            }
                            for i in self.cursor.x..chars.len() {
                                chars[i] = ' ';
                            }
                            *line = chars.into_iter().collect();
                        }
                        // Clear all lines below cursor
                        for y in (self.cursor.y + 1)..self.lines.len() {
                            if let Some(line) = self.lines.get_mut(y) {
                                *line = " ".repeat(self.width);
                            }
                        }
                    },
                    1 => { // Start of screen to cursor
                        // Clear all lines above cursor
                        for y in 0..self.cursor.y {
                            if let Some(line) = self.lines.get_mut(y) {
                                *line = " ".repeat(self.width);
                            }
                        }
                        // Clear from start of current line to cursor
                        if let Some(line) = self.lines.get_mut(self.cursor.y) {
                            let mut chars: Vec<char> = line.chars().collect();
                            while chars.len() < self.width {
                                chars.push(' ');
                            }
                            for i in 0..=self.cursor.x {
                                if i < chars.len() {
                                    chars[i] = ' ';
                                }
                            }
                            *line = chars.into_iter().collect();
                        }
                    },
                    2 | 3 => { // Entire screen (3 also clears scrollback, but we treat same)
                        for line in self.lines.iter_mut() {
                            *line = " ".repeat(self.width);
                        }
                    },
                    _ => {}
                }
            },
            'h' => { // Set Mode
                if params == "?25" {
                    self.cursor.visible = true;
                    log::debug!("CSI h: Cursor Visible");
                }
            },
            'l' => { // Reset Mode
                if params == "?25" {
                    self.cursor.visible = false;
                    log::debug!("CSI l: Cursor Hidden");
                }
            },
            _ => {
                log::warn!("Unhandled CSI command: {} (params: {})", command, params);
            }
        }
    }

    fn process_normal_char(&mut self, c: char) {
        match c {
            '\r' => self.cursor.x = 0,
            '\n' => {
                self.cursor.x = 0;
                self.cursor.y += 1;
                if self.cursor.y >= self.height {
                    self.scroll_up();
                    self.cursor.y = self.height - 1;
                }
            }
            '\x08' => {
                if self.cursor.x > 0 {
                    self.cursor.x -= 1;
                }
            }
            _ => {
                if self.cursor.x >= self.width {
                    self.cursor.x = 0;
                    self.cursor.y += 1;
                    if self.cursor.y >= self.height {
                        self.scroll_up();
                        self.cursor.y = self.height - 1;
                    }
                }
                
                self.put_char(c);
                self.cursor.x += 1;
            }
        }
    }

    fn put_char(&mut self, c: char) {
        if let Some(line) = self.lines.get_mut(self.cursor.y) {
            let mut chars: Vec<char> = line.chars().collect();
            // Pad if necessary
            while chars.len() < self.width {
                chars.push(' ');
            }

            if self.cursor.x < chars.len() {
                chars[self.cursor.x] = c;
            } else {
                chars.push(c);
            }
            *line = chars.into_iter().collect();
        }
    }

    fn scroll_up(&mut self) {
        self.lines.pop_front();
        self.lines.push_back(" ".repeat(self.width));
    }

    /// Calculate the display width of a character (1 for half-width, 2 for full-width)
    fn char_display_width(c: char) -> usize {
        // Full-width characters: CJK, full-width ASCII, etc.
        // This is a simplified check - a full implementation would use Unicode East Asian Width
        let code = c as u32;
        if (0x1100..=0x115F).contains(&code)   // Hangul Jamo
            || (0x2E80..=0x9FFF).contains(&code)   // CJK
            || (0xAC00..=0xD7A3).contains(&code)   // Hangul Syllables
            || (0xF900..=0xFAFF).contains(&code)   // CJK Compatibility Ideographs
            || (0xFE10..=0xFE1F).contains(&code)   // Vertical forms
            || (0xFE30..=0xFE6F).contains(&code)   // CJK Compatibility Forms
            || (0xFF00..=0xFF60).contains(&code)   // Full-width ASCII
            || (0xFFE0..=0xFFE6).contains(&code)   // Full-width symbols
            || (0x20000..=0x2FFFF).contains(&code) // CJK Extension B and beyond
            || (0x30000..=0x3FFFF).contains(&code) // CJK Extension G and beyond
        {
            2
        } else {
            1
        }
    }

    /// Convert a display column position to a character index in the line
    fn display_col_to_char_index(&self, row: usize, target_display_col: usize) -> usize {
        if let Some(line) = self.lines.get(row) {
            let chars: Vec<char> = line.chars().collect();
            let mut display_col = 0;
            for (char_idx, &c) in chars.iter().enumerate() {
                if display_col >= target_display_col {
                    return char_idx;
                }
                display_col += Self::char_display_width(c);
            }
            // If target is beyond the line, return the length
            chars.len()
        } else {
            target_display_col // Fallback
        }
    }

    pub fn get_lines(&self) -> &VecDeque<String> {
        &self.lines
    }

    pub fn is_cursor_visible(&self) -> bool {
        self.cursor.visible
    }

    pub fn get_cursor_pos(&self) -> (usize, usize) {
        (self.cursor.x, self.cursor.y)
    }

    /// カーソルのピクセル座標を計算する（簡易版：等幅フォント前提）
    pub fn get_cursor_pixel_pos(&self, char_width: i32, char_height: i32) -> (i32, i32) {
        // 現在の描画ロジックに合わせて、単純なグリッド座標から計算
        // 全角文字を考慮する場合、self.cursor.x までの文字の表示幅を合計する必要があるが、
        // 現状の TextOutW の描画と合わせるため、一旦単純な index * width とする。
        let x = self.cursor.x as i32 * char_width;
        let y = self.cursor.y as i32 * char_height;
        (x, y)
    }

    /// バックスペース処理：カーソル位置の前の文字を削除し、カーソルを左に移動
    pub fn backspace(&mut self) {
        if self.cursor.x > 0 {
            self.cursor.x -= 1;
            // カーソル位置の文字をスペースで上書き
            if let Some(line) = self.lines.get_mut(self.cursor.y) {
                let mut chars: Vec<char> = line.chars().collect();
                while chars.len() < self.width {
                    chars.push(' ');
                }
                if self.cursor.x < chars.len() {
                    chars[self.cursor.x] = ' ';
                }
                *line = chars.into_iter().collect();
            }
            log::debug!("TerminalBuffer::backspace: cursor now at ({}, {})", self.cursor.x, self.cursor.y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_initialization() {
        let buffer = TerminalBuffer::new(80, 25);
        assert_eq!(buffer.cursor.x, 0);
        assert_eq!(buffer.cursor.y, 0);
        assert!(buffer.cursor.visible);
    }

    #[test]
    fn test_cursor_visibility() {
        let mut buffer = TerminalBuffer::new(80, 25);

        // Hide cursor
        buffer.write_string("\x1b[?25l");
        assert!(!buffer.is_cursor_visible());

        // Show cursor
        buffer.write_string("\x1b[?25h");
        assert!(buffer.is_cursor_visible());
    }

    #[test]
    fn test_cursor_positioning() {
        let mut buffer = TerminalBuffer::new(80, 25);

        // Move to 10, 20 (1-based)
        buffer.write_string("\x1b[10;20H");
        assert_eq!(buffer.cursor.y, 9);
        assert_eq!(buffer.cursor.x, 19);

        // Move to default (1, 1)
        buffer.write_string("\x1b[H");
        assert_eq!(buffer.cursor.y, 0);
        assert_eq!(buffer.cursor.x, 0);
    }
}
