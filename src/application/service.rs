use crate::domain::parser::AnsiParser;
use crate::domain::terminal::TerminalBuffer;
use crate::infra::conpty::ConPTY;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::WriteFile;

pub struct TerminalService {
    pub(crate) buffer: TerminalBuffer,
    parser: AnsiParser,
    conpty: Option<ConPTY>,
}

impl TerminalService {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            buffer: TerminalBuffer::new(cols, rows),
            parser: AnsiParser::new(),
            conpty: None,
        }
    }

    pub fn set_conpty(&mut self, conpty: ConPTY) {
        self.conpty = Some(conpty);
    }

    #[allow(dead_code)]
    pub fn get_conpty_output_handle(&self) -> Option<HANDLE> {
        self.conpty.as_ref().map(|c| c.get_output_handle().0)
    }

    pub fn take_conpty(&mut self) -> Option<ConPTY> {
        self.conpty.take()
    }

    pub fn process_output(&mut self, output: &str) {
        self.parser.parse(output, &mut self.buffer);
    }

    pub fn send_input(&self, data: &[u8]) -> Result<(), windows::core::Error> {
        if let Some(conpty) = &self.conpty {
            let handle = conpty.get_input_handle();
            let mut bytes_written = 0;
            unsafe {
                WriteFile(handle.0, Some(data), Some(&mut bytes_written), None)?;
            }
        }
        Ok(())
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
