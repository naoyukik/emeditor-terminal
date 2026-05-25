use super::terminal_scrollback_entity::TerminalScrollbackEntity;
use super::terminal_types_entity::Cell;
use std::collections::VecDeque;

pub struct TerminalHistoryViewEntity;

#[cfg(test)]
fn cell_with_text(text: &str) -> Cell {
    Cell {
        text: text.to_string(),
        ..Cell::default()
    }
}

impl TerminalHistoryViewEntity {
    pub fn resolve_visual_row<'a>(
        visual_row: usize,
        height: usize,
        grid_lines: &'a VecDeque<Vec<Cell>>,
        scrollback: &'a TerminalScrollbackEntity,
    ) -> Option<&'a Vec<Cell>> {
        let dist =
            (height.saturating_sub(1).saturating_sub(visual_row)) + scrollback.viewport_offset();
        if dist < grid_lines.len() {
            grid_lines.get(grid_lines.len().saturating_sub(1).saturating_sub(dist))
        } else {
            scrollback.history().get(
                scrollback
                    .history()
                    .len()
                    .saturating_sub(1)
                    .saturating_sub(dist.saturating_sub(grid_lines.len())),
            )
        }
    }

    pub fn history_len(scrollback: &TerminalScrollbackEntity) -> usize {
        scrollback.history().len()
    }

    pub fn viewport_offset(scrollback: &TerminalScrollbackEntity) -> usize {
        scrollback.viewport_offset()
    }

    pub fn scroll_to(scrollback: &mut TerminalScrollbackEntity, offset: usize) {
        scrollback.scroll_to(offset);
    }

    pub fn scroll_lines(scrollback: &mut TerminalScrollbackEntity, delta: isize) {
        scrollback.scroll_lines(delta);
    }

    pub fn reset_viewport(scrollback: &mut TerminalScrollbackEntity) {
        scrollback.reset_viewport();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_visual_row_from_screen_and_history() {
        let mut grid_lines = VecDeque::new();
        grid_lines.push_back(vec![cell_with_text("S0")]);
        grid_lines.push_back(vec![cell_with_text("S1")]);
        grid_lines.push_back(vec![cell_with_text("S2")]);

        let mut scrollback = TerminalScrollbackEntity::new(10);
        scrollback.push(vec![cell_with_text("H0")]);
        scrollback.push(vec![cell_with_text("H1")]);

        // viewport=0: visual_row 2 is newest screen line
        let bottom =
            TerminalHistoryViewEntity::resolve_visual_row(2, 3, &grid_lines, &scrollback).unwrap();
        assert_eq!(bottom[0].text, "S2");

        // viewport=2: visual_row 0 points to older history side
        TerminalHistoryViewEntity::scroll_to(&mut scrollback, 2);
        let from_history =
            TerminalHistoryViewEntity::resolve_visual_row(0, 3, &grid_lines, &scrollback).unwrap();
        assert_eq!(from_history[0].text, "H0");
    }

    #[test]
    fn scroll_operations_are_delegated() {
        let mut scrollback = TerminalScrollbackEntity::new(10);
        scrollback.push(vec![cell_with_text("H0")]);
        scrollback.push(vec![cell_with_text("H1")]);
        scrollback.push(vec![cell_with_text("H2")]);

        assert_eq!(TerminalHistoryViewEntity::history_len(&scrollback), 3);
        assert_eq!(TerminalHistoryViewEntity::viewport_offset(&scrollback), 0);

        TerminalHistoryViewEntity::scroll_to(&mut scrollback, 2);
        assert_eq!(TerminalHistoryViewEntity::viewport_offset(&scrollback), 2);

        TerminalHistoryViewEntity::scroll_lines(&mut scrollback, -1);
        assert_eq!(TerminalHistoryViewEntity::viewport_offset(&scrollback), 1);

        TerminalHistoryViewEntity::reset_viewport(&mut scrollback);
        assert_eq!(TerminalHistoryViewEntity::viewport_offset(&scrollback), 0);
    }
}
