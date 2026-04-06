use std::ffi::c_void;
use std::mem::size_of;
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
use windows::Win32::UI::Input::Ime::{
    ImmAssociateContext, ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext,
    ImmSetCandidateWindow, ImmSetCompositionWindow, CANDIDATEFORM, COMPOSITIONFORM,
    GCS_COMPSTR, GCS_RESULTSTR,
};
use windows::Win32::UI::Input::KeyboardAndMouse::GetFocus;
use windows::Win32::UI::WindowsAndMessaging::{CreateCaret, DestroyCaret, SetCaretPos};

use crate::domain::model::terminal_buffer_entity::TerminalBufferEntity;
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
        log::info!("Creating system caret handle for HWND {:?} size {}x{}", hwnd, width, height);
        let thread_id = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };
        let created = unsafe {
            // Create a caret matching character dimensions
            if CreateCaret(hwnd, None, width, height).is_ok() {
                // We DON'T call ShowCaret here to avoid interfering with our custom cursor.
                // The system caret is only used as a reference point for IME positioning.
                true
            } else {
                false
            }
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

/// Ensures the window has a valid IME context associated with it.
/// Required for some child windows in hosted environments like EmEditor.
pub fn associate_ime_context(hwnd: HWND) {
    unsafe {
        let himc = ImmGetContext(hwnd);
        if !himc.0.is_null() {
            log::info!("Associating IME context for HWND {:?}", hwnd);
            let _ = ImmAssociateContext(hwnd, himc);
            let _ = ImmReleaseContext(hwnd, himc);
        } else {
            log::warn!("Failed to get IME context for associate_ime_context(HWND {:?})", hwnd);
        }
    }
}

/// Updates both the system caret and the IME composition window position.
pub fn sync_system_caret(
    hwnd: HWND,
    cursor_pos: (usize, usize),
    is_visible: bool,
    buffer: &TerminalBufferEntity,
    renderer: &TerminalGuiDriver,
    caret: Option<&CaretHandle>,
    font_face: &str,
) {
    // CRITICAL: Only sync if this window actually has focus.
    // This prevents interfering with the parent EmEditor window's IME.
    unsafe {
        if GetFocus() != hwnd {
            return;
        }
    }

    if !is_visible {
        // If the cursor is hidden, it's often "parked" at the screen edge by TUI apps.
        // We should NOT sync the system caret or IME to a hidden cursor's position.
        return;
    }

    // Convert logical cursor coordinates to visual pixel coordinates.
    let visual_y = buffer.get_visual_row(cursor_pos.1);
    let visual_x = buffer.get_visual_column(cursor_pos.0, cursor_pos.1);

    let relative_y = visual_y.unwrap_or_else(|| {
        // Fallback if cursor is off-screen (should not happen for active IME)
        cursor_pos.1.saturating_sub(buffer.get_viewport_offset())
    });

    // Prefer renderer's measured pixel position (accurate for variable-width chars)
    // ONLY if the logical position matches what was rendered.
    // Otherwise fall back to grid calculation to stay up-to-date with recent app movements.
    let pixel_pos = renderer
        .get_last_cursor_pixel_pos(cursor_pos)
        .or_else(|| {
            let metrics = renderer.get_metrics()?;
            let py = relative_y as i32 * metrics.char_height;
            let px = visual_x as i32 * metrics.base_width;
            Some((px, py))
        });

    if let Some((mut pixel_x, mut pixel_y)) = pixel_pos {
        // Detailed logging for coordinate verification
        let mut client_rect = RECT::default();
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut client_rect);
        }

        log::info!(
            "Syncing IME: client_pos=({}, {}), cursor=({:?}), visual=({:?}, {:?}), v_offset={}, buffer_len={}, buf_height={}, client_rect={:?}, visible={}, alt_screen={}, font={}",
            pixel_x, pixel_y, cursor_pos, visual_x, visual_y, buffer.get_viewport_offset(), buffer.get_buffer_line_count(), buffer.get_height(), client_rect, is_visible, buffer.is_alternate_screen(), font_face
        );

        // If client_rect has an offset, we MUST include it for IMM32/Caret to align with BitBlt.
        // Although GetClientRect usually returns 0,0, this handles hosted scenarios.
        pixel_x += client_rect.left;
        pixel_y += client_rect.top;

        // 1. Update system caret position (Always uses client coordinates)
        if let Some(c) = caret {
            c.set_position(pixel_x, pixel_y);
        }

        // 2. Update IME composition and candidate window positions
        unsafe {
            let himc = ImmGetContext(hwnd);
            if !himc.0.is_null() {
                let metrics = renderer.get_metrics().cloned().unwrap_or(crate::gui::driver::terminal_gui_driver::TerminalMetrics { char_height: 16, base_width: 8 });

                // Use CLIENT coordinates for IMM32.
                let pt_client = POINT {
                    x: pixel_x,
                    y: pixel_y,
                };

                // Log actual OS caret position to verify SetCaretPos success
                let mut os_caret_pos = POINT::default();
                let _ = windows::Win32::UI::WindowsAndMessaging::GetCaretPos(&mut os_caret_pos);
                log::info!("OS Caret Position: {:?}", os_caret_pos);

                // 1. Set the font size for the composition window
                // A negative lfHeight is often preferred for specifying pixel height.
                let mut lf = windows::Win32::Graphics::Gdi::LOGFONTW {
                    lfHeight: -metrics.char_height,
                    lfWidth: 0, // 0 for automatic aspect ratio
                    lfCharSet: windows::Win32::Graphics::Gdi::DEFAULT_CHARSET,
                    lfWeight: 400,
                    ..Default::default()
                };

                // Set font face name
                let face_name_wide: Vec<u16> = font_face.encode_utf16().collect();
                let len = std::cmp::min(face_name_wide.len(), lf.lfFaceName.len() - 1);
                lf.lfFaceName[..len].copy_from_slice(&face_name_wide[..len]);
                lf.lfFaceName[len] = 0;

                let res_font = windows::Win32::UI::Input::Ime::ImmSetCompositionFontW(himc, &lf);

                // 2. Set the composition window position
                // Use CFS_POINT | CFS_FORCE_POSITION for maximum compatibility and to force position.
                let comp_form = COMPOSITIONFORM {
                    dwStyle: windows::Win32::UI::Input::Ime::CFS_POINT | windows::Win32::UI::Input::Ime::CFS_FORCE_POSITION,
                    ptCurrentPos: pt_client,
                    rcArea: RECT::default(),
                };
                let res_comp = ImmSetCompositionWindow(himc, &comp_form);

                // 3. Set candidate window position for all possible indices (0-3)
                // Use CFS_CANDIDATEPOS with Client coordinates.
                let mut res_cand = true;
                for i in 0..4 {
                    let cand_form = CANDIDATEFORM {
                        dwIndex: i,
                        dwStyle: windows::Win32::UI::Input::Ime::CFS_CANDIDATEPOS,
                        ptCurrentPos: pt_client,
                        rcArea: RECT::default(),
                    };
                    if !ImmSetCandidateWindow(himc, &cand_form).as_bool() {
                        res_cand = false;
                    }
                }

                log::info!("IMM32 Calls (Client): Font={:?}, Comp={:?}, Cand={:?}, himc={:?}", res_font, res_comp, res_cand, himc.0);

                let _ = ImmReleaseContext(hwnd, himc);
            } else {
                log::warn!("ImmGetContext returned NULL himc for HWND {:?}", hwnd);
            }
        }
    } else {
        log::warn!("Could not determine pixel position for cursor {:?} (relative_y={})", cursor_pos, relative_y);
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
    buffer: &TerminalBufferEntity,
    renderer: &TerminalGuiDriver,
    caret: Option<&CaretHandle>,
    font_face: &str,
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
                    result = ImeResult::Result(String::from_utf16_lossy(&buffer));
                }
                let _ = ImmReleaseContext(hwnd, himc);
            }
        }
    }

    if (lparam.0 as u32 & GCS_COMPSTR.0) != 0 {
        // When composing, we always treat the cursor as visible to ensure synchronization.
        sync_system_caret(
            hwnd,
            cursor_pos,
            true,
            buffer,
            renderer,
            caret,
            font_face,
        );

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
