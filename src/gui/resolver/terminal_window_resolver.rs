use crate::application::TerminalWorkflow;
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
    pub composition: Option<CompositionInfo>,
    pub scroll_manager: ScrollGuiDriver,
    pub caret: Option<CaretHandle>,
    pub is_conpty_started: bool,
}

impl TerminalWindowResolver {
    /// ターミナルデータを初期化する。
    /// 
    /// このメソッドは lib.rs などの境界層でのみ呼び出されるべきである。
    pub fn init(service: TerminalWorkflow) {
        let resolver = TerminalWindowResolver {
            service,
            renderer: TerminalGuiDriver::new(),
            window_handle: None,
            composition: None,
            scroll_manager: ScrollGuiDriver::new(),
            caret: None,
            is_conpty_started: false,
        };
        let _ = TERMINAL_DATA.set(Arc::new(Mutex::new(resolver)));
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
        .get()
        .expect("TerminalWindowResolver not initialized! Call init() first.")
        .clone()
}
