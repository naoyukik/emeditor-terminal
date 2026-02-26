use std::collections::VecDeque;
use vte::{Params, Perform};

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
    pub is_bold: bool,
    pub is_dim: bool,
    pub is_italic: bool,
    pub is_underline: bool,
    pub is_inverse: bool,
    pub is_strikethrough: bool,
}

impl Default for TerminalAttribute {
    fn default() -> Self {
        Self {
            fg: TerminalColor::Default,
            bg: TerminalColor::Default,
            is_bold: false,
            is_dim: false,
            is_italic: false,
            is_underline: false,
            is_inverse: false,
            is_strikethrough: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Cell {
    pub c: char,
    pub attribute: TerminalAttribute,
    pub is_wide_continuation: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            attribute: TerminalAttribute::default(),
            is_wide_continuation: false,
        }
    }
}

pub struct Cursor {
    pub x: usize, // Column (0 to width-1)
    pub y: usize, // Row (0 to height-1)
    pub is_visible: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            is_visible: true,
        }
    }
}

pub struct TerminalBufferEntity {
    lines: VecDeque<Vec<Cell>>,
    width: usize,
    height: usize,
    cursor: Cursor,
    current_attribute: TerminalAttribute,
    scroll_top: usize,
    scroll_bottom: usize,
    is_origin_mode: bool,
    saved_cursor: Option<(usize, usize)>,

    history: VecDeque<Vec<Cell>>,
    viewport_offset: usize,
    scrollback_limit: usize,
}

