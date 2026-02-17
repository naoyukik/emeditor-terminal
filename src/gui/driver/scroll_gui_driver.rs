use crate::gui::resolver::terminal_window_resolver::get_terminal_data;
use std::mem::size_of;
use windows::Win32::Foundation::{BOOL, HWND};
use windows::Win32::UI::WindowsAndMessaging::{
    SB_BOTTOM, SB_LINEDOWN, SB_LINEUP, SB_PAGEDOWN, SB_PAGEUP, SB_THUMBPOSITION, SB_THUMBTRACK,
    SB_TOP, SB_VERT, SCROLLBAR_COMMAND,
};

const SIF_RANGE: u32 = 0x0001;
const SIF_PAGE: u32 = 0x0002;
const SIF_POS: u32 = 0x0004;
const SIF_DISABLENOSCROLL: u32 = 0x0008;
const SIF_TRACKPOS: u32 = 0x0010;
const SIF_ALL: u32 = SIF_RANGE | SIF_PAGE | SIF_POS | SIF_TRACKPOS;

#[link(name = "user32")]
extern "system" {
    fn SetScrollInfo(hwnd: HWND, nbar: i32, lpsi: *const SCROLLINFO, redraw: BOOL) -> i32;
}

#[repr(C)]
#[allow(non_snake_case)]
#[allow(clippy::upper_case_acronyms)]
struct SCROLLINFO {
    cbSize: u32,
    fMask: u32,
    nMin: i32,
    nMax: i32,
    nPage: u32,
    nPos: i32,
    nTrackPos: i32,
}

pub fn update_window_scroll_info(hwnd: HWND) {
    let data_arc = get_terminal_data();
    let mut window_data = data_arc.lock().unwrap();

    let history_count = window_data.service.get_history_count() as i32;
    let viewport_offset = window_data.service.get_viewport_offset() as i32;
    let height = window_data.service.buffer.height as i32;

    // Update ScrollGuiDriver state
    window_data.scroll_manager.min = 0;
    // The scrollable range is [0, history_count]
    // The ScrollGuiDriver uses nMax = history + page - 1 logic internally if needed, or we set it here.
    // Let's align with existing logic: nMax = history_count + page_size - 1
    // And nPos = history_count - viewport_offset

    let page_size = height;
    window_data.scroll_manager.max = history_count + page_size - 1;
    window_data.scroll_manager.page = page_size as u32;
    window_data.scroll_manager.pos = history_count - viewport_offset;

    let si = SCROLLINFO {
        cbSize: size_of::<SCROLLINFO>() as u32,
        fMask: SIF_ALL | SIF_DISABLENOSCROLL,
        nMin: window_data.scroll_manager.min,
        nMax: window_data.scroll_manager.max,
        nPage: window_data.scroll_manager.page,
        nPos: window_data.scroll_manager.pos,
        nTrackPos: 0,
    };

    unsafe {
        SetScrollInfo(hwnd, SB_VERT.0, &si, BOOL(1));
    }
}

#[derive(Debug, PartialEq)]
pub enum ScrollAction {
    ScrollTo(usize),
    ScrollBy(isize),
    #[allow(dead_code)]
    Redraw,
    None,
}

pub struct ScrollGuiDriver {
    pub min: i32,
    pub max: i32,
    pub page: u32,
    pub pos: i32,
}

impl ScrollGuiDriver {
    pub fn new() -> Self {
        Self {
            min: 0,
            max: 0,
            page: 0,
            pos: 0,
        }
    }

    pub fn handle_vscroll(&mut self, wparam: usize, _lparam: isize) -> ScrollAction {
        let request = SCROLLBAR_COMMAND((wparam & 0xFFFF) as i32);
        match request {
            SB_LINEUP => ScrollAction::ScrollBy(1),
            SB_LINEDOWN => ScrollAction::ScrollBy(-1),
            SB_PAGEUP => ScrollAction::ScrollBy(self.page as isize),
            SB_PAGEDOWN => ScrollAction::ScrollBy(-(self.page as isize)),
            SB_THUMBTRACK | SB_THUMBPOSITION => {
                let pos = (wparam >> 16) & 0xFFFF;
                let history_count = self.max - self.page as i32 + 1;
                let target_pos = pos as i32;

                // viewport_offset = history_count - target_pos
                // Avoid underflow
                if target_pos <= history_count {
                    ScrollAction::ScrollTo((history_count - target_pos) as usize)
                } else {
                    ScrollAction::ScrollTo(0)
                }
            }
            SB_TOP => {
                let history_count = self.max - self.page as i32 + 1;
                ScrollAction::ScrollTo(history_count as usize)
            }
            SB_BOTTOM => ScrollAction::ScrollTo(0),
            _ => ScrollAction::None,
        }
    }

