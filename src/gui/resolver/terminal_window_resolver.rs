use crate::application::TerminalWorkflow;
use crate::gui::driver::terminal_gui_driver::{CompositionInfo, TerminalGuiDriver};
use crate::gui::driver::scroll_gui_driver::ScrollGuiDriver;
use crate::infra::repository::conpty_repository_impl::DummyOutputRepository;
use crate::infra::repository::emeditor_config_repository_impl::EmEditorConfigRepositoryImpl;
use std::sync::{Arc, Mutex, OnceLock};
use windows::Win32::Foundation::HWND;

static TERMINAL_DATA: OnceLock<Arc<Mutex<TerminalWindowResolver>>> = OnceLock::new();

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
        let config_repo = Box::new(EmEditorConfigRepositoryImpl::new());
        self.service = TerminalWorkflow::new(80, 25, output_repo, config_repo);
    }
}

pub fn get_terminal_data() -> Arc<Mutex<TerminalWindowResolver>> {
    TERMINAL_DATA
        .get_or_init(|| {
            let output_repo = Box::new(DummyOutputRepository);
            let config_repo = Box::new(EmEditorConfigRepositoryImpl::new());

            Arc::new(Mutex::new(TerminalWindowResolver {
                service: TerminalWorkflow::new(80, 25, output_repo, config_repo),
                renderer: TerminalGuiDriver::new(),
                window_handle: None,
                composition: None,
                scroll_manager: ScrollGuiDriver::new(),
            }))
        })
        .clone()
}
