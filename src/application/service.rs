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

    /// ビューポートを指定したオフセットにスクロールする
    pub fn scroll_to(&mut self, offset: usize) {
        self.buffer.scroll_to(offset);
    }

    /// ビューポートを相対的にスクロールする
    pub fn scroll_lines(&mut self, delta: isize) {
        self.buffer.scroll_lines(delta);
    }

    /// ビューポートを最新状態（最下部）にリセットする
    pub fn reset_viewport(&mut self) {
        self.buffer.reset_viewport();
    }

    /// ヒストリーの現在の行数を取得する
    pub fn get_history_count(&self) -> usize {
        self.buffer.history.len()
    }

    /// 現在のビューポートのオフセットを取得する
    pub fn get_viewport_offset(&self) -> usize {
        self.buffer.viewport_offset
    }
}
