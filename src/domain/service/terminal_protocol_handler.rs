use crate::domain::model::terminal_buffer_entity::TerminalBufferEntity;
use crate::domain::model::terminal_types_entity::{
    CursorStyle, MouseTrackingMode, TerminalAttribute, TerminalColor,
};
use vte::{Params, Perform};

/// ターミナルプロトコル（ANSI/VT100等）の解釈と実行を担うドメインサービス
pub(crate) struct TerminalProtocolHandler<'a> {
    buffer: &'a mut TerminalBufferEntity,
}

impl<'a> TerminalProtocolHandler<'a> {
    pub fn new(buffer: &'a mut TerminalBufferEntity) -> Self {
        Self { buffer }
    }

    fn get_param(&self, params: &Params, index: usize, default: u16) -> u16 {
        params
            .iter()
            .nth(index)
            .and_then(|p| p.first())
            .copied()
            .unwrap_or(default)
    }

    fn handle_decscusr(&mut self, params: &Params) {
        let n = params
            .iter()
            .last()
            .and_then(|subparams| subparams.first())
            .copied()
            .unwrap_or(1);
        let style = match n {
            0 | 1 => CursorStyle::BlinkingBlock,
            2 => CursorStyle::SteadyBlock,
            3 => CursorStyle::BlinkingUnderline,
            4 => CursorStyle::SteadyUnderline,
            5 => CursorStyle::BlinkingBar,
            6 => CursorStyle::SteadyBar,
            _ => CursorStyle::BlinkingBlock,
        };
        self.buffer.set_cursor_style(style);
    }

