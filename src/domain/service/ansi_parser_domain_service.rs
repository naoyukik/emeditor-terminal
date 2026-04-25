use super::terminal_protocol_handler::TerminalProtocolHandler;
use crate::domain::model::terminal_buffer_entity::TerminalBufferEntity;
use vte::Parser;

/// ANSI エスケープシーケンスのパースを担うドメインサービス
pub(crate) struct AnsiParserDomainService {
    parser: Parser,
}

impl AnsiParserDomainService {
    pub(crate) fn new() -> Self {
        Self {
            parser: Parser::new(),
        }
    }

    pub(crate) fn parse(&mut self, bytes: &[u8], buffer: &mut TerminalBufferEntity) {
        {
            // Protocol Handler を介してバッファを操作する
            let mut handler = TerminalProtocolHandler::new(buffer);
            // vte 0.15 の advance は &[u8] を受け取るため、入力全体をまとめて渡す
            self.parser.advance(&mut handler, bytes);
        }
        // 各データ受信パケットの処理後に強制的にフラッシュを行い、表示遅延を解消する
        buffer.flush_pending_cluster();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::terminal_buffer_entity::{Cell, TerminalBufferEntity, TerminalColor};

    fn line_to_string(line: &[Cell]) -> String {
        line.iter()
            .filter(|c| !c.is_wide_continuation)
            .map(|cell| cell.text.as_str())
            .collect()
    }

    #[test]
    fn test_parser_basic() {
        let mut buffer = TerminalBufferEntity::new(80, 25);
        let mut parser = AnsiParserDomainService::new();
        parser.parse(b"Hello", &mut buffer);
        let first_line = line_to_string(buffer.get_line_at_visual_row(0).unwrap());
        assert!(first_line.starts_with("Hello"));
    }

    #[test]
    fn test_utf8_fragmentation() {
        let mut buffer = TerminalBufferEntity::new(10, 5);
        let mut parser = AnsiParserDomainService::new();
        // "あ" (E3 81 82)
        parser.parse(&[0xE3, 0x81], &mut buffer);
        assert_eq!(buffer.get_cursor_pos().0, 0);
        parser.parse(&[0x82], &mut buffer);
        assert_eq!(buffer.get_cursor_pos().0, 2);
        assert_eq!(buffer.get_line_at_visual_row(0).unwrap()[0].text, "あ");
    }

    #[test]
    fn test_sgr_colors() {
        let mut buffer = TerminalBufferEntity::new(80, 25);
        let mut parser = AnsiParserDomainService::new();
        parser.parse(b"\x1b[31mRED\x1b[0m", &mut buffer);
        let line = buffer.get_line_at_visual_row(0).unwrap();
        assert_eq!(line[0].text, "R");
        assert_eq!(line[0].attribute.fg, TerminalColor::Ansi(1));
        assert_eq!(line[3].text, " ");
    }

    #[test]
    fn test_cursor_visibility() {
        let mut buffer = TerminalBufferEntity::new(80, 25);
        let mut parser = AnsiParserDomainService::new();

        // Cursor invisible
        parser.parse(b"\x1b[?25l", &mut buffer);
        assert!(!buffer.is_cursor_visible());

        // Cursor visible
        parser.parse(b"\x1b[?25h", &mut buffer);
        assert!(buffer.is_cursor_visible());
    }

    #[test]
    fn test_scrollback_history() {
        let mut buffer = TerminalBufferEntity::new(10, 3);
        let mut parser = AnsiParserDomainService::new();
        parser.parse(b"1\n2\n3\n4\n5", &mut buffer);

        // 3 lines height, 5 lines input_value -> 2 lines in history
        assert_eq!(buffer.get_history_len(), 2);
        assert_eq!(buffer.get_line_at_visual_row(0).unwrap()[0].text, "3");
        assert_eq!(buffer.get_line_at_visual_row(2).unwrap()[0].text, "5");
    }

    #[test]
    fn test_mouse_mode_parsing() {
        use crate::domain::model::terminal_types_entity::MouseTrackingMode;
        let mut buffer = TerminalBufferEntity::new(80, 25);
        let mut parser = AnsiParserDomainService::new();

        // Default Tracking (1000)
        parser.parse(b"\x1b[?1000h", &mut buffer);
        assert_eq!(buffer.get_mouse_tracking_mode(), MouseTrackingMode::Default);

        // Button Event Tracking (1002)
        parser.parse(b"\x1b[?1002h", &mut buffer);
        assert_eq!(
            buffer.get_mouse_tracking_mode(),
            MouseTrackingMode::ButtonEvent
        );

        // Any Event Tracking (1003)
        parser.parse(b"\x1b[?1003h", &mut buffer);
        assert_eq!(
            buffer.get_mouse_tracking_mode(),
            MouseTrackingMode::AnyEvent
        );

        // Disable Tracking
        parser.parse(b"\x1b[?1000l", &mut buffer);
        assert_eq!(buffer.get_mouse_tracking_mode(), MouseTrackingMode::None);

        // SGR Encoding (1006)
        assert!(!buffer.is_sgr_mouse_encoding_enabled());
        parser.parse(b"\x1b[?1006h", &mut buffer);
        assert!(buffer.is_sgr_mouse_encoding_enabled());

        parser.parse(b"\x1b[?1006l", &mut buffer);
        assert!(!buffer.is_sgr_mouse_encoding_enabled());
    }
}
