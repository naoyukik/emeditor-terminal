use crate::domain::model::terminal_buffer_entity::TerminalBufferEntity;
use vte::Parser;

pub(crate) struct AnsiParserDomainService {
    parser: Parser,
}

impl AnsiParserDomainService {
    pub(crate) fn new() -> Self {
        Self {
            parser: Parser::new(),
        }
    }
}

impl Default for AnsiParserDomainService {
    fn default() -> Self {
        Self::new()
    }
}

impl AnsiParserDomainService {
    pub(crate) fn parse(&mut self, bytes: &[u8], buffer: &mut TerminalBufferEntity) {
        self.parser.advance(buffer, bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::terminal_buffer_entity::{Cell, TerminalBufferEntity, TerminalColor};

    fn line_to_string(line: &[Cell]) -> String {
        line.iter()
            .filter(|c| !c.is_wide_continuation)
            .map(|cell| cell.c)
            .collect()
    }

    #[test]
    fn test_parser_basic() {
        let mut buffer = TerminalBufferEntity::new(80, 25);
        let mut parser = AnsiParserDomainService::new();
        parser.parse(b"Hello", &mut buffer);
        let first_line = line_to_string(&buffer.get_lines()[0]);
        assert!(first_line.starts_with("Hello"));
    }

    #[test]
    fn test_utf8_fragmentation() {
        let mut buffer = TerminalBufferEntity::new(10, 5);
        let mut parser = AnsiParserDomainService::new();
        parser.parse(&[0xE3, 0x81], &mut buffer);
        assert_eq!(buffer.get_cursor_pos().0, 0);
        parser.parse(&[0x82], &mut buffer);
        assert_eq!(buffer.get_cursor_pos().0, 2);
        assert_eq!(buffer.get_lines()[0][0].c, 'あ');
    }

    #[test]
    fn test_cursor_initialization() {
        let buffer = TerminalBufferEntity::new(80, 25);
        assert_eq!(buffer.get_cursor_pos().0, 0);
        assert_eq!(buffer.get_cursor_pos().1, 0);
    }

    #[test]
    fn test_sgr_colors() {
        let mut buffer = TerminalBufferEntity::new(80, 25);
        let mut parser = AnsiParserDomainService::new();
        parser.parse(b"\x1b[31mRED\x1b[0m", &mut buffer);
        let line = &buffer.get_lines()[0];
        assert_eq!(line[0].c, 'R');
        assert_eq!(line[0].attribute.fg, TerminalColor::Ansi(1));
        assert_eq!(line[3].c, ' ');
    }

    #[test]
    fn test_terminal_resize() {
        let mut buffer = TerminalBufferEntity::new(5, 2);
        let mut parser = AnsiParserDomainService::new();
        parser.parse(b"1234567", &mut buffer);
        assert_eq!(buffer.get_cursor_pos().1, 1);
        assert_eq!(buffer.get_cursor_pos().0, 2);

        buffer.resize(10, 5);
        assert_eq!(buffer.get_width(), 10);
        assert_eq!(buffer.get_height(), 5);
        assert_eq!(buffer.get_lines()[0][0].c, '1');
    }

    #[test]
    fn test_scrollback_history() {
        let mut buffer = TerminalBufferEntity::new(10, 3);
        let mut parser = AnsiParserDomainService::new();
        parser.parse(b"1\n2\n3\n4\n5", &mut buffer);

        assert_eq!(buffer.get_history_len(), 2);
        // lines and history are now private, use get_line_at_visual_row or similar
        assert_eq!(buffer.get_line_at_visual_row(0).unwrap()[0].c, '3');
        assert_eq!(buffer.get_line_at_visual_row(2).unwrap()[0].c, '5');
    }

    #[test]
    fn test_viewport_logic() {
        let mut buffer = TerminalBufferEntity::new(10, 3);
        let mut parser = AnsiParserDomainService::new();
        parser.parse(b"1\n2\n3\n4\n5", &mut buffer);

        let line = buffer.get_line_at_visual_row(0).unwrap();
        assert_eq!(line[0].c, '3');

        buffer.scroll_lines(1);
        let line = buffer.get_line_at_visual_row(0).unwrap();
        assert_eq!(line[0].c, '2');
    }

    #[test]
    fn test_decset_decrst() {
        let mut buffer = TerminalBufferEntity::new(80, 25);
        let mut parser = AnsiParserDomainService::new();

        // Cursor invisible
        parser.parse(b"\x1b[?25l", &mut buffer);
        assert!(!buffer.is_cursor_visible());

        // Cursor visible
        parser.parse(b"\x1b[?25h", &mut buffer);
        assert!(buffer.is_cursor_visible());

        // Origin mode and cursor reset
        parser.parse(b"\x1b[2;10r", &mut buffer); // Scroll region 2-10
        parser.parse(b"\x1b[?6h", &mut buffer);
        assert_eq!(buffer.get_cursor_pos().1, 1); // Row 2 (0-indexed)
        assert_eq!(buffer.get_cursor_pos().0, 0);

        parser.parse(b"\x1b[?6l", &mut buffer);
        assert_eq!(buffer.get_cursor_pos().1, 0);
        assert_eq!(buffer.get_cursor_pos().0, 0);
    }

    #[test]
    fn test_control_characters() {
        let mut buffer = TerminalBufferEntity::new(80, 25);
        let mut parser = AnsiParserDomainService::new();

        // BS
        parser.parse(b"AB\x08", &mut buffer);
        assert_eq!(buffer.get_cursor_pos().0, 1);

        // TAB
        parser.parse(b"\r\t", &mut buffer);
        assert_eq!(buffer.get_cursor_pos().0, 8);

        // CR
        parser.parse(b"XY\r", &mut buffer);
        assert_eq!(buffer.get_cursor_pos().0, 0);
    }
}
