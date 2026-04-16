use std::collections::VecDeque;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use vte::{Params, Perform};

#[derive(Debug, Clone, PartialEq)]
pub enum TerminalColor {
    Default,
    Ansi(u8),
    Xterm(u8),
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Clone, PartialEq, Debug)]
pub struct Cell {
    pub text: String,
    pub attribute: TerminalAttribute,
    pub is_wide_continuation: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            text: " ".to_string(),
            attribute: TerminalAttribute::default(),
            is_wide_continuation: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorStyle {
    BlinkingBlock,
    SteadyBlock,
    BlinkingUnderline,
    SteadyUnderline,
    BlinkingBar,
    SteadyBar,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self::BlinkingBar
    }
}

pub struct Cursor {
    pub x: usize, // Column (0 to width-1)
    pub y: usize, // Row (0 to height-1)
    pub is_visible: bool,
    pub style: CursorStyle,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            is_visible: true,
            style: CursorStyle::default(),
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
    last_inverse_render_pos: Option<(usize, usize)>,
    saved_cursor: Option<(usize, usize)>,

    history: VecDeque<Vec<Cell>>,
    viewport_offset: usize,
    scrollback_limit: usize,

    /// 確定待ちの書記素クラスターバッファ
    pending_cluster: String,
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
            last_inverse_render_pos: None,
            saved_cursor: None,
            history: VecDeque::new(),
            viewport_offset: 0,
            scrollback_limit: 10000,
            pending_cluster: String::new(),
        }
    }

    pub(crate) fn get_empty_cell(&self) -> Cell {
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
                .insert(self.cursor.y, vec![empty_cell.clone(); self.width]);
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
                .insert(self.scroll_bottom, vec![empty_cell.clone(); self.width]);
        }
    }

    pub(crate) fn insert_cells(&mut self, n: usize) {
        let x = self.cursor.x;
        let y = self.cursor.y;
        if y >= self.lines.len() || x >= self.width {
            return;
        }

        let n = std::cmp::min(n, self.width - x);
        if n == 0 {
            return;
        }

        self.ensure_safe_boundary(y, x);

        let empty_cell = self.get_empty_cell();
        if let Some(line) = self.lines.get_mut(y) {
            for _ in 0..n {
                line.insert(x, empty_cell.clone());
                line.pop();
            }
            if self.width > 0 {
                let last_idx = self.width - 1;
                // レビュー指摘修正: width() > 1 でワイド文字（クランプされた絵文字等を含む）を判定
                if line[last_idx].text.width() > 1 && !line[last_idx].is_wide_continuation {
                    line[last_idx] = empty_cell.clone();
                }
            }
        }
    }

    pub(crate) fn delete_cells(&mut self, n: usize) {
        let x = self.cursor.x;
        let y = self.cursor.y;
        if y >= self.lines.len() || x >= self.width {
            return;
        }

        let n = std::cmp::min(n, self.width - x);
        if n == 0 {
            return;
        }

        self.ensure_safe_boundary(y, x);
        self.ensure_safe_boundary(y, x + n);

        let empty_cell = self.get_empty_cell();
        if let Some(line) = self.lines.get_mut(y) {
            line.drain(x..(x + n));
            line.extend(std::iter::repeat_n(empty_cell, n));
        }
    }

    fn ensure_safe_boundary(&mut self, y: usize, x: usize) {
        if y >= self.lines.len() || x >= self.width {
            return;
        }
        let empty_cell = self.get_empty_cell();
        let line = &mut self.lines[y];

        if line[x].is_wide_continuation {
            if x > 0 {
                line[x - 1] = empty_cell.clone();
            }
            line[x] = empty_cell.clone();
        }

        // レビュー指摘修正: width() > 1 でワイド文字（クランプされた絵文字等を含む）を判定
        if line[x].text.width() > 1 && !line[x].is_wide_continuation {
            if x + 1 < line.len() {
                line[x + 1] = empty_cell.clone();
            }
            line[x] = empty_cell.clone();
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

    /// 制御文字やエスケープシーケンスの開始時に、バッファに残っている書記素を強制的に出力する
    pub(crate) fn flush_pending_cluster(&mut self) {
        if self.pending_cluster.is_empty() {
            return;
        }
        let cluster = std::mem::take(&mut self.pending_cluster);
        self.write_cluster_to_grid(&cluster);
    }

    fn write_cluster_to_grid(&mut self, cluster: &str) {
        // unicode-width による過大な幅（Emoji ZWJ 等）を 1〜2 カラムに制限する
        let char_width = cluster.width();
        let display_width = char_width.clamp(1, 2);

        if self.cursor.x + display_width > self.width {
            self.cursor.x = 0;
            self.index();
        }
        self.put_char(cluster, display_width);
        self.cursor.x += display_width;
    }

    pub fn process_normal_char(&mut self, c: char) {
        if c.is_control() && c != '\r' && c != '\n' && c != '\t' && c != '\x08' {
            return;
        }

        self.pending_cluster.push(c);

        // unicode-segmentation を用いた正規の書記素クラスター判定
        let mut clusters: Vec<String> = self
            .pending_cluster
            .graphemes(true)
            .map(|s| s.to_string())
            .collect();

        // 次の文字が来るまで確定できない可能性があるため、2つ以上のクラスターがある場合に前方を確定させる
        if clusters.len() > 1 {
            let last = clusters.pop().unwrap();
            for cluster in clusters {
                self.write_cluster_to_grid(&cluster);
            }
            self.pending_cluster = last;
        }
    }

    pub(crate) fn move_cursor_backward(&mut self, n: usize) {
        for _ in 0..n {
            if self.cursor.x == 0 {
                break;
            }
            self.cursor.x -= 1;
            // 継続セルに乗った場合の正規化（1カラムずつ戻るが、論理文字の途中ならさらに戻る）
            if let Some(line) = self.lines.get(self.cursor.y) {
                if let Some(cell) = line.get(self.cursor.x) {
                    if cell.is_wide_continuation && self.cursor.x > 0 {
                        self.cursor.x -= 1;
                    }
                }
            }
        }
    }

    pub(crate) fn move_cursor_forward(&mut self, n: usize) {
        for _ in 0..n {
            if self.cursor.x >= self.width.saturating_sub(1) {
                break;
            }
            self.cursor.x += 1;
            // ANSI CUF はカラム単位。継続セルに乗った場合、それが論理的に分断されていれば後の put_char で修復される
        }
    }

    pub(crate) fn put_char(&mut self, text: &str, display_width: usize) {
        let x = self.cursor.x;
        let y = self.cursor.y;

        if y >= self.lines.len() || x >= self.width {
            return;
        }

        self.ensure_safe_boundary(y, x);
        if display_width == 2 && x + 1 < self.width {
            self.ensure_safe_boundary(y, x + 1);
        }

        if let Some(line) = self.lines.get_mut(y) {
            if let Some(cell) = line.get_mut(x) {
                *cell = Cell {
                    text: text.to_string(),
                    attribute: self.current_attribute.clone(),
                    is_wide_continuation: false,
                };
            }

            if display_width == 2 && x + 1 < line.len() {
                if let Some(next) = line.get_mut(x + 1) {
                    *next = Cell {
                        text: " ".to_string(),
                        attribute: self.current_attribute.clone(),
                        is_wide_continuation: true,
                    };
                }
            }
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

    #[allow(dead_code)]
    pub(crate) fn get_lines(&self) -> &VecDeque<Vec<Cell>> {
        &self.lines
    }

    pub fn is_cursor_visible(&self) -> bool {
        self.cursor.is_visible
    }
    pub fn get_cursor_pos(&self) -> (usize, usize) {
        (self.cursor.x, self.cursor.y)
    }
    /// Returns the best guess for the IME input position.
    /// - If the hardware cursor is VISIBLE, we trust it absolutely (standard behavior).
    /// - If it's HIDDEN, we check if we have a "logical" cursor position from the last
    ///   inverse-video render (typical for TUI apps like Gemini CLI).
    pub fn get_ime_anchor_pos(&self) -> (usize, usize) {
        if self.cursor.is_visible {
            (self.cursor.x, self.cursor.y)
        } else {
            self.last_inverse_render_pos
                .unwrap_or((self.cursor.x, self.cursor.y))
        }
    }
    pub fn get_cursor_style(&self) -> CursorStyle {
        self.cursor.style
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
        if self.cursor.x < self.width && self.cursor.y < self.height {
            if self.current_attribute.is_inverse {
                self.last_inverse_render_pos = Some((self.cursor.x, self.cursor.y));
            }
        }
        self.process_normal_char(c);
    }

    fn execute(&mut self, byte: u8) {
        self.flush_pending_cluster();
        match byte {
            0x08 => self.move_cursor_backward(1),
            0x09 => {
                let next_x = (self.cursor.x / 8 + 1) * 8;
                if next_x >= self.width {
                    self.cursor.x = 0;
                    self.index();
                } else {
                    while self.cursor.x < next_x {
                        self.put_char(" ", 1);
                        self.cursor.x += 1;
                    }
                }
            }
            0x0A..=0x0C => {
                self.cursor.x = 0;
                self.index();
            }
            0x0D => self.cursor.x = 0,
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        self.flush_pending_cluster();
    }
    fn put(&mut self, _byte: u8) {
        self.flush_pending_cluster();
    }
    fn unhook(&mut self) {
        self.flush_pending_cluster();
    }
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        self.flush_pending_cluster();
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, action: char) {
        self.flush_pending_cluster();
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
                self.move_cursor_forward(n as usize);
            }
            'D' => {
                let n = self.get_param(params, 0, 1);
                self.move_cursor_backward(n as usize);
            }
            'K' => {
                let mode = self.get_param(params, 0, 0);
                let empty_cell = self.get_empty_cell();
                if let Some(line) = self.lines.get_mut(self.cursor.y) {
                    match mode {
                        0 => {
                            for cell in line.iter_mut().skip(self.cursor.x) {
                                *cell = empty_cell.clone();
                            }
                        }
                        1 => {
                            let end = std::cmp::min(self.cursor.x + 1, self.width);
                            for cell in line.iter_mut().take(end) {
                                *cell = empty_cell.clone();
                            }
                        }
                        2 => line.fill(empty_cell),
                        _ => {}
                    }
                }
            }
            'P' => {
                let n = self.get_param(params, 0, 1);
                self.delete_cells(n as usize);
            }
            'X' => {
                let n = self.get_param(params, 0, 1);
                let empty_cell = self.get_empty_cell();
                if let Some(line) = self.lines.get_mut(self.cursor.y) {
                    let end_idx = std::cmp::min(self.cursor.x + n as usize, self.width);
                    for cell in line.iter_mut().take(end_idx).skip(self.cursor.x) {
                        *cell = empty_cell.clone();
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
                let old_x = self.cursor.x;
                let old_y = self.cursor.y;
                self.cursor.y = std::cmp::min(self.height.saturating_sub(1), target_row);
                self.cursor.x = std::cmp::min(self.width.saturating_sub(1), col.saturating_sub(1));
                if self.cursor.x != old_x || self.cursor.y != old_y {
                    log::debug!(
                        "Cursor moved via CUP: ({}, {}) -> ({}, {})",
                        old_x,
                        old_y,
                        self.cursor.x,
                        self.cursor.y
                    );
                }
            }
            'J' => {
                let mode = self.get_param(params, 0, 0);
                let empty_cell = self.get_empty_cell();
                match mode {
                    0 => {
                        if let Some(line) = self.lines.get_mut(self.cursor.y) {
                            for cell in line.iter_mut().skip(self.cursor.x) {
                                *cell = empty_cell.clone();
                            }
                        }
                        for y in (self.cursor.y + 1)..self.height {
                            if let Some(line) = self.lines.get_mut(y) {
                                line.fill(empty_cell.clone());
                            }
                        }
                    }
                    1 => {
                        for y in 0..self.cursor.y {
                            if let Some(line) = self.lines.get_mut(y) {
                                line.fill(empty_cell.clone());
                            }
                        }
                        if let Some(line) = self.lines.get_mut(self.cursor.y) {
                            let end = std::cmp::min(self.cursor.x + 1, self.width);
                            for cell in line.iter_mut().take(end) {
                                *cell = empty_cell.clone();
                            }
                        }
                    }
                    2 | 3 => {
                        for line in self.lines.iter_mut() {
                            line.fill(empty_cell.clone());
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
                if intermediates.first() == Some(&b'?') {
                    for subparams in params.iter() {
                        for mode in subparams.iter() {
                            match mode {
                                6 => {
                                    self.is_origin_mode = true;
                                    self.cursor.y = self.scroll_top;
                                    self.cursor.x = 0;
                                    log::debug!("Origin Mode: ON at (0, {})", self.scroll_top);
                                }
                                25 => {
                                    self.cursor.is_visible = true;
                                    log::debug!(
                                        "Cursor set to VISIBLE at ({}, {})",
                                        self.cursor.x,
                                        self.cursor.y
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            'l' => {
                if intermediates.first() == Some(&b'?') {
                    for subparams in params.iter() {
                        for mode in subparams.iter() {
                            match mode {
                                6 => {
                                    self.is_origin_mode = false;
                                    self.cursor.y = 0;
                                    self.cursor.x = 0;
                                    log::debug!("Origin Mode: OFF");
                                }
                                25 => {
                                    self.cursor.is_visible = false;
                                    log::debug!(
                                        "Cursor set to HIDDEN (Last pos: {}, {})",
                                        self.cursor.x,
                                        self.cursor.y
                                    );
                                }
                                _ => {}
                            }
                        }
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
            'q' => {
                if intermediates.first() == Some(&b' ') {
                    self.handle_decscusr(params);
                }
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
        self.flush_pending_cluster();
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

    fn handle_decscusr(&mut self, params: &Params) {
        let n = params
            .iter()
            .last()
            .and_then(|subparams| subparams.first())
            .copied()
            .unwrap_or(1);
        self.cursor.style = match n {
            0 | 1 => CursorStyle::BlinkingBlock,
            2 => CursorStyle::SteadyBlock,
            3 => CursorStyle::BlinkingUnderline,
            4 => CursorStyle::SteadyUnderline,
            5 => CursorStyle::BlinkingBar,
            6 => CursorStyle::SteadyBar,
            _ => CursorStyle::BlinkingBlock,
        };
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
                7 => self.current_attribute.is_inverse = true,
                27 => self.current_attribute.is_inverse = false,

                9 => self.current_attribute.is_strikethrough = true,
                22 => {
                    self.current_attribute.is_bold = false;
                    self.current_attribute.is_dim = false;
                }
                23 => self.current_attribute.is_italic = false,
                7 => self.current_attribute.is_inverse = true,
                27 => self.current_attribute.is_inverse = false,

                29 => self.current_attribute.is_strikethrough = false,
                30..=37 => self.current_attribute.fg = TerminalColor::Ansi((p - 30) as u8),
                38 => {
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
                                    self.current_attribute.fg = TerminalColor::Rgb(
                                        subparams[2] as u8,
                                        subparams[3] as u8,
                                        subparams[4] as u8,
                                    );
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
                                    self.current_attribute.bg = TerminalColor::Rgb(
                                        subparams[2] as u8,
                                        subparams[3] as u8,
                                        subparams[4] as u8,
                                    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use vte::Parser;

    #[test]
    fn test_decscusr_handling() {
        let mut buffer = TerminalBufferEntity::new(80, 24);
        let mut parser = Parser::new();

        // Default should be BlinkingBar
        assert_eq!(buffer.cursor.style, CursorStyle::BlinkingBar);

        // CSI SP q (no params) -> BlinkingBlock (default to 1)
        let seq = b"\x1b[ q";
        parser.advance(&mut buffer, seq);
        assert_eq!(buffer.cursor.style, CursorStyle::BlinkingBlock);

        // CSI 2 SP q -> SteadyBlock
        let seq = b"\x1b[2 q";
        parser.advance(&mut buffer, seq);
        assert_eq!(buffer.cursor.style, CursorStyle::SteadyBlock);

        // CSI 4 SP q -> SteadyUnderline
        let seq = b"\x1b[4 q";
        parser.advance(&mut buffer, seq);
        assert_eq!(buffer.cursor.style, CursorStyle::SteadyUnderline);

        // CSI 6 SP q -> SteadyBar
        let seq = b"\x1b[6 q";
        parser.advance(&mut buffer, seq);
        assert_eq!(buffer.cursor.style, CursorStyle::SteadyBar);

        // CSI 0 SP q -> BlinkingBlock (0 maps to 1)
        let seq = b"\x1b[0 q";
        parser.advance(&mut buffer, seq);
        assert_eq!(buffer.cursor.style, CursorStyle::BlinkingBlock);

        // CSI 5 SP q -> BlinkingBar
        let seq = b"\x1b[5 q";
        parser.advance(&mut buffer, seq);
        assert_eq!(buffer.cursor.style, CursorStyle::BlinkingBar);

        // Multiple params: CSI 2;5 SP q -> should use last param (5: BlinkingBar)
        let seq = b"\x1b[2;5 q";
        parser.advance(&mut buffer, seq);
        assert_eq!(buffer.cursor.style, CursorStyle::BlinkingBar);

        // Invalid param: CSI 9 SP q -> should default to BlinkingBlock
        let seq = b"\x1b[9 q";
        parser.advance(&mut buffer, seq);
        assert_eq!(buffer.cursor.style, CursorStyle::BlinkingBlock);
    }

    #[test]
    fn test_grapheme_cluster_handling() {
        let mut buffer = TerminalBufferEntity::new(80, 24);
        let mut parser = Parser::new();

        // 1. 結合文字 (a + COMBINING RING ABOVE = å)
        parser.advance(&mut buffer, "a\u{030A}".as_bytes());
        buffer.flush_pending_cluster();
        assert_eq!(buffer.cursor.x, 1);
        let cell = &buffer.lines[0][0];
        assert_eq!(cell.text, "a\u{030A}");
        assert!(!cell.is_wide_continuation);

        // 2. Emoji ZWJ Sequence (Family)
        buffer.cursor.x = 0;
        let family_emoji = "👨‍👩‍👧‍👦";
        parser.advance(&mut buffer, family_emoji.as_bytes());
        buffer.flush_pending_cluster();
        // 物理カラム幅は 2 に制限されるべき
        assert_eq!(buffer.cursor.x, 2);
        assert_eq!(buffer.lines[0][0].text, family_emoji);
        assert!(buffer.lines[0][1].is_wide_continuation);

        // 3. 国旗 (Regional Indicator)
        buffer.cursor.x = 0;
        let flag_emoji = "🇯🇵";
        parser.advance(&mut buffer, flag_emoji.as_bytes());
        buffer.flush_pending_cluster();
        assert_eq!(buffer.cursor.x, 2);
        assert_eq!(buffer.lines[0][0].text, flag_emoji);

        // 4. クラスター単位のバックスペース
        parser.advance(&mut buffer, b"\x08");
        assert_eq!(buffer.cursor.x, 0);
    }

    #[test]
    fn test_wide_char_boundary_protection_issue_104() {
        let mut buffer = TerminalBufferEntity::new(10, 5);
        let mut parser = Parser::new();
        parser.advance(&mut buffer, "日本語".as_bytes());
        buffer.flush_pending_cluster();

        buffer.cursor.x = 1;
        parser.advance(&mut buffer, b"A");
        buffer.flush_pending_cluster();
        assert_eq!(buffer.lines[0][0].text, " ");
        assert_eq!(buffer.lines[0][1].text, "A");

        buffer.cursor.x = 2;
        parser.advance(&mut buffer, b"B");
        buffer.flush_pending_cluster();
        assert_eq!(buffer.lines[0][2].text, "B");
        assert_eq!(buffer.lines[0][3].text, " ");
    }

    #[test]
    fn test_delete_character_alignment_issue() {
        let mut buffer = TerminalBufferEntity::new(10, 5);
        let mut parser = Parser::new();
        parser.advance(&mut buffer, "日本".as_bytes());
        buffer.flush_pending_cluster();

        buffer.cursor.x = 1;
        parser.advance(&mut buffer, b"\x1b[1P");
        assert_eq!(buffer.lines[0][0].text, " ");
    }
}
