use crate::domain::parser::AnsiParser;
use crate::domain::repository::configuration_repository::ConfigurationRepository;
use crate::domain::repository::terminal_output_repository::TerminalOutputRepository;
use crate::domain::terminal::TerminalBuffer;

pub struct TerminalService {
    pub(crate) buffer: TerminalBuffer,
    parser: AnsiParser,
    output_repo: Box<dyn TerminalOutputRepository>,
    config_repo: Box<dyn ConfigurationRepository>,
    // キャッシュされた設定情報
    font_face: String,
    font_size: i32,
}

impl TerminalService {
    pub fn new(
        cols: usize,
        rows: usize,
        output_repo: Box<dyn TerminalOutputRepository>,
        config_repo: Box<dyn ConfigurationRepository>,
    ) -> Self {
        let font_face = config_repo.get_font_face();
        let font_size = config_repo.get_font_size();

        Self {
            buffer: TerminalBuffer::new(cols, rows),
            parser: AnsiParser::new(),
            output_repo,
            config_repo,
            font_face,
            font_size,
        }
    }

    pub fn process_output(&mut self, output: &str) {
        self.parser.parse(output, &mut self.buffer);
    }

    pub fn send_input(&self, data: &[u8]) -> std::io::Result<()> {
        self.output_repo.send_input(data)
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        if let Err(err) = self.output_repo.resize(cols as u16, rows as u16) {
            log::error!(
                "Failed to resize terminal output (requested: {}x{}): {:?}",
                cols,
                rows,
                err
            );
        }
        self.buffer.resize(cols, rows);
    }

    /// 設定を最新状態に更新する
    pub fn refresh_config(&mut self) {
        self.font_face = self.config_repo.get_font_face();
        self.font_size = self.config_repo.get_font_size();
    }

    pub fn get_font_face(&self) -> &str {
        &self.font_face
    }

    pub fn get_font_size(&self) -> i32 {
        self.font_size
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