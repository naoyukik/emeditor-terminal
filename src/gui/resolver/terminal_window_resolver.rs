use crate::application::TerminalWorkflow;
use crate::domain::model::window_id_value::WindowId;
use crate::gui::common::SendHWND;
use crate::gui::driver::scroll_gui_driver::ScrollGuiDriver;
use crate::gui::driver::terminal_gui_driver::{CompositionInfo, TerminalGuiDriver};
use crate::infra::repository::conpty_repository_impl::DummyOutputRepository;
use crate::infra::repository::emeditor_config_repository_impl::EmEditorConfigRepositoryImpl;
use std::sync::{Arc, Mutex, OnceLock};
use windows::Win32::Foundation::HWND;

static TERMINAL_DATA: OnceLock<Arc<Mutex<TerminalWindowResolver>>> = OnceLock::new();

pub struct TerminalWindowResolver {
    pub service: TerminalWorkflow,
    pub renderer: TerminalGuiDriver,
    pub window_handle: Option<SendHWND>,
    pub composition: Option<CompositionInfo>,
    pub scroll_manager: ScrollGuiDriver,
}

impl TerminalWindowResolver {
    /// TerminalServiceをダミー実装でリセットし、ConPTY等のリソースを解放する
    pub fn reset_service(&mut self) {
        let output_repo = Box::new(DummyOutputRepository);
        let config_repo_for_service = Box::new(EmEditorConfigRepositoryImpl::new(WindowId(
            HWND::default().0 as isize,
        )));
        self.service = TerminalWorkflow::new(80, 25, output_repo, config_repo_for_service);
    }
}

pub fn get_terminal_data() -> Arc<Mutex<TerminalWindowResolver>> {
    TERMINAL_DATA
        .get_or_init(|| {
            let output_repo = Box::new(DummyOutputRepository);
            let config_repo_for_service = Box::new(EmEditorConfigRepositoryImpl::new(WindowId(
                HWND::default().0 as isize,
            )));

            Arc::new(Mutex::new(TerminalWindowResolver {
                service: TerminalWorkflow::new(80, 25, output_repo, config_repo_for_service),
                renderer: TerminalGuiDriver::new(),
                window_handle: None,
                composition: None,
                scroll_manager: ScrollGuiDriver::new(),
            }))
        })
        .clone()
}
