use crate::domain::model::color_theme_value::ColorTheme;
use crate::domain::model::input_value::{InputKey, MouseEvent};
use crate::domain::model::terminal_buffer_entity::TerminalBufferEntity;
use crate::domain::model::terminal_buffer_view_entity::{SelectionPoint, TerminalBufferViewEntity};
use crate::domain::model::terminal_config_value::TerminalConfig;
use crate::domain::repository::clipboard_repository::ClipboardRepository;
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
    clipboard_repo: Box<dyn ClipboardRepository>,
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
        clipboard_repo: Box<dyn ClipboardRepository>,
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
            clipboard_repo,
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

    #[allow(dead_code)]
    pub fn get_buffer(&self) -> &TerminalBufferEntity {
        &self.buffer
    }

    pub fn buffer_view(&self) -> &dyn TerminalBufferViewEntity {
        &self.buffer
    }

    pub fn get_buffer_width(&self) -> usize {
        self.buffer.get_width()
    }

    pub fn get_buffer_height(&self) -> usize {
        self.buffer.get_height()
    }

    pub fn get_ime_anchor_pos(&self) -> (usize, usize) {
        self.buffer.get_ime_anchor_pos()
    }

    /// ヒストリーの現在の行数を取得する
    pub fn get_history_count(&self) -> usize {
        self.buffer.get_history_len()
    }

    /// 現在のビューポートのオフセットを取得する
    pub fn get_viewport_offset(&self) -> usize {
        self.buffer.get_viewport_offset()
    }

    /// キー入力イベントを処理する
    pub fn handle_key_event(&mut self, key: InputKey) -> std::io::Result<Option<Vec<u8>>> {
        // Ctrl+C (VK_C = 0x43)
        if key.vk_code == 0x43
            && key.modifiers.is_ctrl_pressed
            && !key.modifiers.is_shift_pressed
            && !key.modifiers.is_alt_pressed
            && self.buffer.get_selection_range().is_some()
        {
            let text = self.buffer.get_selected_text();
            if !text.is_empty() {
                let _ = self.clipboard_repo.set_text(&text);
            }
            self.buffer.set_selection_range(None);
            return Ok(Some(Vec::new())); // コピーしたので、ターミナルには何も送らない
        }

        // Ctrl+V (VK_V = 0x56)
        if key.vk_code == 0x56
            && key.modifiers.is_ctrl_pressed
            && !key.modifiers.is_shift_pressed
            && !key.modifiers.is_alt_pressed
            && let Ok(text) = self.clipboard_repo.get_text()
            && !text.is_empty()
        {
            self.reset_viewport();
            self.send_input(text.as_bytes())?;
            return Ok(Some(Vec::new())); // 貼り付けたので、キー自体は送らない
        }

        // 通常の翻訳
        Ok(self.translator.translate(key))
    }

    /// マウスイベントを処理する
    pub fn handle_mouse_event(&mut self, event: MouseEvent) -> std::io::Result<bool> {
        use crate::domain::model::input_value::MouseButton;
        use crate::domain::model::terminal_types_entity::MouseTrackingMode;

        let mode = self.buffer.get_mouse_tracking_mode();

        if mode == MouseTrackingMode::None {
            // 右クリックでコピーまたは貼り付け (Down時に実行)
            if event.button == MouseButton::Right && !event.is_release && !event.is_drag {
                // 選択範囲があればコピー
                if self.buffer.get_selection_range().is_some() {
                    let text = self.buffer.get_selected_text();
                    if !text.is_empty() {
                        let _ = self.clipboard_repo.set_text(&text);
                    }
                    self.buffer.set_selection_range(None);
                    return Ok(true);
                }

                // 選択範囲がなければ貼り付け
                if let Ok(text) = self.clipboard_repo.get_text()
                    && !text.is_empty()
                {
                    self.reset_viewport();
                    self.send_input(text.as_bytes())?;
                    return Ok(true);
                }
            }

            // 左クリックでのテキスト選択またはカーソル移動
            if event.button == MouseButton::Left {
                if !event.is_release {
                    if !event.is_drag {
                        // ダウン時：選択範囲を開始座標で初期化
                        let logical_row = self.buffer.visual_row_to_logical_row(event.y);
                        let point = SelectionPoint {
                            x: event.x,
                            logical_row,
                        };
                        self.buffer.set_selection_range(Some((point, point)));
                    } else {
                        // ドラッグ中：選択範囲の更新
                        let current_range = self.buffer.get_selection_range();
                        match current_range {
                            Some((start, _)) => {
                                let end = SelectionPoint {
                                    x: event.x,
                                    logical_row: self.buffer.visual_row_to_logical_row(event.y),
                                };
                                self.buffer.set_selection_range(Some((start, end)));
                            }
                            None => {
                                // セーフティ：万が一Downを逃していた場合
                                let logical_row = self.buffer.visual_row_to_logical_row(event.y);
                                let point = SelectionPoint {
                                    x: event.x,
                                    logical_row,
                                };
                                self.buffer.set_selection_range(Some((point, point)));
                            }
                        }
                        return Ok(true); // 再描画を促す
                    }
                } else {
                    // アップ時
                    if !event.is_drag {
                        // ドラッグなしの場合：
                        let current_range = self.buffer.get_selection_range();
                        let is_click = match current_range {
                            Some((start, end)) => start == end,
                            None => true,
                        };

                        if is_click {
                            // 同一座標でのクリックなら、カーソル移動を試行し、選択を解除
                            self.buffer.set_selection_range(None);
                            let (cur_x, cur_y) = self.buffer.get_cursor_pos();
                            let viewport_offset = self.buffer.get_viewport_offset();

                            if viewport_offset == 0 && event.y == cur_y {
                                let mut seq = Vec::new();
                                if event.x > cur_x {
                                    let diff = event.x - cur_x;
                                    for _ in 0..diff {
                                        seq.extend_from_slice(b"\x1b[C");
                                    }
                                } else if event.x < cur_x {
                                    let diff = cur_x - event.x;
                                    for _ in 0..diff {
                                        seq.extend_from_slice(b"\x1b[D");
                                    }
                                }

                                if !seq.is_empty() {
                                    self.send_input(&seq)?;
                                    return Ok(true);
                                }
                            }
                        }
                    }
                    // ドラッグ終了（または移動後のアップ）では、選択範囲を維持（コピー待ち）
                }
            }
            return Ok(false);
        }

        // SGR 1006 が有効でない場合は、現在サポートしていないため送信しない
        if !self.buffer.is_sgr_mouse_encoding_enabled() {
            return Ok(false);
        }

        // 座標が変わっていない移動（ホバーまたはドラッグ）は抑制する
        if (event.button == MouseButton::None || event.is_drag)
            && self.buffer.get_last_mouse_pos() == Some((event.x, event.y))
        {
            return Ok(false);
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

        if should_send && let Some(seq) = self.translator.translate_mouse(event) {
            log::debug!(
                "Sending mouse VT sequence: {:?}",
                String::from_utf8_lossy(&seq)
            );
            self.reset_viewport();
            self.buffer.set_last_mouse_pos(Some((event.x, event.y)));
            self.send_input(&seq)?;
            return Ok(true);
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::input_value::{Modifiers, MouseButton};
    use crate::domain::model::terminal_config_value::TerminalConfig;
    use crate::domain::repository::clipboard_repository::ClipboardRepository;
    use crate::domain::repository::configuration_repository::ConfigurationRepository;
    use crate::domain::repository::key_translator_repository::KeyTranslatorRepository;
    use crate::domain::repository::terminal_output_repository::TerminalOutputRepository;
    use std::sync::{Arc, Mutex};

    struct MockOutputRepo {
        sent: Arc<Mutex<Vec<Vec<u8>>>>,
    }
    impl TerminalOutputRepository for MockOutputRepo {
        fn send_input(&self, data: &[u8]) -> std::io::Result<()> {
            self.sent.lock().unwrap().push(data.to_vec());
            Ok(())
        }
        fn resize(&self, _cols: u16, _rows: u16) -> std::io::Result<()> {
            Ok(())
        }
    }

    struct MockConfigRepo;
    impl ConfigurationRepository for MockConfigRepo {
        fn load(&self) -> TerminalConfig {
            TerminalConfig::default()
        }
        fn save(&self, _config: &TerminalConfig) -> Result<(), ConfigError> {
            Ok(())
        }
        fn get_terminal_config(&self) -> TerminalConfig {
            TerminalConfig::default()
        }
    }

    struct MockTranslator;
    impl KeyTranslatorRepository for MockTranslator {
        fn translate(&self, _key: crate::domain::model::input_value::InputKey) -> Option<Vec<u8>> {
            None
        }
        fn translate_mouse(&self, _event: MouseEvent) -> Option<Vec<u8>> {
            None
        }
    }

    struct MockClipboardRepo {
        text: Arc<Mutex<String>>,
    }
    impl ClipboardRepository for MockClipboardRepo {
        fn get_text(&self) -> Result<String, String> {
            Ok(self.text.lock().unwrap().clone())
        }
        fn set_text(&self, text: &str) -> Result<(), String> {
            *self.text.lock().unwrap() = text.to_string();
            Ok(())
        }
    }

    #[test]
    fn test_handle_mouse_event_paste() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let clipboard_text = Arc::new(Mutex::new("hello".to_string()));
        let mut workflow = TerminalWorkflow::new(
            80,
            25,
            Box::new(MockOutputRepo { sent: sent.clone() }),
            Box::new(MockConfigRepo),
            Box::new(MockTranslator),
            Box::new(MockClipboardRepo {
                text: clipboard_text.clone(),
            }),
            false,
        );

        let event = MouseEvent::new(
            MouseButton::Right,
            10,
            10,
            Modifiers::none(),
            false, // Down
            false,
        );

        let result = workflow.handle_mouse_event(event).unwrap();
        assert!(result);
        assert_eq!(sent.lock().unwrap().get(0).unwrap(), b"hello");
    }

    #[test]
    fn test_handle_mouse_event_no_paste_if_tracking_on() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let clipboard_text = Arc::new(Mutex::new("hello".to_string()));
        let mut workflow = TerminalWorkflow::new(
            80,
            25,
            Box::new(MockOutputRepo { sent: sent.clone() }),
            Box::new(MockConfigRepo),
            Box::new(MockTranslator),
            Box::new(MockClipboardRepo {
                text: clipboard_text.clone(),
            }),
            false,
        );

        // トラッキングを有効にする
        use crate::domain::model::terminal_types_entity::MouseTrackingMode;
        workflow
            .buffer
            .set_mouse_tracking_mode(MouseTrackingMode::Default);
        workflow.buffer.set_sgr_mouse_encoding(true);

        let event = MouseEvent::new(
            MouseButton::Right,
            10,
            10,
            Modifiers::none(),
            false, // Down
            false,
        );

        let result = workflow.handle_mouse_event(event).unwrap();
        // トラッキング有効時は MockTranslator が None を返すので false になるはず
        assert!(!result);
        assert!(sent.lock().unwrap().is_empty());
    }

    #[test]
    fn test_handle_mouse_event_cursor_move_right() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let clipboard_text = Arc::new(Mutex::new("".to_string()));
        let mut workflow = TerminalWorkflow::new(
            80,
            25,
            Box::new(MockOutputRepo { sent: sent.clone() }),
            Box::new(MockConfigRepo),
            Box::new(MockTranslator),
            Box::new(MockClipboardRepo {
                text: clipboard_text.clone(),
            }),
            false,
        );

        // カーソルを (5, 5) に配置
        workflow.buffer.move_cursor_to_pos(6, 6); // 1-based

        let event = MouseEvent::new(
            MouseButton::Left,
            10, // 10列目をクリック
            5,  // 5行目（カーソルと同一行）
            Modifiers::none(),
            true, // Up
            false,
        );

        let result = workflow.handle_mouse_event(event).unwrap();
        assert!(result);
        // 右に 5 回移動するはず (\x1b[C が 5回)
        let expected = b"\x1b[C\x1b[C\x1b[C\x1b[C\x1b[C";
        assert_eq!(sent.lock().unwrap().get(0).unwrap(), expected);
        assert!(workflow.buffer.get_selection_range().is_none());
    }

    #[test]
    fn test_handle_mouse_event_cursor_move_left() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let clipboard_text = Arc::new(Mutex::new("".to_string()));
        let mut workflow = TerminalWorkflow::new(
            80,
            25,
            Box::new(MockOutputRepo { sent: sent.clone() }),
            Box::new(MockConfigRepo),
            Box::new(MockTranslator),
            Box::new(MockClipboardRepo {
                text: clipboard_text.clone(),
            }),
            false,
        );

        // カーソルを (10, 5) に配置
        workflow.buffer.move_cursor_to_pos(6, 11); // 1-based

        let event = MouseEvent::new(
            MouseButton::Left,
            5, // 5列目をクリック
            5, // 5行目
            Modifiers::none(),
            true, // Up
            false,
        );

        let result = workflow.handle_mouse_event(event).unwrap();
        assert!(result);
        // 左に 5 回移動するはず (\x1b[D が 5回)
        let expected = b"\x1b[D\x1b[D\x1b[D\x1b[D\x1b[D";
        assert_eq!(sent.lock().unwrap().get(0).unwrap(), expected);
        assert!(workflow.buffer.get_selection_range().is_none());
    }

    #[test]
    fn test_handle_mouse_event_selection_drag() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let clipboard_text = Arc::new(Mutex::new("".to_string()));
        let mut workflow = TerminalWorkflow::new(
            80,
            25,
            Box::new(MockOutputRepo { sent: sent.clone() }),
            Box::new(MockConfigRepo),
            Box::new(MockTranslator),
            Box::new(MockClipboardRepo {
                text: clipboard_text.clone(),
            }),
            false,
        );

        // ダウン時 (10, 5)
        let event_down = MouseEvent::new(
            MouseButton::Left,
            10,
            5,
            Modifiers::none(),
            false, // Down
            false, // is_drag: false
        );

        let result = workflow.handle_mouse_event(event_down).unwrap();
        // Down時は初期化されるが再描画は必要ない（描画側で同一座標は非表示にするため）
        assert!(!result);
        let range = workflow.buffer.get_selection_range().unwrap();
        assert_eq!(
            range,
            (
                SelectionPoint {
                    x: 10,
                    logical_row: 5,
                },
                SelectionPoint {
                    x: 10,
                    logical_row: 5,
                },
            )
        );

        // ドラッグ移動 (20, 6)
        let event_move = MouseEvent::new(
            MouseButton::Left,
            20,
            6,
            Modifiers::none(),
            false,
            true, // is_drag
        );

        let result = workflow.handle_mouse_event(event_move).unwrap();
        assert!(result);
        let range = workflow.buffer.get_selection_range().unwrap();
        assert_eq!(
            range,
            (
                SelectionPoint {
                    x: 10,
                    logical_row: 5,
                },
                SelectionPoint {
                    x: 20,
                    logical_row: 6,
                },
            )
        );

        // アップ（移動後）
        let event_up = MouseEvent::new(
            MouseButton::Left,
            20,
            6,
            Modifiers::none(),
            true, // Up
            false,
        );
        let result = workflow.handle_mouse_event(event_up).unwrap();
        assert!(!result);
        // 範囲は維持されているはず
        let range = workflow.buffer.get_selection_range().unwrap();
        assert_eq!(
            range,
            (
                SelectionPoint {
                    x: 10,
                    logical_row: 5,
                },
                SelectionPoint {
                    x: 20,
                    logical_row: 6,
                },
            )
        );

        // クリックで解除
        let event_down_again =
            MouseEvent::new(MouseButton::Left, 5, 5, Modifiers::none(), false, false);
        workflow.handle_mouse_event(event_down_again).unwrap();
        // Downの瞬間に以前の選択はクリアされ、新しい (5,5)-(5,5) で初期化される
        let range = workflow.buffer.get_selection_range().unwrap();
        assert_eq!(
            range,
            (
                SelectionPoint {
                    x: 5,
                    logical_row: 5,
                },
                SelectionPoint {
                    x: 5,
                    logical_row: 5,
                },
            )
        );

        let event_up_again =
            MouseEvent::new(MouseButton::Left, 5, 5, Modifiers::none(), true, false);
        workflow.handle_mouse_event(event_up_again).unwrap();
        // ドラッグなしのUp（クリック確定）で範囲がNoneになる
        assert!(workflow.buffer.get_selection_range().is_none());
    }

    #[test]
    fn test_handle_mouse_event_copy_on_right_click() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let clipboard_text = Arc::new(Mutex::new("".to_string()));
        let mut workflow = TerminalWorkflow::new(
            80,
            25,
            Box::new(MockOutputRepo { sent: sent.clone() }),
            Box::new(MockConfigRepo),
            Box::new(MockTranslator),
            Box::new(MockClipboardRepo {
                text: clipboard_text.clone(),
            }),
            false,
        );

        // テキストを書き込む
        workflow.buffer.print_cell('A');
        workflow.buffer.print_cell('B');
        workflow.buffer.flush_pending_cluster();

        // 範囲を選択 (0,0) to (1,0) -> "AB"
        workflow.buffer.set_selection_range(Some((
            SelectionPoint {
                x: 0,
                logical_row: 0,
            },
            SelectionPoint {
                x: 1,
                logical_row: 0,
            },
        )));

        // 右クリック
        let event = MouseEvent::new(MouseButton::Right, 10, 10, Modifiers::none(), false, false);

        let result = workflow.handle_mouse_event(event).unwrap();
        assert!(result);
        assert_eq!(*clipboard_text.lock().unwrap(), "AB");
        assert!(workflow.buffer.get_selection_range().is_none());
        assert!(sent.lock().unwrap().is_empty()); // 貼り付けは行われない
    }

    #[test]
    fn test_handle_key_event_copy_on_ctrl_c() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let clipboard_text = Arc::new(Mutex::new("".to_string()));
        let mut workflow = TerminalWorkflow::new(
            80,
            25,
            Box::new(MockOutputRepo { sent: sent.clone() }),
            Box::new(MockConfigRepo),
            Box::new(MockTranslator),
            Box::new(MockClipboardRepo {
                text: clipboard_text.clone(),
            }),
            false,
        );

        // テキストを書き込む
        workflow.buffer.print_cell('X');
        workflow.buffer.print_cell('Y');
        workflow.buffer.flush_pending_cluster();

        // 範囲を選択 (0,0) to (1,0) -> "XY"
        workflow.buffer.set_selection_range(Some((
            SelectionPoint {
                x: 0,
                logical_row: 0,
            },
            SelectionPoint {
                x: 1,
                logical_row: 0,
            },
        )));

        // Ctrl+C (VK_C = 0x43)
        let key = InputKey::new(
            0x43,
            Modifiers {
                is_ctrl_pressed: true,
                is_shift_pressed: false,
                is_alt_pressed: false,
            },
        );

        let result = workflow.handle_key_event(key).unwrap();
        assert_eq!(result, Some(Vec::new())); // Handled (empty seq)
        assert_eq!(*clipboard_text.lock().unwrap(), "XY");
        assert!(workflow.buffer.get_selection_range().is_none());
    }

    #[test]
    fn test_handle_key_event_no_copy_if_no_selection() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let clipboard_text = Arc::new(Mutex::new("".to_string()));

        struct MockTranslatorWithCtrlC;
        impl KeyTranslatorRepository for MockTranslatorWithCtrlC {
            fn translate(&self, key: InputKey) -> Option<Vec<u8>> {
                if key.vk_code == 0x43 && key.modifiers.is_ctrl_pressed {
                    Some(vec![3])
                } else {
                    None
                }
            }
            fn translate_mouse(&self, _event: MouseEvent) -> Option<Vec<u8>> {
                None
            }
        }

        let mut workflow = TerminalWorkflow::new(
            80,
            25,
            Box::new(MockOutputRepo { sent: sent.clone() }),
            Box::new(MockConfigRepo),
            Box::new(MockTranslatorWithCtrlC),
            Box::new(MockClipboardRepo {
                text: clipboard_text.clone(),
            }),
            false,
        );

        // 選択範囲なし
        let key = InputKey::new(
            0x43,
            Modifiers {
                is_ctrl_pressed: true,
                is_shift_pressed: false,
                is_alt_pressed: false,
            },
        );

        let result = workflow.handle_key_event(key).unwrap();
        assert_eq!(result, Some(vec![3])); // Standard translation
        assert_eq!(*clipboard_text.lock().unwrap(), "");
    }

    #[test]
    fn test_handle_key_event_paste_on_ctrl_v() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let clipboard_text = Arc::new(Mutex::new("paste content".to_string()));
        let mut workflow = TerminalWorkflow::new(
            80,
            25,
            Box::new(MockOutputRepo { sent: sent.clone() }),
            Box::new(MockConfigRepo),
            Box::new(MockTranslator),
            Box::new(MockClipboardRepo {
                text: clipboard_text.clone(),
            }),
            false,
        );

        // Ctrl+V (VK_V = 0x56)
        let key = InputKey::new(
            0x56,
            Modifiers {
                is_ctrl_pressed: true,
                is_shift_pressed: false,
                is_alt_pressed: false,
            },
        );

        let result = workflow.handle_key_event(key).unwrap();
        assert_eq!(result, Some(Vec::new())); // Handled (empty seq)
        assert_eq!(sent.lock().unwrap().get(0).unwrap(), b"paste content");
    }
}
