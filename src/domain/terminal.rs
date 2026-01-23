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
                } else if let Some(&']') = chars.peek() {
                    // Handle OSC (Operating System Command) \x1b]...\x07 or \x1b]...\x1b\
                    chars.next(); // consume ']'
                    let mut osc_str = String::new();
                    loop {
                        match chars.next() {
                            Some('\x07') => break, // BEL terminator
                            Some('\x1b') => {
                                if let Some(&'\\') = chars.peek() {
                                    chars.next(); // consume '\'
                                    break; // ST terminator
                                }
                            }
                            Some(ch) => osc_str.push(ch),
                            None => break,
                        }
                    }
                    log::debug!("Ignored OSC sequence: {}", osc_str);
                }
            } else {
                self.process_normal_char(c);
            }
        }
    }

    fn handle_csi(&mut self, command: char, params: &str) {
        match command {
            'A' => {
                // Cursor Up
                let n = params.parse::<usize>().unwrap_or(1);
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                if self.cursor.y >= n {
                    self.cursor.y -= n;
                } else {
                    self.cursor.y = 0;
                }
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, current_col);
            }
            'B' => {
                // Cursor Down
                let n = params.parse::<usize>().unwrap_or(1);
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                self.cursor.y = std::cmp::min(self.height - 1, self.cursor.y + n);
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, current_col);
            }
            'C' => {
                // Cursor Forward
                let n = params.parse::<usize>().unwrap_or(1);
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                let target_col = std::cmp::min(self.width - 1, current_col + n);
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, target_col);
            }
            'D' => {
                // Cursor Back
                let n = params.parse::<usize>().unwrap_or(1);
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                let target_col = current_col.saturating_sub(n);
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, target_col);
            }
            'K' => {
                // Erase in Line
                let mode = params.parse::<usize>().unwrap_or(0);
                if let Some(line) = self.lines.get_mut(self.cursor.y) {
                    let mut chars: Vec<char> = line.chars().collect();
                    // Pad if necessary
                    while chars.len() < self.width {
                        chars.push(' ');
                    }

                    match mode {
                        0 => {
                            // Cursor to end
                            if self.cursor.x < chars.len() {
                                for ch in chars.iter_mut().skip(self.cursor.x) {
                                    *ch = ' ';
                                }
                            }
                        }
                        1 => {
                            // Start to cursor
                            let end = std::cmp::min(self.cursor.x + 1, chars.len());
                            for ch in chars.iter_mut().take(end) {
                                *ch = ' ';
                            }
                        }
                        2 => {
                            // Whole line
                            chars.fill(' ');
                        }
                        _ => {}
                    }
                    *line = chars.into_iter().collect();
                }
            }
            'P' => {
                // Delete Character (Shift Left)
                let n = params.parse::<usize>().unwrap_or(1);
                if let Some(line) = self.lines.get_mut(self.cursor.y) {
                    let mut chars: Vec<char> = line.chars().collect();
                    if self.cursor.x < chars.len() {
                        let end_idx = std::cmp::min(self.cursor.x + n, chars.len());
                        let removed_count = end_idx - self.cursor.x;
                        chars.drain(self.cursor.x..end_idx);
                        // Pad with spaces at the end
                        chars.extend(std::iter::repeat_n(' ', removed_count));
                        *line = chars.into_iter().collect();
                    }
                }
            }
            'X' => {
                // Erase Character (Replace with Space)
                let n = params.parse::<usize>().unwrap_or(1);
                if let Some(line) = self.lines.get_mut(self.cursor.y) {
                    let mut chars: Vec<char> = line.chars().collect();
                    // Pad if necessary
                    while chars.len() < self.width {
                        chars.push(' ');
                    }
                    if self.cursor.x < chars.len() {
                        let end_idx = std::cmp::min(self.cursor.x + n, chars.len());
                        for ch in chars.iter_mut().take(end_idx).skip(self.cursor.x) {
                            *ch = ' ';
                        }
                        *line = chars.into_iter().collect();
                    }
                }
            }
            'H' | 'f' => {
                // Cursor Position (CUP) - ESC[row;colH or ESC[row;colf
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
            }
            'J' => {
                // Erase in Display
                let mode = params.parse::<usize>().unwrap_or(0);
                match mode {
                    0 => {
                        // Cursor to end of screen
                        // Clear from cursor to end of current line
                        if let Some(line) = self.lines.get_mut(self.cursor.y) {
                            let mut chars: Vec<char> = line.chars().collect();
                            while chars.len() < self.width {
                                chars.push(' ');
                            }
                            for ch in chars.iter_mut().skip(self.cursor.x) {
                                *ch = ' ';
                            }
                            *line = chars.into_iter().collect();
                        }
                        // Clear all lines below cursor
                        for y in (self.cursor.y + 1)..self.lines.len() {
                            if let Some(line) = self.lines.get_mut(y) {
                                *line = " ".repeat(self.width);
                            }
                        }
                    }
                    1 => {
                        // Start of screen to cursor
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
                    }
                    2 | 3 => {
                        // Entire screen (3 also clears scrollback, but we treat same)
                        for line in self.lines.iter_mut() {
                            *line = " ".repeat(self.width);
                        }
                    }
                    _ => {}
                }
            }
            'h' => {
                // Set Mode
                if params == "?25" {
                    self.cursor.visible = true;
                    log::debug!("CSI h: Cursor Visible");
                }
            }
            'l' => {
                // Reset Mode
                if params == "?25" {
                    self.cursor.visible = false;
                    log::debug!("CSI l: Cursor Hidden");
                }
            }
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
                // Backspace: Move cursor back by 1 column
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                let target_col = if current_col > 0 { current_col - 1 } else { 0 };
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, target_col);
            }
            _ => {
                // Check if adding this character would exceed the width
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                let char_width = Self::char_display_width(c);

                if current_col + char_width > self.width {
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
    pub fn char_display_width(c: char) -> usize {
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
            || (0x30000..=0x3FFFF).contains(&code)
        // CJK Extension G and beyond
        {
            2
        } else {
            1
        }
    }

    pub fn get_display_width_up_to(&self, row: usize, char_index: usize) -> usize {
        if let Some(line) = self.lines.get(row) {
            let mut width = 0;
            for (i, c) in line.chars().enumerate() {
                if i >= char_index {
                    break;
                }
                width += Self::char_display_width(c);
            }
            width
        } else {
            0
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

    /// カーソル位置より前のテキストを取得する（描画幅計算用）
    #[allow(dead_code)]
    pub fn get_text_before_cursor(&self) -> String {
        if let Some(line) = self.lines.get(self.cursor.y) {
            line.chars().take(self.cursor.x).collect()
        } else {
            String::new()
        }
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        // 幅の変更: 各行を調整
        for line in &mut self.lines {
            let mut current_width = 0;
            let mut new_chars = Vec::new();
            for c in line.chars() {
                let w = Self::char_display_width(c);
                if current_width + w > new_width {
                    break;
                }
                current_width += w;
                new_chars.push(c);
            }

            // パディング
            while current_width < new_width {
                new_chars.push(' ');
                current_width += Self::char_display_width(' ');
            }

            *line = new_chars.into_iter().collect();
        }

        self.width = new_width;

        // 高さの変更
        if new_height > self.height {
            // 行を追加
            for _ in 0..(new_height - self.height) {
                self.lines.push_back(" ".repeat(new_width));
            }
        } else if new_height < self.height {
            // 行を削除（末尾を削除して上部を残す）
            self.lines.truncate(new_height);
        }
        self.height = new_height;

        // カーソル位置の調整
        if self.height > 0 {
            // 先にカーソルYを現在の高さにクランプしてから、その行に基づいてXを調整
            self.cursor.y = std::cmp::min(self.cursor.y, self.height.saturating_sub(1));
            if let Some(line) = self.lines.get(self.cursor.y) {
                self.cursor.x = std::cmp::min(self.cursor.x, line.chars().count());
            }
        } else {
            // 高さ0の場合は安全なデフォルト位置にリセット
            self.cursor.x = 0;
            self.cursor.y = 0;
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

    #[test]
    fn test_cursor_movement_relative() {
        let mut buffer = TerminalBuffer::new(80, 25);

        // Move to 5, 5 (1-based -> 4, 4 0-based)
        buffer.write_string("\x1b[5;5H");
        assert_eq!(buffer.cursor.y, 4);
        assert_eq!(buffer.cursor.x, 4);

        // Up 2
        buffer.write_string("\x1b[2A");
        assert_eq!(buffer.cursor.y, 2);

        // Down 1
        buffer.write_string("\x1b[1B");
        assert_eq!(buffer.cursor.y, 3);

        // Right 2
        buffer.write_string("\x1b[2C");
        assert_eq!(buffer.cursor.x, 6);

        // Left 1
        buffer.write_string("\x1b[1D");
        assert_eq!(buffer.cursor.x, 5);
    }

    #[test]
    fn test_terminal_resize() {
        let mut buffer = TerminalBuffer::new(10, 5);
        buffer.write_string("Hello CJKあいう"); // Width: 10. "Hello CJK" is 9. "あ" wraps.
                                                // Line 0: "Hello CJK "
                                                // Line 1: "あいう"
                                                // Cursor at index 3 on Line 1.

        assert_eq!(buffer.cursor.y, 1);
        assert_eq!(buffer.cursor.x, 3);
        assert_eq!(buffer.lines[0], "Hello CJK ");
        assert_eq!(buffer.lines[1], "あいう       ");

        // Resize larger
        buffer.resize(20, 10);
        assert_eq!(buffer.width, 20);
        assert_eq!(buffer.height, 10);
        assert_eq!(buffer.lines[0], "Hello CJK           "); // Padded
        assert_eq!(buffer.lines[1], "あいう              ");
        assert_eq!(buffer.cursor.y, 1);
        assert_eq!(buffer.cursor.x, 3);

        // Resize smaller (truncate)
        buffer.resize(5, 2);
        assert_eq!(buffer.width, 5);
        assert_eq!(buffer.height, 2);
        assert_eq!(buffer.lines[0], "Hello"); // Truncated
                                              // "あいう" is 6 columns, but resize(5) truncates at 5.
                                              // 'あ'(2), 'い'(2) -> 4. 'う'(2) would make 6. So 'う' is truncated.
                                              // Line 1 becomes "あい " (4 cols + 1 space)
        assert_eq!(buffer.lines[1], "あい ");

        // Cursor on line 1 was at x=3.
        // "あい " has 3 characters. x=3 is valid (at end).
        assert_eq!(buffer.cursor.y, 1);
        assert_eq!(buffer.cursor.x, 3);

        // Resize very small (cursor clamp)
        buffer.resize(1, 1);
        // Line 0 was "Hello". Truncated to 1 col -> "H"
        assert_eq!(buffer.lines[0], "H");
        assert_eq!(buffer.cursor.y, 0);
        assert_eq!(buffer.cursor.x, 1); // "H" has 1 char, x can be 1 (at end)

        // Height 0 case
        buffer.resize(0, 0);
        assert_eq!(buffer.height, 0);
        assert_eq!(buffer.cursor.y, 0);
        assert_eq!(buffer.cursor.x, 0);
    }

    #[test]
    fn test_cursor_movement_cjk() {
        let mut buffer = TerminalBuffer::new(80, 25);

        // "あいう" (3 full-width chars)
        buffer.write_string("あいう");
        assert_eq!(buffer.cursor.x, 3); // 3 characters

        // Move back 2 columns (should move back 1 full-width char)
        // Current display pos: 6. Move back 2 -> 4.
        // Chars: 'あ'(2), 'い'(2), 'う'(2). Pos 4 corresponds to start of 'う'.
        // So char index should be 2.
        buffer.write_string("\x1b[2D");
        assert_eq!(buffer.cursor.x, 2);

        // Move forward 2 columns
        buffer.write_string("\x1b[2C");
        assert_eq!(buffer.cursor.x, 3);
    }
}