    pub fn handle_mousewheel(&self, wparam: usize) -> ScrollAction {
        let z_delta = (wparam >> 16) as i16;
        // 3 lines per notch
        let lines = (z_delta as f32 / 120.0 * 3.0) as isize;
        if lines != 0 {
            ScrollAction::ScrollBy(lines)
        } else {
            ScrollAction::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let manager = ScrollGuiDriver::new();
        assert_eq!(manager.min, 0);
        assert_eq!(manager.max, 0);
        assert_eq!(manager.page, 0);
        assert_eq!(manager.pos, 0);
    }

    #[test]
    fn test_handle_vscroll() {
        let mut manager = ScrollGuiDriver::new();
        manager.page = 10;
        manager.max = 109; // history 100 + page 10 - 1 = 109

        // SB_LINEUP -> ScrollBy(1)
        let action = manager.handle_vscroll(SB_LINEUP.0 as usize, 0);
        assert_eq!(action, ScrollAction::ScrollBy(1));

        // SB_LINEDOWN -> ScrollBy(-1)
        let action = manager.handle_vscroll(SB_LINEDOWN.0 as usize, 0);
        assert_eq!(action, ScrollAction::ScrollBy(-1));

        // SB_PAGEUP -> ScrollBy(page)
        let action = manager.handle_vscroll(SB_PAGEUP.0 as usize, 0);
        assert_eq!(action, ScrollAction::ScrollBy(10));

        // SB_PAGEDOWN -> ScrollBy(-page)
        let action = manager.handle_vscroll(SB_PAGEDOWN.0 as usize, 0);
        assert_eq!(action, ScrollAction::ScrollBy(-10));

        // SB_TOP
        let action = manager.handle_vscroll(SB_TOP.0 as usize, 0);
        assert_eq!(action, ScrollAction::ScrollTo(100)); // history count

        // SB_BOTTOM
        let action = manager.handle_vscroll(SB_BOTTOM.0 as usize, 0);
        assert_eq!(action, ScrollAction::ScrollTo(0));
    }

    #[test]
    fn test_handle_thumbtrack() {
        let mut manager = ScrollGuiDriver::new();
        manager.page = 10;
        manager.max = 109; // history 100

        // Pos 0 (Top) -> history count (100)
        // wparam low word is request, high word is pos
        let wparam = SB_THUMBTRACK.0 as usize;
        let action = manager.handle_vscroll(wparam, 0);
        assert_eq!(action, ScrollAction::ScrollTo(100));

        // Pos 50 -> 100 - 50 = 50
        let wparam = (SB_THUMBTRACK.0 as usize) | (50 << 16);
        let action = manager.handle_vscroll(wparam, 0);
        assert_eq!(action, ScrollAction::ScrollTo(50));

        // Pos 100 (Bottom) -> 100 - 100 = 0
        let wparam = (SB_THUMBTRACK.0 as usize) | (100 << 16);
        let action = manager.handle_vscroll(wparam, 0);
        assert_eq!(action, ScrollAction::ScrollTo(0));
    }

    #[test]
    fn test_handle_mousewheel() {
        let manager = ScrollGuiDriver::new();

        // Delta 120 (1 notch up) -> 3 lines
        let wparam = (120i16 as u16 as usize) << 16;
        let action = manager.handle_mousewheel(wparam);
        assert_eq!(action, ScrollAction::ScrollBy(3));

        // Delta -120 (1 notch down) -> -3 lines
        let wparam = ((-120i16) as u16 as usize) << 16;
        let action = manager.handle_mousewheel(wparam);
        assert_eq!(action, ScrollAction::ScrollBy(-3));
    }
}
