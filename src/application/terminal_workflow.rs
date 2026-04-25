use crate::domain::model::color_theme_value::ColorTheme;
use crate::domain::model::input_value::MouseEvent;
use crate::domain::model::terminal_buffer_entity::TerminalBufferEntity;
use crate::domain::model::terminal_config_value::TerminalConfig;
use crate::domain::repository::configuration_repository::{ConfigError, ConfigurationRepository};
use crate::domain::repository::key_translator_repository::KeyTranslatorRepository;
use crate::domain::repository::terminal_output_repository::TerminalOutputRepository;
use crate::domain::service::ansi_parser_domain_service::AnsiParserDomainService;

pub struct TerminalWorkflow {
    buffer: TerminalBufferEntity,
    parser: AnsiParserDomainService,
    output_repo: Box<dyn TerminalOutputRepository>,
    config_repo: Box<dyn ConfigurationRepository>,
    translator: Box<dyn KeyTranslatorRepository>,
    // キャッシュされた設定情報
    font_face: String,
    font_size: i32,
    #[allow(dead_code)]
    font_weight: i32,
    #[allow(dead_code)]
    font_italic: bool,
    pub(crate) config: TerminalConfig,
    pub(crate) color_theme: ColorTheme,
    is_dark: bool,
}

impl TerminalWorkflow {
    pub fn new(
        cols: usize,
        rows: usize,
        output_repo: Box<dyn TerminalOutputRepository>,
        config_repo: Box<dyn ConfigurationRepository>,
        translator: Box<dyn KeyTranslatorRepository>,
        is_dark: bool,
    ) -> Self {
        let config = config_repo.load();
        log::info!(
            "Loaded terminal config: theme={:?}, font_face='{}', font_size={}, weight={}, italic={}",
            config.theme_type,
            config.font_face,
            config.font_size,
            config.font_weight,
            config.font_italic
        );
        log::debug!("Loaded terminal shell_path='{}'", config.shell_path);
        let font_face = config.font_face.clone();
        let font_size = config.font_size;
        let font_weight = config.font_weight;
        let font_italic = config.font_italic;
        let color_theme = config.get_color_theme(is_dark);

        Self {
            buffer: TerminalBufferEntity::new(cols, rows),
            parser: AnsiParserDomainService::new(),
            output_repo,
            config_repo,
            translator,
            font_face,
            font_size,
            font_weight,
            font_italic,
            config,
            color_theme,
            is_dark,
        }
    }

    pub fn process_output(&mut self, output_bytes: &[u8]) {
        self.parser.parse(output_bytes, &mut self.buffer);
    }

    pub fn send_input(&self, input_bytes: &[u8]) -> std::io::Result<()> {
        self.output_repo.send_input(input_bytes)
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

    /// 現在の設定を永続化領域に保存する
    #[allow(dead_code)]
    pub fn persist_config(&self) -> Result<(), ConfigError> {
        self.config_repo.save(&self.config)
    }

    /// 設定を最新状態に更新する
    #[allow(dead_code)] // TODO: フォント設定UI実装時に使用予定
    pub fn refresh_config(&mut self) {
        let config = self.config_repo.load();
        self.font_face = config.font_face.clone();
        self.font_size = config.font_size;
        self.color_theme = config.get_color_theme(self.is_dark);
        self.config = config;
    }

    #[allow(dead_code)] // TODO: フォント設定UI実装時に使用予定
    pub fn get_font_face(&self) -> &str {
        &self.font_face
    }

    #[allow(dead_code)] // TODO: フォント設定UI実装時に使用予定
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

    pub fn get_buffer(&self) -> &TerminalBufferEntity {
        &self.buffer
    }

    /// ヒストリーの現在の行数を取得する
    pub fn get_history_count(&self) -> usize {
        self.buffer.get_history_len()
    }

    /// 現在のビューポートのオフセットを取得する
    pub fn get_viewport_offset(&self) -> usize {
        self.buffer.get_viewport_offset()
    }

    /// マウスイベントを処理する
    pub fn handle_mouse_event(&mut self, event: MouseEvent) -> std::io::Result<()> {
        use crate::domain::model::input_value::MouseButton;
        use crate::domain::model::terminal_types_entity::MouseTrackingMode;

        let mode = self.buffer.get_mouse_tracking_mode();

        if mode == MouseTrackingMode::None {
            return Ok(());
        }

        // モードに応じたフィルタリング
        let should_send = match mode {
            MouseTrackingMode::Default => {
                // 1000: ボタンプレス/リリースのみ（ドラッグ/ホバー除外）
                !event.is_drag && event.button != MouseButton::None
            }
            MouseTrackingMode::ButtonEvent => {
                // 1002: ボタンプレス/リリース + ドラッグ（ホバー除外）
                event.button != MouseButton::None
            }
            MouseTrackingMode::AnyEvent => {
                // 1003: 全て送信
                true
            }
            _ => false,
        };

        if should_send
            && let Some(seq) = self.translator.translate_mouse(event)
        {
            log::debug!(
                "Sending mouse VT sequence: {:?}",
                String::from_utf8_lossy(&seq)
            );
            self.reset_viewport();
            return self.send_input(&seq);
        }

        Ok(())
    }
}
