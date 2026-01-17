use std::collections::VecDeque;

pub struct TerminalBuffer {
    lines: VecDeque<String>,
    max_lines: usize,
    width: usize,
    height: usize,
    cursor_x: usize,
    cursor_y: usize,
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
            cursor_x: 0,
            cursor_y: 0,
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
                if self.cursor_x >= n {
                    self.cursor_x -= n;
                } else {
                    self.cursor_x = 0;
                }
            },
            'C' => { // Cursor Forward
                let n = params.parse::<usize>().unwrap_or(1);
                self.cursor_x = std::cmp::min(self.width - 1, self.cursor_x + n);
            },
            'K' => { // Erase in Line
                 let mode = params.parse::<usize>().unwrap_or(0);
                 if let Some(line) = self.lines.get_mut(self.cursor_y) {
                    let mut chars: Vec<char> = line.chars().collect();
                    // Pad if necessary
                    while chars.len() < self.width {
                        chars.push(' ');
                    }

                    match mode {
                        0 => { // Cursor to end
                            if self.cursor_x < chars.len() {
                                for i in self.cursor_x..chars.len() {
                                    chars[i] = ' ';
                                }
                            }
                        },
                        1 => { // Start to cursor
                            let end = std::cmp::min(self.cursor_x + 1, chars.len());
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
                if let Some(line) = self.lines.get_mut(self.cursor_y) {
                     let mut chars: Vec<char> = line.chars().collect();
                     if self.cursor_x < chars.len() {
                         let end_idx = std::cmp::min(self.cursor_x + n, chars.len());
                         chars.drain(self.cursor_x..end_idx);
                         // Pad with spaces at the end
                         for _ in 0..(end_idx - self.cursor_x) {
                             chars.push(' ');
                         }
                         *line = chars.into_iter().collect();
                     }
                }
            },
            'X' => { // Erase Character (Replace with Space)
                let n = params.parse::<usize>().unwrap_or(1);
                if let Some(line) = self.lines.get_mut(self.cursor_y) {
                     let mut chars: Vec<char> = line.chars().collect();
                     // Pad if necessary
                     while chars.len() < self.width {
                         chars.push(' ');
                     }
                     if self.cursor_x < chars.len() {
                         let end_idx = std::cmp::min(self.cursor_x + n, chars.len());
                         for i in self.cursor_x..end_idx {
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
                self.cursor_y = if row > 0 { row - 1 } else { 0 };
                // Clamp row to valid range
                if self.cursor_y >= self.height {
                    self.cursor_y = self.height - 1;
                }
                // Convert display column to character index (accounting for wide characters)
                let target_display_col = if col > 0 { col - 1 } else { 0 };
                self.cursor_x = self.display_col_to_char_index(self.cursor_y, target_display_col);
                log::debug!("CSI H: params={}, row={}, col={}, target_display_col={}, cursor now at ({}, {})", 
                    params, row, col, target_display_col, self.cursor_x, self.cursor_y);
            },
            'J' => { // Erase in Display
                let mode = params.parse::<usize>().unwrap_or(0);
                match mode {
                    0 => { // Cursor to end of screen
                        // Clear from cursor to end of current line
                        if let Some(line) = self.lines.get_mut(self.cursor_y) {
                            let mut chars: Vec<char> = line.chars().collect();
                            while chars.len() < self.width {
                                chars.push(' ');
                            }
                            for i in self.cursor_x..chars.len() {
                                chars[i] = ' ';
                            }
                            *line = chars.into_iter().collect();
                        }
                        // Clear all lines below cursor
                        for y in (self.cursor_y + 1)..self.lines.len() {
                            if let Some(line) = self.lines.get_mut(y) {
                                *line = " ".repeat(self.width);
                            }
                        }
                    },
                    1 => { // Start of screen to cursor
                        // Clear all lines above cursor
                        for y in 0..self.cursor_y {
                            if let Some(line) = self.lines.get_mut(y) {
                                *line = " ".repeat(self.width);
                            }
                        }
                        // Clear from start of current line to cursor
                        if let Some(line) = self.lines.get_mut(self.cursor_y) {
                            let mut chars: Vec<char> = line.chars().collect();
                            while chars.len() < self.width {
                                chars.push(' ');
                            }
                            for i in 0..=self.cursor_x {
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
            _ => {
                log::warn!("Unhandled CSI command: {} (params: {})", command, params);
            }
        }
    }

    fn process_normal_char(&mut self, c: char) {
        match c {
            '\r' => self.cursor_x = 0,
            '\n' => {
                self.cursor_x = 0;
                self.cursor_y += 1;
                if self.cursor_y >= self.height {
                    self.scroll_up();
                    self.cursor_y = self.height - 1;
                }
            }
            '\x08' => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            _ => {
                if self.cursor_x >= self.width {
                    self.cursor_x = 0;
                    self.cursor_y += 1;
                    if self.cursor_y >= self.height {
                        self.scroll_up();
                        self.cursor_y = self.height - 1;
                    }
                }
                
                self.put_char(c);
                self.cursor_x += 1;
            }
        }
    }

    fn put_char(&mut self, c: char) {
        if let Some(line) = self.lines.get_mut(self.cursor_y) {
            let mut chars: Vec<char> = line.chars().collect();
            // Pad if necessary
            while chars.len() < self.width {
                chars.push(' ');
            }

            if self.cursor_x < chars.len() {
                chars[self.cursor_x] = c;
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

    /// バックスペース処理：カーソル位置の前の文字を削除し、カーソルを左に移動
    pub fn backspace(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
            // カーソル位置の文字をスペースで上書き
            if let Some(line) = self.lines.get_mut(self.cursor_y) {
                let mut chars: Vec<char> = line.chars().collect();
                while chars.len() < self.width {
                    chars.push(' ');
                }
                if self.cursor_x < chars.len() {
                    chars[self.cursor_x] = ' ';
                }
                *line = chars.into_iter().collect();
            }
            log::debug!("TerminalBuffer::backspace: cursor now at ({}, {})", self.cursor_x, self.cursor_y);
        }
    }
}
