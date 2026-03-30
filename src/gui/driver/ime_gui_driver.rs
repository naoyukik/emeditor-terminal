use std::ffi::c_void;
use std::mem::size_of;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::Graphics::Gdi::InvalidateRect;
use windows::Win32::UI::Input::Ime::{
    ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext, ImmSetCompositionWindow, CFS_POINT,
    COMPOSITIONFORM, GCS_COMPSTR, GCS_RESULTSTR,
};
use windows::Win32::UI::WindowsAndMessaging::{CreateCaret, DestroyCaret, SetCaretPos};

use crate::application::TerminalWorkflow;
use crate::gui::driver::terminal_gui_driver::{CompositionInfo, TerminalGuiDriver};

/// RAII handle for the system caret.
/// This is required for SetCaretPos to work correctly and anchor the IME window.
pub struct CaretHandle {
    hwnd: HWND,
}

// SAFETY: Caret operations in Win32 are thread-local, but we ensure all access
// happens on the UI thread.
unsafe impl Send for CaretHandle {}
unsafe impl Sync for CaretHandle {}

impl CaretHandle {
    /// Creates a new invisible system caret for the specified window.
    /// The size is set to 1x1 to be practically invisible but still serve as an anchor.
    pub fn new(hwnd: HWND) -> Self {
        log::info!("Creating system caret handle for HWND {:?}", hwnd);
        unsafe {
            // Width 1, Height 1 (practically invisible)
            let _ = CreateCaret(hwnd, None, 1, 1);
        }
        Self { hwnd }
    }

    /// Moves the system caret to the specified pixel coordinates.
    pub fn set_position(&self, x: i32, y: i32) {
        unsafe {
            let _ = SetCaretPos(x, y);
        }
    }
}

impl Drop for CaretHandle {
    fn drop(&mut self) {
        log::info!("Destroying system caret handle for HWND {:?}", self.hwnd);
        unsafe {
            let _ = DestroyCaret();
        }
    }
}

/// Updates both the system caret and the IME composition window position.
pub fn sync_system_caret(
    hwnd: HWND,
    service: &TerminalWorkflow,
    renderer: &TerminalGuiDriver,
    caret: Option<&CaretHandle>,
) {
    if let Some((pixel_x, pixel_y)) = {
        let (cursor_x, cursor_y) = service.get_buffer().get_cursor_pos();
        renderer.cell_to_pixel(cursor_x, cursor_y)
    } {
        log::debug!("Syncing system caret: pixel=({}, {})", pixel_x, pixel_y);

        // 1. Update system caret position (for IME anchoring)
        if let Some(c) = caret {
            c.set_position(pixel_x, pixel_y);
        }

        // 2. Update IME composition window position
        unsafe {
            let himc = ImmGetContext(hwnd);
            if !himc.0.is_null() {
                let form = COMPOSITIONFORM {
                    dwStyle: CFS_POINT,
                    ptCurrentPos: windows::Win32::Foundation::POINT {
                        x: pixel_x,
                        y: pixel_y,
                    },
                    rcArea: windows::Win32::Foundation::RECT::default(),
                };
                let _ = ImmSetCompositionWindow(himc, &form);
                let _ = ImmReleaseContext(hwnd, himc);
            }
        }
    }
}

/// Deprecated: use sync_system_caret instead.
pub fn update_window_position(
    hwnd: HWND,
    service: &TerminalWorkflow,
    renderer: &TerminalGuiDriver,
) {
    sync_system_caret(hwnd, service, renderer, None);
}

pub fn is_composing(hwnd: HWND) -> bool {
    unsafe {
        let himc = ImmGetContext(hwnd);
        if himc.0.is_null() {
            return false;
        }
        let len = ImmGetCompositionStringW(himc, GCS_COMPSTR, None, 0);
        let _ = ImmReleaseContext(hwnd, himc);
        if len < 0 {
            return false;
        }
        len > 0
    }
}

pub fn handle_composition(
    hwnd: HWND,
    lparam: LPARAM,
    service: &mut TerminalWorkflow,
    renderer: &TerminalGuiDriver,
    composition: &mut Option<CompositionInfo>,
) -> bool {
    log::debug!("WM_IME_COMPOSITION: lparam={:?}", lparam);
    let mut handled = false;

    if (lparam.0 as u32 & GCS_RESULTSTR.0) != 0 {
        unsafe {
            let himc = ImmGetContext(hwnd);
            if !himc.0.is_null() {
                let len_bytes = ImmGetCompositionStringW(himc, GCS_RESULTSTR, None, 0);
                if len_bytes >= 0 {
                    let len_u16 = (len_bytes as usize) / size_of::<u16>();
                    let mut buffer = vec![0u16; len_u16];
                    let _ = ImmGetCompositionStringW(
                        himc,
                        GCS_RESULTSTR,
                        Some(buffer.as_mut_ptr() as *mut c_void),
                        len_bytes as u32,
                    );
                    let result_str = String::from_utf16_lossy(&buffer);
                    log::info!("IME Result: '{}'", result_str);

                    let _ = service.send_input(result_str.as_bytes());
                    *composition = None;
                    let _ = InvalidateRect(hwnd, None, BOOL(0));
                    handled = true;
                }
                let _ = ImmReleaseContext(hwnd, himc);
            }
        }
    }

    if (lparam.0 as u32 & GCS_COMPSTR.0) != 0 {
        update_window_position(hwnd, service, renderer);
        unsafe {
            let himc = ImmGetContext(hwnd);
            if !himc.0.is_null() {
                let len_bytes = ImmGetCompositionStringW(himc, GCS_COMPSTR, None, 0);
                if len_bytes >= 0 {
                    let len_u16 = (len_bytes as usize) / size_of::<u16>();
                    let mut buffer = vec![0u16; len_u16];
                    let _ = ImmGetCompositionStringW(
                        himc,
                        GCS_COMPSTR,
                        Some(buffer.as_mut_ptr() as *mut c_void),
                        len_bytes as u32,
                    );

                    let comp_str = String::from_utf16_lossy(&buffer);
                    log::info!("IME Composition: '{}' (len={})", comp_str, len_u16);

                    if comp_str.is_empty() {
                        *composition = None;
                    } else {
                        *composition = Some(CompositionInfo { text: comp_str });
                    }

                    let _ = InvalidateRect(hwnd, None, BOOL(0));
                    handled = true;
                }
                let _ = ImmReleaseContext(hwnd, himc);
            }
        }
    }

    handled
}

pub fn handle_start_composition(hwnd: HWND, service: &mut TerminalWorkflow) {
    log::debug!("WM_IME_STARTCOMPOSITION");
    service.reset_viewport();
    unsafe {
        let _ = InvalidateRect(hwnd, None, BOOL(0));
    }
}

pub fn handle_end_composition(hwnd: HWND, composition: &mut Option<CompositionInfo>) {
    log::debug!("WM_IME_ENDCOMPOSITION");
    *composition = None;
    unsafe {
        let _ = InvalidateRect(hwnd, None, BOOL(0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::driver::terminal_gui_driver::TerminalMetrics;

    #[test]
    fn test_cell_to_pixel_translation() {
        let mut renderer = TerminalGuiDriver::new();
        // Setup mock metrics
        renderer.metrics = Some(TerminalMetrics {
            char_height: 20,
            base_width: 10,
        });

        assert_eq!(renderer.cell_to_pixel(0, 0), Some((0, 0)));
        assert_eq!(renderer.cell_to_pixel(5, 3), Some((50, 60)));
        assert_eq!(renderer.cell_to_pixel(100, 50), Some((1000, 1000)));
    }
}
