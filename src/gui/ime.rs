use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::Graphics::Gdi::InvalidateRect;
use windows::Win32::UI::Input::Ime::{
    ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext, ImmSetCompositionWindow, CFS_POINT,
    COMPOSITIONFORM, GCS_COMPSTR, GCS_RESULTSTR,
};
use windows::Win32::UI::WindowsAndMessaging::SetCaretPos;
use std::ffi::c_void;
use std::mem::size_of;

use crate::application::TerminalService;
use crate::gui::renderer::{CompositionData, TerminalRenderer};

/// Helper to update IME window position based on the terminal cursor
pub fn update_window_position(hwnd: HWND, service: &TerminalService, renderer: &TerminalRenderer) {
    if let Some(metrics) = renderer.get_metrics() {
        let (cursor_x, cursor_y) = service.buffer.get_cursor_pos();
        let display_cols = service.buffer.get_display_width_up_to(cursor_y, cursor_x);

        let pixel_x = display_cols as i32 * metrics.base_width;
        let pixel_y = cursor_y as i32 * metrics.char_height;

        log::debug!(
            "Updating IME Window position: cursor=({}, {}), pixel=({}, {})",
            cursor_x,
            cursor_y,
            pixel_x,
            pixel_y
        );

        unsafe {
            // Update system caret position (IME uses this as reference)
            let _ = SetCaretPos(pixel_x, pixel_y);

            // Explicitly set composition window position
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

/// Helper to check if IME is composing
pub fn is_composing(hwnd: HWND) -> bool {
    unsafe {
        let himc = ImmGetContext(hwnd);
        if himc.0.is_null() {
            return false;
        }
        let len = ImmGetCompositionStringW(himc, GCS_COMPSTR, None, 0);
        let _ = ImmReleaseContext(hwnd, himc);
        if len < 0 {
            // ImmGetCompositionStringW failed; treat as not composing
            return false;
        }
        len > 0
    }
}

/// Handles WM_IME_COMPOSITION message
/// Returns true if handled, false if DefWindowProc should be called
pub fn handle_composition(
    hwnd: HWND,
    lparam: LPARAM,
    service: &mut TerminalService,
    renderer: &TerminalRenderer,
    composition: &mut Option<CompositionData>,
) -> bool {
    log::debug!("WM_IME_COMPOSITION: lparam={:?}", lparam);
    let mut handled = false;

    // Handle Result String (Committed)
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

                    // Send result string to ConPTY
                    let _ = service.send_input(result_str.as_bytes());

                    // Clear composition data on commit
                    *composition = None;

                    let _ = InvalidateRect(hwnd, None, BOOL(0));
                    handled = true;
                }
                let _ = ImmReleaseContext(hwnd, himc);
            }
        }
    }

    // Handle Composition String (In-progress)
    if (lparam.0 as u32 & GCS_COMPSTR.0) != 0 {
        update_window_position(hwnd, service, renderer);
        unsafe {
            let himc = ImmGetContext(hwnd);
            if !himc.0.is_null() {
                // Get size first
                let len_bytes = ImmGetCompositionStringW(himc, GCS_COMPSTR, None, 0);
                if len_bytes >= 0 {
                    let len_u16 = (len_bytes as usize) / size_of::<u16>();
                    let mut buffer = vec![0u16; len_u16];

                    // Get content
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
                        *composition = Some(CompositionData { text: comp_str });
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

pub fn handle_start_composition(hwnd: HWND, service: &mut TerminalService) {
    log::debug!("WM_IME_STARTCOMPOSITION");
    // Snap on Input for IME
    service.reset_viewport();
    
    // Invalidate rect is typically handled by caller or update_scroll_info logic which might follow
    unsafe { let _ = InvalidateRect(hwnd, None, BOOL(0)); }
}

pub fn handle_end_composition(hwnd: HWND, composition: &mut Option<CompositionData>) {
     log::debug!("WM_IME_ENDCOMPOSITION");
    // Ensure composition is cleared when IME ends
    *composition = None;
    unsafe {
        let _ = InvalidateRect(hwnd, None, BOOL(0));
    }
}