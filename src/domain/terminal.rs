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
    scroll_top: usize,    // 0-based, inclusive
    scroll_bottom: usize, // 0-based, inclusive
    incomplete_sequence: String,
    saved_cursor: Option<(usize, usize)>,
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

        // [DEBUG] Issue #30: Dump raw input for analysis (Removed for final)
        // log::debug!("RAW INPUT: {:?}", input);

        let char_vec: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < char_vec.len() {
            let c = char_vec[i];
            if c == '\x1b' {
                // If EOF immediately after ESC, save and break
                if i + 1 >= char_vec.len() {
                    self.incomplete_sequence.push(c);
                    break;
                }

                let next_c = char_vec[i + 1];
                if next_c == '[' {
                    // CSI: ESC [ ... cmd
                    let start_idx = i;
                    // Check params and intermediates
                    // Format: ESC [ [parameter bytes] [intermediate bytes] final byte
                    // Parameter bytes: 0x30-0x3F
                    // Intermediate bytes: 0x20-0x2F
                    // Final byte: 0x40-0x7E

                    let mut current_idx = i + 2;
                    let mut complete = false;

                    while current_idx < char_vec.len() {
                        let ch = char_vec[current_idx];
                        let code = ch as u32 as u8;
                        if (0x30..=0x3F).contains(&code) || (0x20..=0x2F).contains(&code) {
                            current_idx += 1;
                        } else if (0x40..=0x7E).contains(&code) {
                            // Found final byte
                            complete = true;
                            break;
                        } else {
                            // Invalid char in CSI, treat as complete to consume
                            complete = true;
                            break;
                        }
                    }

                    if complete {
                        let end_idx = current_idx;
                        let cmd = char_vec[end_idx];

                        // Extract params and intermediates
                        let inner = &char_vec[(i + 2)..end_idx];
                        let param_str: String = inner
                            .iter()
                            .take_while(|&&c| (0x30..=0x3F).contains(&(c as u32 as u8)))
                            .collect();

                        let intermediate_str: String = inner.iter().skip(param_str.len()).collect();

                        self.handle_csi(cmd, &param_str, &intermediate_str);
                        i = end_idx + 1;
                    } else {
                        // Incomplete CSI
                        self.incomplete_sequence = char_vec[start_idx..].iter().collect();
                        break;
                    }
                } else if next_c == ']' {
                    // OSC: ESC ] ... BEL or ESC \
                    let start_idx = i;
                    let mut current_idx = i + 2;
                    let mut found_terminator = false;
                    let mut terminator_len = 0;

                    while current_idx < char_vec.len() {
                        let ch = char_vec[current_idx];
                        if ch == '\x07' {
                            // BEL
                            found_terminator = true;
                            terminator_len = 1;
                            break;
                        } else if ch == '\x1b' {
                            if current_idx + 1 < char_vec.len() {
                                if char_vec[current_idx + 1] == '\\' {
                                    // ESC \
                                    found_terminator = true;
                                    terminator_len = 2;
                                    break;
                                }
                            } else {
                                // ESC at end, might be partial terminator
                                break;
                            }
                        }
                        current_idx += 1;
                    }

                    if found_terminator {
                        let _osc_content: String = char_vec[(i + 2)..current_idx].iter().collect();
                        log::debug!("Ignored OSC sequence");
                        i = current_idx + terminator_len;
                    } else {
                        // Incomplete OSC
                        self.incomplete_sequence = char_vec[start_idx..].iter().collect();
                        break;
                    }
                } else {
                    // Other ESC sequences (e.g. ESC M, ESC 7, ESC 8, ESC c)
                    match next_c {
                        '7' => {
                            self.save_cursor();
                        }
                        '8' => {
                            self.restore_cursor();
                        }
                        _ => {
                            // Assume 2 chars length (ESC + x) for simplicity for now
                            // TODO: Handle other specific sequences if needed
                            log::debug!("Unhandled simple ESC sequence: ESC {}", next_c);
                        }
                    }
                    i += 2;
                }
            } else {
                self.process_normal_char(c);
                i += 1;
            }
        }
    }

    fn handle_csi(&mut self, command: char, params: &str, intermediates: &str) {
        match command {
            'm' => {
                // SGR (Select Graphic Rendition)
                // Issue #30: We need to consume and skip complex SGR sequences like 38;2;R;G;B
                // and 48;2;R;G;B to avoid raw text leakage.
                log::debug!("SGR parameters: {}", params);
            }
            'A' => {
                // Cursor Up
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
                // Cursor Down
                let n = self.parse_csi_param(params, 1);
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                self.cursor.y = std::cmp::min(self.height - 1, self.cursor.y + n);
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, current_col);
            }
            'C' => {
                // Cursor Forward
                let n = self.parse_csi_param(params, 1);
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                let target_col = std::cmp::min(self.width - 1, current_col + n);
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, target_col);
            }
            'D' => {
                // Cursor Back
                let n = self.parse_csi_param(params, 1);
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
                let n = self.parse_csi_param(params, 1);
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
                let n = self.parse_csi_param(params, 1);
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
            'G' => {
                // Cursor Horizontal Absolute (CHA)
                let col = self.parse_csi_param(params, 1);
                let target_display_col = if col > 0 { col - 1 } else { 0 };
                let target_display_col =
                    std::cmp::min(target_display_col, self.width.saturating_sub(1));
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, target_display_col);
            }
            'd' => {
                // Vertical Line Position Absolute (VPA)
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
                // Cursor Next Line (CNL)
                let n = self.parse_csi_param(params, 1);
                self.cursor.y = std::cmp::min(self.height - 1, self.cursor.y + n);
                self.cursor.x = 0;
            }
            'F' => {
                // Cursor Previous Line (CPL)
                let n = self.parse_csi_param(params, 1);
                if self.cursor.y >= n {
                    self.cursor.y -= n;
                } else {
                    self.cursor.y = 0;
                }
                self.cursor.x = 0;
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
            'r' => {
                // DECSTBM - Set Top and Bottom Margins
                log::debug!("DECSTBM (Set Scrolling Region): params={}", params);
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

                // Convert to 0-based
                let top_idx = if top > 0 { top - 1 } else { 0 };
                let bottom_idx = if bottom > 0 { bottom - 1 } else { 0 };

                // Validate and set
                if top_idx < bottom_idx && bottom_idx < self.height {
                    self.scroll_top = top_idx;
                    self.scroll_bottom = bottom_idx;
                    // Cursor is reset to home position (1;1) by DECSTBM spec
                    self.cursor.x = 0;
                    self.cursor.y = 0;
                } else {
                    // Reset to full screen if invalid (or explicitly requested default)
                    self.scroll_top = 0;
                    self.scroll_bottom = self.height.saturating_sub(1);
                    self.cursor.x = 0;
                    self.cursor.y = 0;
                }
                log::debug!(
                    "DECSTBM applied: top={}, bottom={}, cursor reset to (0,0)",
                    self.scroll_top,
                    self.scroll_bottom
                );
            }
            _ => {
                if !intermediates.is_empty() {
                    log::warn!(
                        "Unhandled CSI command: {} (params: {}, intermediates: {})",
                        command,
                        params,
                        intermediates
                    );
                } else {
                    log::warn!("Unhandled CSI command: {} (params: {})", command, params);
                }
            }
        }
    }

    fn process_normal_char(&mut self, c: char) {
        match c {
            '\r' => self.cursor.x = 0,
            '\n' => {
                self.cursor.x = 0;
                // Check if cursor is at the bottom of the scroll region
                if self.cursor.y == self.scroll_bottom {
                    self.scroll_up();
                } else if self.cursor.y < self.height - 1 {
                    self.cursor.y += 1;
                }
            }
            '\x08' => {
                // Backspace: Move cursor back by 1 column
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                let target_col = if current_col > 0 { current_col - 1 } else { 0 };
                self.cursor.x = self.display_col_to_char_index(self.cursor.y, target_col);
            }
            '\t' => {
                // Tab: Move to next tab stop (every 8 columns)
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                let tab_stop = 8;
                let next_col = (current_col / tab_stop + 1) * tab_stop;
                let spaces = next_col - current_col;

                // Check wrapping
                if next_col >= self.width {
                    self.cursor.x = 0;
                    if self.cursor.y == self.scroll_bottom {
                        self.scroll_up();
                    } else if self.cursor.y < self.height - 1 {
                        self.cursor.y += 1;
                    }
                } else {
                    // Fill with spaces? Or just move cursor?
                    // Terminal usually just moves cursor, but buffer needs content.
                    // If we overwrite, we should put spaces.
                    for _ in 0..spaces {
                        self.put_char(' ');
                        self.cursor.x += 1;
                    }
                }
            }
            _ => {
                // Check if adding this character would exceed the width
                let current_col = self.get_display_width_up_to(self.cursor.y, self.cursor.x);
                let char_width = Self::char_display_width(c);

                if current_col + char_width > self.width {
                    self.cursor.x = 0;
                    // Wrap to next line
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
        // Ensure scroll region is valid
        if self.scroll_top >= self.lines.len()
            || self.scroll_bottom >= self.lines.len()
            || self.scroll_top > self.scroll_bottom
        {
            // Fallback to full scroll if invalid state
            self.lines.pop_front();
            self.lines.push_back(" ".repeat(self.width));
            return;
        }

        // Remove line at scroll_top
        self.lines.remove(self.scroll_top);

        // Insert new empty line at scroll_bottom
        // After removal, the indices shift down by 1 for items after scroll_top.
        // So scroll_bottom index effectively points to the slot where the new line should go.
        // Example: 0, 1, 2, 3. Top=1, Bottom=2.
        // Remove 1 -> 0, 2, 3. (Old 2 is now at 1)
        // Insert at 2 -> 0, 2, New, 3. (Correct?)
        // Wait.
        // Original: A, B, C, D. Top=1, Bottom=2. (Scroll region: B, C)
        // Expected: A, C, New, D.
        // Remove at 1 (B): -> A, C, D.
        // Insert at 2: -> A, C, New, D.
        // Yes, insertion at scroll_bottom works because removal at <= scroll_bottom shifts indices.
        self.lines
            .insert(self.scroll_bottom, " ".repeat(self.width));
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

    /// CSIパラメータをパースし、0を1に正規化する。
    fn parse_csi_param(&self, params: &str, default: usize) -> usize {
        let n = params.parse::<usize>().unwrap_or(default);
        if n == 0 {
            1
        } else {
            n
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

    pub fn save_cursor(&mut self) {
        self.saved_cursor = Some((self.cursor.x, self.cursor.y));
        log::debug!("Cursor saved at ({}, {})", self.cursor.x, self.cursor.y);
    }

    pub fn restore_cursor(&mut self) {
        if let Some((x, y)) = self.saved_cursor {
            // 現在の画面サイズに合わせてクリッピング
            self.cursor.y = std::cmp::min(y, self.height.saturating_sub(1));

            if let Some(line) = self.lines.get(self.cursor.y) {
                // 保存時の文字インデックスが現在の行の文字数を超えないようにクランプ
                let char_count = line.chars().count();
                self.cursor.x = std::cmp::min(x, char_count);
            } else {
                self.cursor.x = 0;
            }
            log::debug!("Cursor restored to ({}, {})", self.cursor.x, self.cursor.y);
        }
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

        // Reset scroll region
        self.scroll_top = 0;
        self.scroll_bottom = self.height.saturating_sub(1);

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

    #[test]
    fn test_cha() {
        let mut buffer = TerminalBuffer::new(80, 25);
        buffer.write_string("abcあいう"); // display width: 3 + 6 = 9

        // Move to column 1 (display col 0)
        buffer.write_string("\x1b[1G");
        assert_eq!(buffer.cursor.x, 0);

        // Move to column 4 (display col 3) - start of 'あ'
        buffer.write_string("\x1b[4G");
        assert_eq!(buffer.cursor.x, 3);

        // Move to column 5 (display col 4) - middle of 'あ' (should snap to start of 'あ' or 'い')
        // display_col_to_char_index current implementation returns index that covers the target_display_col.
        // If we target col 4, it should return char_idx 3.
        buffer.write_string("\x1b[5G");
        assert_eq!(buffer.cursor.x, 4); // index of 'い'

        // Out of bounds
        buffer.write_string("\x1b[999G");
        // Row 0 has "abcあいう" (9 display width) + 71 spaces. Total display width 80.
        // target_display_col 998 is clamped to 79.
        // Index 6 is the start of spaces. Index 6 + 70 spaces = 76.
        assert_eq!(buffer.cursor.x, 76);
    }

    #[test]
    fn test_vpa() {
        let mut buffer = TerminalBuffer::new(80, 25);
        buffer.write_string("abc");
        buffer.write_string("\r\n");
        buffer.write_string("あいう"); // Row 1: "あいう" + spaces

        // Move to row 1 (0-based) at column 4 (display col 3)
        // Row 0 is "abc" + spaces. display col 3 is at index 3 (first space).
        buffer.write_string("\x1b[1;4H");
        assert_eq!(buffer.cursor.y, 0);
        assert_eq!(buffer.cursor.x, 3);

        // VPA to row 2 (1-based -> index 1)
        // Row 1 has "あいう". display col 3 corresponds to start of 'う' (index 2).
        buffer.write_string("\x1b[2d");
        assert_eq!(buffer.cursor.y, 1);
        assert_eq!(buffer.cursor.x, 2);

        // Move to column 10 (display col 9) in row 2
        buffer.write_string("\x1b[10G");
        assert_eq!(buffer.cursor.x, 6); // end of "あいう" (3 chars) + 3 spaces = index 6

        // VPA back to row 1
        buffer.write_string("\x1b[1d");
        assert_eq!(buffer.cursor.y, 0);
        assert_eq!(buffer.cursor.x, 9); // "abc" (3 chars) + 6 spaces = index 9
    }

    #[test]
    fn test_cnl_cpl() {
        let mut buffer = TerminalBuffer::new(80, 25);
        buffer.write_string("line1\r\nline2\r\nline3");

        // Move to row 3 (idx 2) at some X
        buffer.write_string("\x1b[3;5H");
        assert_eq!(buffer.cursor.y, 2);
        assert_eq!(buffer.cursor.x, 4);

        // CPL 2 (Previous Line) -> should be Row 1 (idx 0), Col 1 (idx 0)
        buffer.write_string("\x1b[2F");
        assert_eq!(buffer.cursor.y, 0);
        assert_eq!(buffer.cursor.x, 0);

        // CNL 1 (Next Line) -> should be Row 2 (idx 1), Col 1 (idx 0)
        buffer.write_string("\x1b[1E");
        assert_eq!(buffer.cursor.y, 1);
        assert_eq!(buffer.cursor.x, 0);
    }

    #[test]
    fn test_integration_cursor_movements() {
        let mut buffer = TerminalBuffer::new(10, 5); // Small buffer for boundary testing
        buffer.write_string("あいうえお"); // Display width: 10. Row 0: "あいうえお"
        buffer.write_string("\r\n12345"); // Row 1: "12345     "

        // 1. CHA to middle of a CJK character.
        // Visual columns: 'あ' at 1-2, 'い' at 3-4. CHA to column 3 (3G), the first cell of 'い'.
        buffer.write_string("\x1b[1;1H"); // Back to top-left
        buffer.write_string("\x1b[3G"); // To column 3
                                        // In the internal buffer, column 3 (3G) corresponds to idx 1.
        assert_eq!(buffer.cursor.x, 1);

        // 2. VPA maintaining column
        buffer.write_string("\x1b[2d"); // To row 2
        assert_eq!(buffer.cursor.y, 1);
        // Row 1 is "12345". Col 2 is idx 2 ('3').
        assert_eq!(buffer.cursor.x, 2);

        // 3. Boundary Clamping
        buffer.write_string("\x1b[99G"); // Far right
        assert_eq!(buffer.cursor.x, 9); // Clamped to width-1

        buffer.write_string("\x1b[99d"); // Far bottom
        assert_eq!(buffer.cursor.y, 4);

        buffer.write_string("\x1b[0E"); // CNL 0 -> should be treated as 1
        assert_eq!(buffer.cursor.y, 4); // Still bottom
        assert_eq!(buffer.cursor.x, 0); // But at start

        buffer.write_string("\x1b[99F"); // CPL 99 -> top
        assert_eq!(buffer.cursor.y, 0);
        assert_eq!(buffer.cursor.x, 0);
    }

    #[test]
    fn test_csi_with_intermediate_bytes() {
        let mut buffer = TerminalBuffer::new(80, 25);
        // DECSCUSR (Set Cursor Style): ESC [ 0 SP q
        // Should be parsed correctly and NOT print 'q' to the buffer.
        buffer.write_string("\x1b[0 q");

        // Cursor should not move (because 'q' is unhandled, not printed)
        assert_eq!(buffer.cursor.x, 0);

        // Buffer should be empty (spaces)
        let line = buffer.lines.front().unwrap();
        assert!(line.chars().all(|c| c == ' '));
    }

    #[test]
    fn test_save_restore_cursor_basic() {
        let mut buffer = TerminalBuffer::new(80, 25);

        // Move to specific position
        buffer.write_string("\x1b[10;20H"); // Row 9, Col 19
        assert_eq!(buffer.cursor.y, 9);
        assert_eq!(buffer.cursor.x, 19);

        // Save cursor
        buffer.save_cursor();

        // Move elsewhere
        buffer.write_string("\x1b[1;1H");
        assert_eq!(buffer.cursor.y, 0);
        assert_eq!(buffer.cursor.x, 0);

        // Restore cursor
        buffer.restore_cursor();

        // Should be back at 9, 19
        assert_eq!(buffer.cursor.y, 9);
        assert_eq!(buffer.cursor.x, 19);
    }

    #[test]
    fn test_restore_without_save() {
        let mut buffer = TerminalBuffer::new(80, 25);

        // Move to specific position
        buffer.write_string("\x1b[5;5H");
        assert_eq!(buffer.cursor.y, 4);
        assert_eq!(buffer.cursor.x, 4);

        // Restore without saving (should do nothing)
        buffer.restore_cursor();

        assert_eq!(buffer.cursor.y, 4);
        assert_eq!(buffer.cursor.x, 4);
    }

    #[test]
    fn test_restore_cursor_clipping() {
        let mut buffer = TerminalBuffer::new(20, 20);

        // Move to 15, 15 (idx 14, 14)
        buffer.write_string("\x1b[15;15H");
        buffer.save_cursor();

        // Resize smaller than saved position
        buffer.resize(10, 10);

        // Move cursor to home to ensure restore actually does something
        buffer.write_string("\x1b[H");
        assert_eq!(buffer.cursor.x, 0);
        assert_eq!(buffer.cursor.y, 0);

        // Restore
        buffer.restore_cursor();

        // Should be clamped to max indices (9, 9) -> (9, 10) because x can be equal to width
        assert_eq!(buffer.cursor.y, 9);
        assert_eq!(buffer.cursor.x, 10);
    }

    #[test]
    fn test_integration_save_restore() {
        let mut buffer = TerminalBuffer::new(80, 25);

        // Move cursor using CSI
        buffer.write_string("\x1b[5;10H"); // 4, 9
        assert_eq!(buffer.cursor.y, 4);
        assert_eq!(buffer.cursor.x, 9);

        // Save using ESC 7
        buffer.write_string("\x1b7");

        // Move elsewhere
        buffer.write_string("\x1b[1;1H");
        assert_eq!(buffer.cursor.y, 0);
        assert_eq!(buffer.cursor.x, 0);

        // Restore using ESC 8
        buffer.write_string("\x1b8");

        // Check restoration
        assert_eq!(buffer.cursor.y, 4);
        assert_eq!(buffer.cursor.x, 9);
    }
}
