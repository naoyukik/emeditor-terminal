use super::terminal_types_entity::Cell;
use std::collections::VecDeque;

pub struct TerminalScrollbackEntity {
    history: VecDeque<Vec<Cell>>,
    viewport_offset: usize,
    scrollback_limit: usize,
}

impl TerminalScrollbackEntity {
    pub fn new(limit: usize) -> Self {
        Self {
            history: VecDeque::new(),
            viewport_offset: 0,
            scrollback_limit: limit,
        }
    }

    pub fn history(&self) -> &VecDeque<Vec<Cell>> {
        &self.history
    }
    pub fn viewport_offset(&self) -> usize {
        self.viewport_offset
    }

    pub fn push(&mut self, line: Vec<Cell>) {
        if self.scrollback_limit == 0 {
            return;
        }
        if self.history.len() >= self.scrollback_limit {
            self.history.pop_front();
        }
        self.history.push_back(line);
        if self.viewport_offset > 0 {
            self.viewport_offset = (self.viewport_offset + 1).min(self.history.len());
        }
    }

    pub fn scroll_to(&mut self, offset: usize) {
        self.viewport_offset = offset.min(self.history.len());
    }

    pub fn scroll_lines(&mut self, delta: isize) {
        let new_offset = (self.viewport_offset as isize + delta)
            .max(0)
            .min(self.history.len() as isize);
        self.viewport_offset = new_offset as usize;
    }

    pub fn reset_viewport(&mut self) {
        self.viewport_offset = 0;
    }
}
