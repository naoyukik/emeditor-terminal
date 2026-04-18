use std::collections::VecDeque;
use unicode_width::UnicodeWidthStr;
use super::terminal_types_entity::{Cell, TerminalAttribute, TerminalColor};

pub struct TerminalGridEntity {
    lines: VecDeque<Vec<Cell>>,
    width: usize,
    height: usize,
}

impl TerminalGridEntity {
    pub fn new(width: usize, height: usize) -> Self {
        let mut lines = VecDeque::with_capacity(height);
        for _ in 0..height {
            lines.push_back(vec![Cell::default(); width]);
        }
        Self { lines, width, height }
    }

    pub fn lines(&self) -> &VecDeque<Vec<Cell>> { &self.lines }
    pub fn lines_mut(&mut self) -> &mut VecDeque<Vec<Cell>> { &mut self.lines }

    pub fn ensure_safe_boundary(&mut self, y: usize, x: usize, bg_color: &TerminalColor) {
        if y >= self.lines.len() || x >= self.width { return; }
        let empty = Cell { text: " ".to_string(), attribute: TerminalAttribute { fg: TerminalColor::Default, bg: bg_color.clone(), ..TerminalAttribute::default() }, is_wide_continuation: false };
        if self.lines[y][x].is_wide_continuation {
            if x > 0 { self.lines[y][x - 1] = empty.clone(); }
            self.lines[y][x] = empty.clone();
        } else if self.lines[y][x].text.width() > 1 {
            if x + 1 < self.width { self.lines[y][x + 1] = empty.clone(); }
            self.lines[y][x] = empty.clone();
        }
    }

    pub fn put_cell(&mut self, x: usize, y: usize, cell: Cell, display_width: usize, bg_color: &TerminalColor) {
        if y >= self.lines.len() || x >= self.width { return; }
        self.ensure_safe_boundary(y, x, bg_color);
        if display_width == 2 && x + 1 < self.width { self.ensure_safe_boundary(y, x + 1, bg_color); }
        if let Some(line) = self.lines.get_mut(y) {
            line[x] = cell.clone();
            if display_width == 2 && x + 1 < line.len() {
                line[x + 1] = Cell { text: " ".to_string(), attribute: cell.attribute, is_wide_continuation: true };
            }
        }
    }

    pub fn insert_lines(&mut self, row: usize, n: usize, bottom: usize, empty_line: Vec<Cell>) {
        for _ in 0..n {
            if bottom < self.lines.len() { self.lines.remove(bottom); }
            if row < self.lines.len() { self.lines.insert(row, empty_line.clone()); }
        }
    }

    pub fn delete_lines(&mut self, row: usize, n: usize, bottom: usize, empty_line: Vec<Cell>) {
        for _ in 0..n {
            if row < self.lines.len() { self.lines.remove(row); }
            if bottom < self.lines.len() { self.lines.insert(bottom, empty_line.clone()); }
        }
    }

    pub fn fill_line(&mut self, y: usize, start_x: usize, end_x: usize, cell: Cell) {
        if let Some(line) = self.lines.get_mut(y) {
            let end = end_x.min(line.len());
            for c in line.iter_mut().take(end).skip(start_x) { *c = cell.clone(); }
        }
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        for line in &mut self.lines { line.resize(new_width, Cell::default()); }
        if new_height > self.height {
            for _ in 0..(new_height - self.height) { self.lines.push_back(vec![Cell::default(); new_width]); }
        } else { self.lines.truncate(new_height); }
        self.width = new_width;
        self.height = new_height;
    }
}
