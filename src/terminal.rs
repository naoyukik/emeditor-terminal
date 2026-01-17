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
        // 簡易的なエスケープシーケンス除去
        let clean_s = self.strip_ansi(s);
        
        for c in clean_s.chars() {
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
    }

    fn put_char(&mut self, c: char) {
        if let Some(line) = self.lines.get_mut(self.cursor_y) {
            let mut chars: Vec<char> = line.chars().collect();
            if self.cursor_x < chars.len() {
                chars[self.cursor_x] = c;
            } else {
                // 必要に応じて拡張（通常はnewで確保済み）
                chars.push(c);
            }
            *line = chars.into_iter().collect();
        }
    }

    fn scroll_up(&mut self) {
        self.lines.pop_front();
        self.lines.push_back(" ".repeat(self.width));
    }

    fn strip_ansi(&self, s: &str) -> String {
        // 本当に簡易的な実装。ESC [ ... m などを除去
        let mut result = String::new();
        let mut in_escape = false;
        let mut in_csi = false;
        
        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
                continue;
            }
            if in_escape {
                if c == '[' {
                    in_csi = true;
                }
                in_escape = false;
                continue;
            }
            if in_csi {
                if (0x40..=0x7E).contains(&(c as u8)) {
                    in_csi = false;
                }
                continue;
            }
            result.push(c);
        }
        result
    }

    pub fn get_lines(&self) -> &VecDeque<String> {
        &self.lines
    }
}
