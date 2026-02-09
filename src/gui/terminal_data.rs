use crate::application::TerminalService;
use crate::gui::renderer::{CompositionData, TerminalRenderer};
use crate::gui::scroll::ScrollManager;
use crate::infra::repository::conpty_repository_impl::DummyOutputRepository;
use crate::infra::repository::emeditor_config_repository_impl::EmEditorConfigRepositoryImpl;
use std::sync::{Arc, Mutex, OnceLock};
use windows::Win32::Foundation::HWND;

static TERMINAL_DATA: OnceLock<Arc<Mutex<TerminalData>>> = OnceLock::new();

#[derive(Clone, Copy)]
pub struct SendHWND(pub HWND);

unsafe impl Send for SendHWND {}
unsafe impl Sync for SendHWND {}

pub struct TerminalData {
    pub service: TerminalService,
    pub renderer: TerminalRenderer,
    pub window_handle: Option<SendHWND>,
    pub composition: Option<CompositionData>,
    pub scroll_manager: ScrollManager,
}

pub fn get_terminal_data() -> Arc<Mutex<TerminalData>> {
    TERMINAL_DATA
        .get_or_init(|| {
            let output_repo = Box::new(DummyOutputRepository);
            let config_repo = Box::new(EmEditorConfigRepositoryImpl::new());

            Arc::new(Mutex::new(TerminalData {
                service: TerminalService::new(80, 25, output_repo, config_repo),
                renderer: TerminalRenderer::new(),
                window_handle: None,
                composition: None,
                scroll_manager: ScrollManager::new(),
            }))
        })
        .clone()
}
