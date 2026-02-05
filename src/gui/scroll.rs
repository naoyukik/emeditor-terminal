use windows::Win32::UI::WindowsAndMessaging::{
    SB_BOTTOM, SB_LINEDOWN, SB_LINEUP, SB_PAGEDOWN, SB_PAGEUP, SB_THUMBPOSITION, SB_THUMBTRACK,
    SB_TOP, SCROLLBAR_COMMAND,
};

#[derive(Debug, PartialEq)]
pub enum ScrollAction {
    ScrollTo(usize),
    ScrollBy(isize),
    Redraw,
    None,
}

pub struct ScrollManager {
    pub min: i32,
    pub max: i32,
    pub page: u32,
    pub pos: i32,
}

impl ScrollManager {
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
        let manager = ScrollManager::new();
        assert_eq!(manager.min, 0);
        assert_eq!(manager.max, 0);
        assert_eq!(manager.page, 0);
        assert_eq!(manager.pos, 0);
    }

    #[test]
    fn test_handle_vscroll() {
        let mut manager = ScrollManager::new();
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
        let mut manager = ScrollManager::new();
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
        let manager = ScrollManager::new();

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
