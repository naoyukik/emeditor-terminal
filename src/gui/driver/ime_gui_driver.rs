use std::ffi::c_void;
use std::mem::size_of;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, POINT, RECT};
use windows::Win32::Graphics::Gdi::InvalidateRect;
use windows::Win32::UI::Input::Ime::{
    ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext, ImmSetCandidateWindow,
    ImmSetCompositionWindow, CANDIDATEFORM, CFS_EXCLUDE, CFS_FORCE_POSITION, CFS_POINT,
    COMPOSITIONFORM, GCS_COMPSTR, GCS_RESULTSTR,
};
use windows::Win32::UI::Input::KeyboardAndMouse::GetFocus;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateCaret, DestroyCaret, GetClientRect, HideCaret, SetCaretPos, ShowCaret,
};

use crate::gui::driver::terminal_gui_driver::TerminalGuiDriver;

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
            // Create a small caret
            let _ = CreateCaret(hwnd, None, 1, 1);
            // Some IMEs need to see the caret "active" to anchor to it.
            // We show and then hide it immediately to keep it invisible but active.
            let _ = ShowCaret(hwnd);
            let _ = HideCaret(hwnd);
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
    cursor_pos: (usize, usize),
    viewport_offset: usize,
    renderer: &TerminalGuiDriver,
    caret: Option<&CaretHandle>,
) {
    // CRITICAL: Only sync if this window actually has focus.
    // This prevents interfering with the parent EmEditor window's IME.
    unsafe {
        if GetFocus() != hwnd {
            return;
        }
    }

    // Convert absolute cursor Y to screen-relative Y
    let relative_y = cursor_pos.1.saturating_sub(viewport_offset);

    if let Some((pixel_x, pixel_y)) = renderer.cell_to_pixel(cursor_pos.0, relative_y) {
        // 0. Clamp coordinates to client area to prevent disappearing off-screen
        let mut client_rect = RECT::default();
        let (clamped_x, clamped_y) = unsafe {
            if GetClientRect(hwnd, &mut client_rect).is_ok() {
                let margin = 5;
                (
                    pixel_x.clamp(client_rect.left, client_rect.right - margin),
                    pixel_y.clamp(client_rect.top, client_rect.bottom - margin),
                )
            } else {
                (pixel_x, pixel_y)
            }
        };

        log::debug!(
            "Syncing system caret: cursor=({},{}), offset={}, pixel=({}, {}), clamped=({}, {})",
            cursor_pos.0,
            cursor_pos.1,
            viewport_offset,
            pixel_x,
            pixel_y,
            clamped_x,
            clamped_y
        );

        // 1. Update system caret position (for IME anchoring)
        if let Some(c) = caret {
            c.set_position(clamped_x, clamped_y);
        }

        // 2. Update IME composition and candidate window positions
        unsafe {
            let himc = ImmGetContext(hwnd);
            if !himc.0.is_null() {
                let metrics = renderer.get_metrics().cloned().unwrap_or(crate::gui::driver::terminal_gui_driver::TerminalMetrics { char_height: 16, base_width: 8 });

                // For composition position, use clamped coordinates
                let pt_current_pos = POINT {
                    x: clamped_x,
                    y: clamped_y,
                };

                // The exclusion area is the rectangle we want the IME list to AVOID covering.
                let rc_exclude = RECT {
                    left: clamped_x,
                    top: clamped_y,
                    right: clamped_x + metrics.base_width,
                    bottom: clamped_y + metrics.char_height,
                };

                // Use CFS_FORCE_POSITION to bypass IME's own adjustments
                let comp_form = COMPOSITIONFORM {
                    dwStyle: CFS_FORCE_POSITION,
                    ptCurrentPos: pt_current_pos,
                    rcArea: RECT::default(),
                };
                let _ = ImmSetCompositionWindow(himc, &comp_form);

                // Set candidate window position for all possible indices (0-3)
                // to ensure maximum compatibility with different IME implementations.
                for i in 0..4 {
                    let cand_form = CANDIDATEFORM {
                        dwIndex: i,
                        dwStyle: CFS_EXCLUDE,
                        ptCurrentPos: pt_current_pos,
                        rcArea: rc_exclude,
                    };
                    let _ = ImmSetCandidateWindow(himc, &cand_form);
                }

                let _ = ImmReleaseContext(hwnd, himc);
            }
        }
    }
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

/// Results of IME processing to be handled by the upper layer.
#[derive(PartialEq, Debug)]
pub enum ImeResult {
    NotHandled,
    Result(String),
    Composition(String),
}

/// Processes WM_IME_COMPOSITION and returns the result to be integrated into Workflow.
pub fn handle_composition(
    hwnd: HWND,
    lparam: LPARAM,
    cursor_pos: (usize, usize),
    viewport_offset: usize,
    renderer: &TerminalGuiDriver,
    caret: Option<&CaretHandle>,
) -> ImeResult {
    log::debug!("WM_IME_COMPOSITION: lparam={:?}", lparam);

    let mut result = ImeResult::NotHandled;

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
                    result = ImeResult::Result(result_str);
                }
                let _ = ImmReleaseContext(hwnd, himc);
            }
        }
    }

    if (lparam.0 as u32 & GCS_COMPSTR.0) != 0 {
        sync_system_caret(hwnd, cursor_pos, viewport_offset, renderer, caret);
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
                    if result == ImeResult::NotHandled {
                        result = ImeResult::Composition(comp_str);
                    }
                }
                let _ = ImmReleaseContext(hwnd, himc);
            }
        }
    }

    result
}

pub fn handle_start_composition(hwnd: HWND) {
    log::debug!("WM_IME_STARTCOMPOSITION");
    unsafe {
        let _ = InvalidateRect(hwnd, None, BOOL(0));
    }
}

pub fn handle_end_composition(hwnd: HWND) {
    log::debug!("WM_IME_ENDCOMPOSITION");
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