    fn handle_sgr(&mut self, params: &Params) {
        if params.is_empty() {
            self.buffer.set_attribute(TerminalAttribute::default());
            return;
        }
        let mut attr = self.buffer.get_current_attribute().clone();
        let mut iter = params.iter();
        while let Some(subparams) = iter.next() {
            let p = subparams.first().copied().unwrap_or(0);
            match p {
                0 => attr = TerminalAttribute::default(),
                1 => attr.is_bold = true,
                2 => attr.is_dim = true,
                3 => attr.is_italic = true,
                4 => attr.is_underline = true,
                7 => attr.is_inverse = true,
                27 => attr.is_inverse = false,
                9 => attr.is_strikethrough = true,
                22 => {
                    attr.is_bold = false;
                    attr.is_dim = false;
                }
                23 => attr.is_italic = false,
                24 => attr.is_underline = false,
                29 => attr.is_strikethrough = false,
                30..=37 => attr.fg = TerminalColor::Ansi((p - 30) as u8),
                38 => {
                    if let Some(type_p) = subparams.get(1).copied() {
                        match type_p {
                            5 => {
                                if let Some(color_idx) = subparams.get(2).copied() {
                                    attr.fg = TerminalColor::Xterm(color_idx as u8);
                                }
                            }
                            2 if subparams.len() >= 5 => {
                                attr.fg = TerminalColor::Rgb(
                                    subparams[2] as u8,
                                    subparams[3] as u8,
                                    subparams[4] as u8,
                                );
                            }
                            _ => {}
                        }
                    } else if let Some(next_subparams) = iter.next() {
                        let type_p = next_subparams.first().copied().unwrap_or(0);
                        match type_p {
                            5 => {
                                if let Some(color_sub) = iter.next()
                                    && let Some(color_idx) = color_sub.first().copied()
                                {
                                    attr.fg = TerminalColor::Xterm(color_idx as u8);
                                }
                            }
                            2 => {
                                let r = iter.next().and_then(|s| s.first()).copied();
                                let g = iter.next().and_then(|s| s.first()).copied();
                                let b = iter.next().and_then(|s| s.first()).copied();
                                if let (Some(r), Some(g), Some(b)) = (r, g, b) {
                                    attr.fg = TerminalColor::Rgb(r as u8, g as u8, b as u8);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                39 => attr.fg = TerminalColor::Default,
                40..=47 => attr.bg = TerminalColor::Ansi((p - 40) as u8),
                48 => {
                    if let Some(type_p) = subparams.get(1).copied() {
                        match type_p {
                            5 => {
                                if let Some(color_idx) = subparams.get(2).copied() {
                                    attr.bg = TerminalColor::Xterm(color_idx as u8);
                                }
                            }
                            2 if subparams.len() >= 5 => {
                                attr.bg = TerminalColor::Rgb(
                                    subparams[2] as u8,
                                    subparams[3] as u8,
                                    subparams[4] as u8,
                                );
                            }
                            _ => {}
                        }
                    } else if let Some(next_subparams) = iter.next() {
                        let type_p = next_subparams.first().copied().unwrap_or(0);
                        match type_p {
                            5 => {
                                if let Some(color_sub) = iter.next()
                                    && let Some(color_idx) = color_sub.first().copied()
                                {
                                    attr.bg = TerminalColor::Xterm(color_idx as u8);
                                }
                            }
                            2 => {
                                let r = iter.next().and_then(|s| s.first()).copied();
                                let g = iter.next().and_then(|s| s.first()).copied();
                                let b = iter.next().and_then(|s| s.first()).copied();
                                if let (Some(r), Some(g), Some(b)) = (r, g, b) {
                                    attr.bg = TerminalColor::Rgb(r as u8, g as u8, b as u8);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                49 => attr.bg = TerminalColor::Default,
                90..=97 => attr.fg = TerminalColor::Ansi((p - 90 + 8) as u8),
                100..=107 => attr.bg = TerminalColor::Ansi((p - 100 + 8) as u8),
                _ => {}
            }
        }
        self.buffer.set_attribute(attr);
    }
}

impl<'a> Perform for TerminalProtocolHandler<'a> {
    fn print(&mut self, c: char) {
        self.buffer.print_cell(c);
    }
    fn execute(&mut self, byte: u8) {
        self.buffer.flush_pending_cluster();
        match byte {
            0x08 => self.buffer.move_cursor_backward(1),
            0x09 => self.buffer.handle_tab(),
            0x0A..=0x0C => {
                self.buffer.move_cursor_to_col(0);
                self.buffer.index();
            }
            0x0D => self.buffer.move_cursor_to_col(0),
            _ => {}
        }
    }
    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        self.buffer.flush_pending_cluster();
    }
    fn put(&mut self, _byte: u8) {
        self.buffer.flush_pending_cluster();
    }
    fn unhook(&mut self) {
        self.buffer.flush_pending_cluster();
    }
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        self.buffer.flush_pending_cluster();
    }
    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, action: char) {
        self.buffer.flush_pending_cluster();
        match action {
            'm' => self.handle_sgr(params),
            'A' => self
                .buffer
                .move_cursor_up(self.get_param(params, 0, 1) as usize),
            'B' => self
                .buffer
                .move_cursor_down(self.get_param(params, 0, 1) as usize),
            '@' => self
                .buffer
                .insert_cells(self.get_param(params, 0, 1) as usize),
            'C' => self
                .buffer
                .move_cursor_forward(self.get_param(params, 0, 1) as usize),
            'D' => self
                .buffer
                .move_cursor_backward(self.get_param(params, 0, 1) as usize),
            'K' => self
                .buffer
                .erase_in_line(self.get_param(params, 0, 0) as u8),
            'P' => self
                .buffer
                .delete_cells(self.get_param(params, 0, 1) as usize),
            'X' => self
                .buffer
                .erase_cells(self.get_param(params, 0, 1) as usize),
            'H' | 'f' => self.buffer.move_cursor_to_pos(
                self.get_param(params, 0, 1) as usize,
                self.get_param(params, 1, 1) as usize,
            ),
            'J' => self
                .buffer
                .erase_in_display(self.get_param(params, 0, 0) as u8),
            'G' => self
                .buffer
                .move_cursor_to_col(self.get_param(params, 0, 1).saturating_sub(1) as usize),
            'd' => self
                .buffer
                .move_cursor_to_row(self.get_param(params, 0, 1).saturating_sub(1) as usize),
            'E' => {
                let n = self.get_param(params, 0, 1) as usize;
                self.buffer.move_cursor_down(n);
                self.buffer.move_cursor_to_col(0);
            }
            'F' => {
                let n = self.get_param(params, 0, 1) as usize;
                self.buffer.move_cursor_up(n);
                self.buffer.move_cursor_to_col(0);
            }
            'h' if intermediates.first() == Some(&b'?') => {
                for subparams in params.iter() {
                    for mode in subparams.iter() {
                        match mode {
                            6 => self.buffer.set_origin_mode(true),
                            25 => self.buffer.set_cursor_visible(true),
                            1000 => {
                                log::debug!("Enabling Default Mouse Tracking (1000)");
                                self.buffer
                                    .set_mouse_tracking_mode(MouseTrackingMode::Default);
                            }
                            1002 => {
                                log::debug!("Enabling Button Event Mouse Tracking (1002)");
                                self.buffer
                                    .set_mouse_tracking_mode(MouseTrackingMode::ButtonEvent);
                            }
                            1003 => {
                                log::debug!("Enabling Any Event Mouse Tracking (1003)");
                                self.buffer
                                    .set_mouse_tracking_mode(MouseTrackingMode::AnyEvent);
                            }
                            1006 => {
                                log::debug!("Enabling SGR Mouse Encoding (1006)");
                                self.buffer.set_sgr_mouse_encoding(true);
                            }
                            _ => {}
                        }
                    }
                }
            }
            'l' if intermediates.first() == Some(&b'?') => {
                for subparams in params.iter() {
                    for mode in subparams.iter() {
                        match mode {
                            6 => self.buffer.set_origin_mode(false),
                            25 => self.buffer.set_cursor_visible(false),
                            1000 | 1002 | 1003 => {
                                log::debug!("Disabling Mouse Tracking");
                                self.buffer.set_mouse_tracking_mode(MouseTrackingMode::None)
                            }
                            1006 => {
                                log::debug!("Disabling SGR Mouse Encoding");
                                self.buffer.set_sgr_mouse_encoding(false);
                            }
                            _ => {}
                        }
                    }
                }
            }
            'r' => self.buffer.set_scroll_region(
                self.get_param(params, 0, 1) as usize,
                self.get_param(params, 1, self.buffer.get_height() as u16) as usize,
            ),
            'S' => {
                for _ in 0..self.get_param(params, 0, 1) {
                    self.buffer.scroll_up();
                }
            }
            'T' => {
                for _ in 0..self.get_param(params, 0, 1) {
                    self.buffer.scroll_down();
                }
            }
            'L' => self
                .buffer
                .insert_lines(self.get_param(params, 0, 1) as usize),
            'M' => self
                .buffer
                .delete_lines(self.get_param(params, 0, 1) as usize),
            'q' if intermediates.first() == Some(&b' ') => {
                self.handle_decscusr(params);
            }
            _ => {}
        }
    }
    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
        self.buffer.flush_pending_cluster();
        match byte as char {
            '7' => self.buffer.save_cursor(),
            '8' => self.buffer.restore_cursor(),
            'M' => self.buffer.reverse_index(),
            'D' => self.buffer.index(),
            _ => {}
        }
    }
}
