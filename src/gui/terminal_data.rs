use crate::application::TerminalService;
use crate::gui::renderer::{CompositionData, TerminalRenderer};
use crate::gui::scroll::ScrollManager;
use std::sync::{Arc, Mutex, OnceLock};
use windows::Win32::Foundation::HWND;

static TERMINAL_DATA: OnceLock<Arc<Mutex<TerminalData>>> = OnceLock::new();

#[derive(Clone, Copy)]
/// Wrapper around a Windows `HWND` handle that is treated as `Send` and `Sync`.
///
/// On Windows, many operations on `HWND` (such as `PostMessageW`) are documented
/// as cross-thread safe, but some operations must only be performed on the thread
/// that created/owns the window (for example, most UI updates and message loops).
pub struct SendHWND(pub HWND);

/// SAFETY:
/// - The `HWND` handle value itself may be moved across threads.
/// - Callers must only perform operations from other threads that the Windows
///   API documents as thread-safe for `HWND` (e.g., `PostMessageW`).
/// - Thread-affine operations must still be invoked on the thread that owns the window.
unsafe impl Send for SendHWND {}

/// SAFETY:
/// - Sharing an `HWND` between threads does not in itself cause data races, as
///   long as all threads confine thread-affine operations to the owning thread
///   and only perform cross-thread-safe operations from other threads.
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
            Arc::new(Mutex::new(TerminalData {
                service: TerminalService::new(80, 25),
                renderer: TerminalRenderer::new(),
                window_handle: None,
                composition: None,
                scroll_manager: ScrollManager::new(),
            }))
        })
        .clone()
}
