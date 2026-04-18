use crate::domain::model::terminal_buffer_entity::TerminalBufferEntity;
use super::terminal_protocol_handler::TerminalProtocolHandler;
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
        // Protocol Handler を介してバッファを操作する
        let mut handler = TerminalProtocolHandler::new(buffer);
        // vte 0.15 の advance は &[u8] を受け取るため、一文字ずつスライスとして渡す
        for byte in bytes {
            self.parser.advance(&mut handler, std::slice::from_ref(byte));
        }
    }
}