impl TerminalBufferEntity {
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
            is_origin_mode: false,
            saved_cursor: None,
            history: VecDeque::new(),
            viewport_offset: 0,
            scrollback_limit: 10000,
        }
    }

    pub(crate) fn get_empty_cell(&self) -> Cell {
        Cell {
            c: ' ',
            attribute: TerminalAttribute {
                fg: TerminalColor::Default,
                bg: self.current_attribute.bg,
                ..TerminalAttribute::default()
            },
            is_wide_continuation: false,
        }
    }

    pub(crate) fn scroll_up(&mut self) {
        let empty_cell = self.get_empty_cell();
        if self.scroll_top >= self.lines.len()
            || self.scroll_bottom >= self.lines.len()
            || self.scroll_top > self.scroll_bottom
        {
            if let Some(line) = self.lines.pop_front() {
                self.push_to_history(line);
            }
            self.lines.push_back(vec![empty_cell; self.width]);
            return;
        }

        if self.scroll_top == 0 {
            if let Some(line) = self.lines.remove(0) {
                self.push_to_history(line);
            }
        } else {
            self.lines.remove(self.scroll_top);
        }

        self.lines
            .insert(self.scroll_bottom, vec![empty_cell; self.width]);
    }

    pub(crate) fn scroll_down(&mut self) {
        let empty_cell = self.get_empty_cell();
        if self.scroll_top >= self.lines.len()
            || self.scroll_bottom >= self.lines.len()
            || self.scroll_top > self.scroll_bottom
        {
            self.lines.pop_back();
            self.lines.push_front(vec![empty_cell; self.width]);
            return;
        }

        self.lines.remove(self.scroll_bottom);
        self.lines
            .insert(self.scroll_top, vec![empty_cell; self.width]);
    }

    pub(crate) fn reverse_index(&mut self) {
        if self.cursor.y == self.scroll_top {
            self.scroll_down();
        } else if self.cursor.y > 0 {
            self.cursor.y -= 1;
        }
    }

    pub(crate) fn index(&mut self) {
        if self.cursor.y == self.scroll_bottom {
            self.scroll_up();
        } else if self.cursor.y < self.height - 1 {
            self.cursor.y += 1;
        }
    }

    pub(crate) fn insert_lines(&mut self, n: usize) {
        let n = std::cmp::min(n, self.scroll_bottom.saturating_sub(self.cursor.y) + 1);
        if self.cursor.y < self.scroll_top || self.cursor.y > self.scroll_bottom {
            return;
        }
        let empty_cell = self.get_empty_cell();
        for _ in 0..n {
            self.lines.remove(self.scroll_bottom);
            self.lines
                .insert(self.cursor.y, vec![empty_cell; self.width]);
        }
    }

    pub(crate) fn delete_lines(&mut self, n: usize) {
        let n = std::cmp::min(n, self.scroll_bottom.saturating_sub(self.cursor.y) + 1);
        if self.cursor.y < self.scroll_top || self.cursor.y > self.scroll_bottom {
            return;
        }
        let empty_cell = self.get_empty_cell();
        for _ in 0..n {
            self.lines.remove(self.cursor.y);
            self.lines
                .insert(self.scroll_bottom, vec![empty_cell; self.width]);
        }
    }

    pub(crate) fn insert_cells(&mut self, n: usize) {
        let empty_cell = self.get_empty_cell();
        if let Some(line) = self.lines.get_mut(self.cursor.y) {
            let n = std::cmp::min(n, self.width.saturating_sub(self.cursor.x));
            if n == 0 {
                return;
            }
            for _ in 0..n {
                line.insert(self.cursor.x, empty_cell);
                line.pop();
            }
        }
    }

    fn push_to_history(&mut self, line: Vec<Cell>) {
        if self.scrollback_limit == 0 {
            return;
        }
        if self.history.len() >= self.scrollback_limit {
            self.history.pop_front();
        }
        self.history.push_back(line);
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
                self.index();
            }
            '\x08' => {
                if self.cursor.x > 0 {
                    self.cursor.x -= 1;
                }
            }
            '\t' => {
                let next_x = (self.cursor.x / 8 + 1) * 8;
                if next_x >= self.width {
                    self.cursor.x = 0;
                    self.index();
                } else {
                    while self.cursor.x < next_x {
                        self.put_char(' ');
                        self.cursor.x += 1;
                    }
                }
            }
            _ => {
                let char_width = Self::char_display_width(c);
                if self.cursor.x + char_width > self.width {
                    // TUI apps usually don't rely on auto-wrap, but we handle it just in case
                    self.cursor.x = 0;
                    self.index();
                }
                self.put_char(c);
                self.cursor.x += char_width;
            }
        }
    }

    pub(crate) fn put_char(&mut self, c: char) {
        let char_width = Self::char_display_width(c);
        let x = self.cursor.x;
        let y = self.cursor.y;

        if y >= self.lines.len() || x >= self.width {
            return;
        }

        let empty_cell = self.get_empty_cell();
        if let Some(line) = self.lines.get_mut(y) {
            if x >= line.len() {
                return;
            }

            // Clear existing wide char parts if we overwrite them
            if x > 0 && line.get(x).is_some_and(|cell| cell.is_wide_continuation) {
                if let Some(prev) = line.get_mut(x - 1) {
                    *prev = empty_cell;
                }
            }
            if let Some(current) = line.get(x) {
                if Self::char_display_width(current.c) == 2 && x + 1 < line.len() {
                    if let Some(next) = line.get_mut(x + 1) {
                        *next = empty_cell;
                    }
                }
            }

            if let Some(cell) = line.get_mut(x) {
                *cell = Cell {
                    c,
                    attribute: self.current_attribute,
                    is_wide_continuation: false,
                };
            }

            if char_width == 2 && x + 1 < line.len() {
                if let Some(next) = line.get_mut(x + 1) {
                    *next = Cell {
                        c: ' ',
                        attribute: self.current_attribute,
                        is_wide_continuation: true,
                    };
                }
            }
        }
    }

    pub fn char_display_width(c: char) -> usize {
        let code = c as u32;
        // East Asian Wide / Fullwidth characters → 2 columns
        // Box Drawing (0x2500-0x257F) is NOT included (stays at 1 column)
        if (0x1100..=0x115F).contains(&code) // Hangul Jamo
            || (0x2E80..=0x9FFF).contains(&code) // CJK Unified Ideographs
            || (0xAC00..=0xD7A3).contains(&code) // Hangul Syllables
            || (0xF900..=0xFAFF).contains(&code) // CJK Compatibility Ideographs
            || (0xFE10..=0xFE1F).contains(&code) // Vertical forms
            || (0xFE30..=0xFE6F).contains(&code) // CJK Compatibility Forms
            || (0xFF00..=0xFF60).contains(&code) // Fullwidth Forms
            || (0xFFE0..=0xFFE6).contains(&code) // Fullwidth Symbols
            || (0x20000..=0x3FFFF).contains(&code)
        // Plane 2 & 3
        {
            2
        } else {
            1
        }
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
            } else {
                None
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_lines(&self) -> &VecDeque<Vec<Cell>> {
        &self.lines
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_history_len(&self) -> usize {
        self.history.len()
    }

    pub fn get_viewport_offset(&self) -> usize {
        self.viewport_offset
    }

    pub fn is_cursor_visible(&self) -> bool {
        self.cursor.is_visible
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
            self.cursor.x = std::cmp::min(x, self.width.saturating_sub(1));
        }
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        for line in &mut self.lines {
            line.resize(new_width, Cell::default());
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
        self.cursor.y = std::cmp::min(self.cursor.y, self.height.saturating_sub(1));
        self.cursor.x = std::cmp::min(self.cursor.x, self.width.saturating_sub(1));
    }
}

impl Perform for TerminalBufferEntity {
    fn print(&mut self, c: char) {
        self.process_normal_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x08 => {
                // BS
                if self.cursor.x > 0 {
                    self.cursor.x -= 1;
                }
            }
            0x09 => {
                // TAB
                let next_x = (self.cursor.x / 8 + 1) * 8;
                if next_x >= self.width {
                    self.cursor.x = 0;
                    self.index();
                } else {
                    while self.cursor.x < next_x {
                        self.put_char(' ');
                        self.cursor.x += 1;
                    }
                }
            }
            0x0A..=0x0C => {
                // LF, VT, FF
                self.cursor.x = 0;
                self.index();
            }
            0x0D => {
                // CR
                self.cursor.x = 0;
            }
            _ => {} // Ignore other control characters (BEL, etc.) to prevent garbage rendering
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        // No-op for now
    }

    fn put(&mut self, _byte: u8) {
        // No-op for now
    }

    fn unhook(&mut self) {
        // No-op for now
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        // No-op for now
    }

    fn csi_dispatch(
        &mut self,
        params: &Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        match action {
            'm' => self.handle_sgr(params),
            'A' => {
                let n = self.get_param(params, 0, 1);
                self.cursor.y = self.cursor.y.saturating_sub(n as usize);
            }
            'B' => {
                let n = self.get_param(params, 0, 1);
                self.cursor.y =
                    std::cmp::min(self.height.saturating_sub(1), self.cursor.y + n as usize);
            }
            '@' => {
                let n = self.get_param(params, 0, 1);
                self.insert_cells(n as usize);
            }
            'C' => {
                let n = self.get_param(params, 0, 1);
                self.cursor.x =
                    std::cmp::min(self.width.saturating_sub(1), self.cursor.x + n as usize);
            }
            'D' => {
                let n = self.get_param(params, 0, 1);
                self.cursor.x = self.cursor.x.saturating_sub(n as usize);
            }
            'K' => {
                let mode = self.get_param(params, 0, 0);
                let empty_cell = self.get_empty_cell();
                if let Some(line) = self.lines.get_mut(self.cursor.y) {
                    match mode {
                        0 => {
                            for cell in line.iter_mut().skip(self.cursor.x) {
                                *cell = empty_cell;
                            }
                        }
                        1 => {
                            let end = std::cmp::min(self.cursor.x + 1, self.width);
                            for cell in line.iter_mut().take(end) {
                                *cell = empty_cell;
                            }
                        }
                        2 => {
                            line.fill(empty_cell);
                        }
                        _ => {}
                    }
                }
            }
            'P' => {
                let n = self.get_param(params, 0, 1);
                let empty_cell = self.get_empty_cell();
                if let Some(line) = self.lines.get_mut(self.cursor.y) {
                    let x = self.cursor.x;
                    if x < self.width {
                        let end_idx = std::cmp::min(x + n as usize, self.width);
                        let removed_count = end_idx - x;
                        line.drain(x..end_idx);
                        line.extend(std::iter::repeat_n(empty_cell, removed_count));
                    }
                }
            }
            'X' => {
                let n = self.get_param(params, 0, 1);
                let empty_cell = self.get_empty_cell();
                if let Some(line) = self.lines.get_mut(self.cursor.y) {
                    let end_idx = std::cmp::min(self.cursor.x + n as usize, self.width);
                    for cell in line.iter_mut().take(end_idx).skip(self.cursor.x) {
                        *cell = empty_cell;
                    }
                }
            }
            'H' | 'f' => {
                let row = self.get_param(params, 0, 1) as usize;
                let col = self.get_param(params, 1, 1) as usize;

                let target_row = if self.is_origin_mode {
                    (self.scroll_top + row).saturating_sub(1)
                } else {
                    row.saturating_sub(1)
                };
                self.cursor.y = std::cmp::min(self.height.saturating_sub(1), target_row);
                self.cursor.x = std::cmp::min(self.width.saturating_sub(1), col.saturating_sub(1));
            }
            'J' => {
                let mode = self.get_param(params, 0, 0);
                let empty_cell = self.get_empty_cell();
                match mode {
                    0 => {
                        if let Some(line) = self.lines.get_mut(self.cursor.y) {
                            for cell in line.iter_mut().skip(self.cursor.x) {
                                *cell = empty_cell;
                            }
                        }
                        for y in (self.cursor.y + 1)..self.height {
                            if let Some(line) = self.lines.get_mut(y) {
                                line.fill(empty_cell);
                            }
                        }
                    }
                    1 => {
                        for y in 0..self.cursor.y {
                            if let Some(line) = self.lines.get_mut(y) {
                                line.fill(empty_cell);
                            }
                        }
                        if let Some(line) = self.lines.get_mut(self.cursor.y) {
                            let end = std::cmp::min(self.cursor.x + 1, self.width);
                            for cell in line.iter_mut().take(end) {
                                *cell = empty_cell;
                            }
                        }
                    }
                    2 | 3 => {
                        for line in self.lines.iter_mut() {
                            line.fill(empty_cell);
                        }
                    }
                    _ => {}
                }
            }
            'G' => {
                let col = self.get_param(params, 0, 1) as usize;
                self.cursor.x = std::cmp::min(self.width.saturating_sub(1), col.saturating_sub(1));
            }
            'd' => {
                let row = self.get_param(params, 0, 1) as usize;
                self.cursor.y = std::cmp::min(self.height.saturating_sub(1), row.saturating_sub(1));
            }
            'E' => {
                let n = self.get_param(params, 0, 1) as usize;
                self.cursor.y = std::cmp::min(self.height.saturating_sub(1), self.cursor.y + n);
                self.cursor.x = 0;
            }
            'F' => {
                let n = self.get_param(params, 0, 1) as usize;
                self.cursor.y = self.cursor.y.saturating_sub(n);
                self.cursor.x = 0;
            }
            'h' => {
                if _intermediates.first() == Some(&b'?') {
                    let mode = self.get_param(params, 0, 0);
                    match mode {
                        6 => {
                            self.is_origin_mode = true;
                            self.cursor.y = self.scroll_top;
                            self.cursor.x = 0;
                        }
                        25 => self.cursor.is_visible = true,
                        _ => {}
                    }
                }
            }
            'l' => {
                if _intermediates.first() == Some(&b'?') {
                    let mode = self.get_param(params, 0, 0);
                    match mode {
                        6 => {
                            self.is_origin_mode = false;
                            self.cursor.y = 0;
                            self.cursor.x = 0;
                        }
                        25 => self.cursor.is_visible = false,
                        _ => {}
                    }
                }
            }
            'r' => {
                let top = self.get_param(params, 0, 1) as usize;
                let bottom = self.get_param(params, 1, self.height as u16) as usize;

                let top_idx = top.saturating_sub(1);
                let bottom_idx = bottom.saturating_sub(1);
                if top_idx < bottom_idx && bottom_idx < self.height {
                    self.scroll_top = top_idx;
                    self.scroll_bottom = bottom_idx;
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
            'S' => {
                let n = self.get_param(params, 0, 1) as usize;
                for _ in 0..n {
                    self.scroll_up();
                }
            }
            'T' => {
                let n = self.get_param(params, 0, 1) as usize;
                for _ in 0..n {
                    self.scroll_down();
                }
            }
            'L' => {
                let n = self.get_param(params, 0, 1) as usize;
                self.insert_lines(n);
            }
            'M' => {
                let n = self.get_param(params, 0, 1) as usize;
                self.delete_lines(n);
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
        match byte as char {
            '7' => self.save_cursor(),
            '8' => self.restore_cursor(),
            'M' => self.reverse_index(),
            'D' => self.index(),
            _ => {}
        }
    }
}

impl TerminalBufferEntity {
    fn get_param(&self, params: &Params, index: usize, default: u16) -> u16 {
        params
            .iter()
            .nth(index)
            .and_then(|p| p.first())
            .copied()
            .unwrap_or(default)
    }

    fn handle_sgr(&mut self, params: &Params) {
        if params.is_empty() {
            self.current_attribute = TerminalAttribute::default();
            return;
        }

        let mut iter = params.iter();
        while let Some(subparams) = iter.next() {
            let p = subparams.first().copied().unwrap_or(0);
            match p {
                0 => self.current_attribute = TerminalAttribute::default(),
                1 => self.current_attribute.is_bold = true,
                2 => self.current_attribute.is_dim = true,
                3 => self.current_attribute.is_italic = true,
                4 => self.current_attribute.is_underline = true,
                7 => self.current_attribute.is_inverse = true,
                9 => self.current_attribute.is_strikethrough = true,
                22 => {
                    self.current_attribute.is_bold = false;
                    self.current_attribute.is_dim = false;
                }
                23 => self.current_attribute.is_italic = false,
                24 => self.current_attribute.is_underline = false,
                27 => self.current_attribute.is_inverse = false,
                29 => self.current_attribute.is_strikethrough = false,
                30..=37 => self.current_attribute.fg = TerminalColor::Ansi((p - 30) as u8),
                38 => {
                    // Extended foreground color
                    if let Some(type_p) = subparams.get(1).copied() {
                        match type_p {
                            5 => {
                                if let Some(color_idx) = subparams.get(2).copied() {
                                    self.current_attribute.fg =
                                        TerminalColor::Xterm(color_idx as u8);
                                }
                            }
                            2 => {
                                if subparams.len() >= 5 {
                                    let r = subparams[2] as u8;
                                    let g = subparams[3] as u8;
                                    let b = subparams[4] as u8;
                                    self.current_attribute.fg = TerminalColor::Rgb(r, g, b);
                                }
                            }
                            _ => {}
                        }
                    } else if let Some(next_subparams) = iter.next() {
                        // Legacy semicolon separated format (38;5;n)
                        let type_p = next_subparams.first().copied().unwrap_or(0);
                        match type_p {
                            5 => {
                                if let Some(color_sub) = iter.next() {
                                    if let Some(color_idx) = color_sub.first().copied() {
                                        self.current_attribute.fg =
                                            TerminalColor::Xterm(color_idx as u8);
                                    }
                                }
                            }
                            2 => {
                                let r = iter.next().and_then(|s| s.first()).copied();
                                let g = iter.next().and_then(|s| s.first()).copied();
                                let b = iter.next().and_then(|s| s.first()).copied();
                                if let (Some(r), Some(g), Some(b)) = (r, g, b) {
                                    self.current_attribute.fg =
                                        TerminalColor::Rgb(r as u8, g as u8, b as u8);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                39 => self.current_attribute.fg = TerminalColor::Default,
                40..=47 => self.current_attribute.bg = TerminalColor::Ansi((p - 40) as u8),
                48 => {
                    // Extended background color
                    if let Some(type_p) = subparams.get(1).copied() {
                        match type_p {
                            5 => {
                                if let Some(color_idx) = subparams.get(2).copied() {
                                    self.current_attribute.bg =
                                        TerminalColor::Xterm(color_idx as u8);
                                }
                            }
                            2 => {
                                if subparams.len() >= 5 {
                                    let r = subparams[2] as u8;
                                    let g = subparams[3] as u8;
                                    let b = subparams[4] as u8;
                                    self.current_attribute.bg = TerminalColor::Rgb(r, g, b);
                                }
                            }
                            _ => {}
                        }
                    } else if let Some(next_subparams) = iter.next() {
                        let type_p = next_subparams.first().copied().unwrap_or(0);
                        match type_p {
                            5 => {
                                if let Some(color_sub) = iter.next() {
                                    if let Some(color_idx) = color_sub.first().copied() {
                                        self.current_attribute.bg =
                                            TerminalColor::Xterm(color_idx as u8);
                                    }
                                }
                            }
                            2 => {
                                let r = iter.next().and_then(|s| s.first()).copied();
                                let g = iter.next().and_then(|s| s.first()).copied();
                                let b = iter.next().and_then(|s| s.first()).copied();
                                if let (Some(r), Some(g), Some(b)) = (r, g, b) {
                                    self.current_attribute.bg =
                                        TerminalColor::Rgb(r as u8, g as u8, b as u8);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                49 => self.current_attribute.bg = TerminalColor::Default,
                90..=97 => self.current_attribute.fg = TerminalColor::Ansi((p - 90 + 8) as u8),
                100..=107 => self.current_attribute.bg = TerminalColor::Ansi((p - 100 + 8) as u8),
                _ => {}
            }
        }
    }
}
