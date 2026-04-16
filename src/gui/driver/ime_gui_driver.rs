use std::ffi::c_void;
use std::mem::size_of;
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
use windows::Win32::UI::Input::Ime::{
    ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext, ImmSetCandidateWindow,
    ImmSetCompositionWindow, CANDIDATEFORM, CFS_CANDIDATEPOS, CFS_EXCLUDE, CFS_RECT,
    COMPOSITIONFORM, GCS_COMPSTR, GCS_RESULTSTR,
};
use windows::Win32::UI::Input::KeyboardAndMouse::GetFocus;
use windows::Win32::UI::WindowsAndMessaging::{CreateCaret, DestroyCaret, SetCaretPos};

use crate::gui::driver::terminal_gui_driver::TerminalGuiDriver;

/// RAII handle for the system caret.
/// This is required for SetCaretPos to work correctly and anchor the IME window.
/// Managed strictly on the UI thread to satisfy Win32 thread-locality.
pub struct CaretHandle {
    hwnd: HWND,
    created: bool,
    thread_id: u32,
}

// Win32 Caret API is thread-local to the UI thread.
// We implement Send/Sync to allow it to be stored in TerminalWindowResolver (Mutex),
// but we MUST ensure it is created and destroyed on the same UI thread.
unsafe impl Send for CaretHandle {}
unsafe impl Sync for CaretHandle {}

impl CaretHandle {
    /// Creates a new invisible system caret for the specified window with given dimensions.
    /// Matching the caret size to the actual font dimensions helps IME to position correctly.
    pub fn new(hwnd: HWND, width: i32, height: i32) -> Self {
        log::info!(
            "Creating system caret handle for HWND {:?} size {}x{}",
            hwnd,
            width,
            height
        );
        let thread_id = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };
        let created = unsafe {
            // Create a caret matching character dimensions
            CreateCaret(hwnd, None, width, height).is_ok()
        };
        if !created {
            log::error!("Failed to create system caret for HWND {:?}", hwnd);
        }
        Self {
            hwnd,
            created,
            thread_id,
        }
    }

    /// Moves the system caret to the specified pixel coordinates.
    pub fn set_position(&self, x: i32, y: i32) {
        if self.created {
            // Caret operations must happen on the same thread
            let current_thread = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };
            if current_thread == self.thread_id {
                unsafe {
                    let _ = SetCaretPos(x, y);
                }
            } else {
                log::warn!("Caret::set_position called from non-UI thread");
            }
        }
    }
}

impl Drop for CaretHandle {
    fn drop(&mut self) {
        if self.created {
            let current_thread = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };
            if current_thread == self.thread_id {
                log::info!("Destroying system caret handle for HWND {:?}", self.hwnd);
                unsafe {
                    let _ = DestroyCaret();
                }
            } else {
                log::error!(
                    "CaretHandle dropped on wrong thread! created={}, current={}",
                    self.thread_id,
                    current_thread
                );
                // We cannot call DestroyCaret safely on another thread as it's thread-local.
                // This indicates a bug in lifecycle management.
            }
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
        let focus_hwnd = GetFocus();
        if focus_hwnd != hwnd {
            log::debug!(
                "sync_system_caret: Skipped sync (No Focus). hwnd={:?}, focus_hwnd={:?}",
                hwnd,
                focus_hwnd
            );
            return;
        }
    }

    // Convert absolute cursor Y to screen-relative Y
    let relative_y = cursor_pos.1.saturating_sub(viewport_offset);

    if let Some((pixel_x, pixel_y)) = renderer.cell_to_pixel(cursor_pos.0, relative_y) {
        log::debug!(
            "sync_system_caret: cell=({},{}), pixel=({}, {})",
            cursor_pos.0,
            cursor_pos.1,
            pixel_x,
            pixel_y
        );

        // 1. Update system caret position (for IME anchoring)
        if let Some(c) = caret {
            c.set_position(pixel_x, pixel_y);
        }

        // 2. Update IME composition and candidate window positions
        unsafe {
            let himc = ImmGetContext(hwnd);
            if !himc.0.is_null() {
                let metrics = renderer.get_metrics().cloned().unwrap_or(
                    crate::gui::driver::terminal_gui_driver::TerminalMetrics {
                        char_height: 16,
                        base_width: 8,
                    },
                );
                let pt_current_pos = POINT {
                    x: pixel_x,
                    y: pixel_y,
                };

                // The exclusion area is the rectangle we want the IME list to AVOID covering.
                // We use the full character height and a reasonable width for the exclusion.
                let rc_exclude = RECT {
                    left: pixel_x,
                    top: pixel_y,
                    right: pixel_x + (metrics.base_width * 2),
                    bottom: pixel_y + metrics.char_height,
                };

                // Style for composition window
                // Using CFS_RECT ensures the candidate window stays near the caret but avoids overlapping.
                let comp_form = COMPOSITIONFORM {
                    dwStyle: CFS_RECT,
                    ptCurrentPos: pt_current_pos,
                    rcArea: rc_exclude,
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

pub fn handle_composition(
    hwnd: HWND,
    lparam: LPARAM,
    cursor_pos: (usize, usize),
    viewport_offset: usize,
    renderer: &TerminalGuiDriver,
    caret: Option<&CaretHandle>,
) -> ImeResult {
    log::info!(
        "WM_IME_COMPOSITION: lparam={:?}, cursor_pos={:?}",
        lparam,
        cursor_pos
    );

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
                    result = ImeResult::Result(String::from_utf16_lossy(&buffer));
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

pub fn handle_start_composition(_hwnd: HWND) {
    log::debug!("WM_IME_STARTCOMPOSITION");
}

pub fn handle_end_composition(_hwnd: HWND) {
    log::debug!("WM_IME_ENDCOMPOSITION");
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
