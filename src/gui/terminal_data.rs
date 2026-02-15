use crate::application::TerminalService;
use crate::gui::renderer::{CompositionInfo, TerminalRenderer};
use crate::gui::scroll::ScrollManager;
use crate::infra::repository::conpty_repository_impl::DummyOutputRepository;
use crate::infra::repository::emeditor_config_repository_impl::EmEditorConfigRepositoryImpl;
use std::sync::{Arc, Mutex, OnceLock};
use windows::Win32::Foundation::HWND;

static TERMINAL_DATA: OnceLock<Arc<Mutex<TerminalData>>> = OnceLock::new();

#[derive(Clone, Copy)]
pub struct SendHWND(pub HWND);

// SAFETY:
// - HWND は OS によって管理されるウィンドウハンドルであり、このラッパー型は
//   そのハンドル値を他スレッドへ送る目的のためだけに使用する。
// - 他スレッドからこの HWND に対して行ってよい操作は、PostMessageW など
//   Win32 がスレッドセーフであると明示している API の呼び出しに限定する。
// - SendHWND を介してウィンドウプロシージャを直接呼び出したり、非スレッドセーフな
//   Win32 API（例: 一部の GDI 関数やスレッドアフィニティを前提とした API）を呼び出さないこと。
// - 上記の前提を満たす限り、SendHWND / &SendHWND を複数スレッド間で送受信・共有しても、
//   Rust の意味でのデータ競合や未定義動作は発生しないものとみなせる。
unsafe impl Send for SendHWND {}
unsafe impl Sync for SendHWND {}

pub struct TerminalData {
    pub service: TerminalService,
    pub renderer: TerminalRenderer,
    pub window_handle: Option<SendHWND>,
    pub composition: Option<CompositionInfo>,
    pub scroll_manager: ScrollManager,
}

impl TerminalData {
    /// TerminalServiceをダミー実装でリセットし、ConPTY等のリソースを解放する
    pub fn reset_service(&mut self) {
        let output_repo = Box::new(DummyOutputRepository);
        let config_repo = Box::new(EmEditorConfigRepositoryImpl::new());
        self.service = TerminalService::new(80, 25, output_repo, config_repo);
    }
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
