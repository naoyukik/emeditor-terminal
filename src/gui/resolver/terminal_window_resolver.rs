use crate::application::TerminalWorkflow;
use crate::domain::model::window_id_value::WindowId;
use crate::gui::common::SendHWND;
use crate::gui::driver::ime_gui_driver::CaretHandle;
use crate::gui::driver::scroll_gui_driver::ScrollGuiDriver;
use crate::gui::driver::terminal_gui_driver::{CompositionInfo, TerminalGuiDriver};
use std::sync::{Arc, Mutex, OnceLock};

static TERMINAL_DATA: OnceLock<Arc<Mutex<TerminalWindowResolver>>> = OnceLock::new();

pub struct TerminalWindowResolver {
    pub service: TerminalWorkflow,
    pub renderer: TerminalGuiDriver,
    pub window_handle: Option<SendHWND>,
    pub editor_handle: Option<SendHWND>,
    pub composition: Option<CompositionInfo>,
    pub scroll_manager: ScrollGuiDriver,
    pub caret: Option<CaretHandle>,
    pub is_conpty_started: bool,
}

impl TerminalWindowResolver {
    /// ターミナルデータを遅延初期化する。
    fn new_default() -> Self {
        use crate::domain::service::vt_sequence_translator_domain_service::VtSequenceTranslatorDomainService;
        use crate::infra::repository::conpty_repository_impl::DummyOutputRepository;
        use crate::infra::repository::emeditor_config_repository_impl::EmEditorConfigRepositoryImpl;

        let output_repo = Box::new(DummyOutputRepository);
        let config_repo = Box::new(EmEditorConfigRepositoryImpl::new(WindowId(0)));
        let translator = Box::new(VtSequenceTranslatorDomainService::new());
        let is_dark = crate::infra::driver::emeditor_io_driver::is_system_dark_mode();
        let service = TerminalWorkflow::new(80, 25, output_repo, config_repo, translator, is_dark);

        TerminalWindowResolver {
            service,
            renderer: TerminalGuiDriver::new(),
            window_handle: None,
            editor_handle: None,
            composition: None,
            scroll_manager: ScrollGuiDriver::new(),
            caret: None,
            is_conpty_started: false,
        }
    }

    /// TerminalServiceをリセットする (外部から新しい Workflow を注入)
    pub fn reset_service(&mut self, new_service: TerminalWorkflow) {
        self.service = new_service;
        self.caret = None;
        self.is_conpty_started = false;
    }
}

pub fn get_terminal_data() -> Arc<Mutex<TerminalWindowResolver>> {
    TERMINAL_DATA
        .get_or_init(|| Arc::new(Mutex::new(TerminalWindowResolver::new_default())))
        .clone()
}
