use crate::domain::terminal::TerminalBuffer;
use crate::domain::parser::AnsiParser;
use crate::infra::conpty::ConPTY;

pub struct TerminalService {
    pub buffer: TerminalBuffer,
    pub parser: AnsiParser,
    pub conpty: Option<ConPTY>,
}

impl TerminalService {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            buffer: TerminalBuffer::new(cols, rows),
            parser: AnsiParser::new(),
            conpty: None,
        }
    }

    pub fn process_output(&mut self, output: &str) {
        self.parser.parse(output, &mut self.buffer);
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        if let Some(conpty) = &self.conpty {
            let _ = conpty.resize(cols as i16, rows as i16);
        }
        self.buffer.resize(cols, rows);
    }
}
