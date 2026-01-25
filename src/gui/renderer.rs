use crate::domain::terminal::{Color, TerminalBuffer};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, RECT, SIZE};
use windows::Win32::Graphics::Gdi::{
    CreateFontW, DeleteObject, ExtTextOutW, GetTextExtentPoint32W, GetTextMetricsW, InvertRect,
    SelectObject, SetBkColor, SetTextColor, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_QUALITY,
    ETO_OPAQUE, ETO_OPTIONS, FF_MODERN, FIXED_PITCH, FW_NORMAL, HDC, HFONT, HGDIOBJ,
    OUT_DEFAULT_PRECIS, TEXTMETRICW,
};

#[derive(Clone, Debug)]
pub struct CompositionData {
    pub text: String,
}

pub struct TerminalMetrics {
    pub char_height: i32,
    pub base_width: i32,
}

pub struct SendHFONT(pub HFONT);
unsafe impl Send for SendHFONT {}
unsafe impl Sync for SendHFONT {}

pub struct TerminalRenderer {
    font: Option<SendHFONT>,
    metrics: Option<TerminalMetrics>,
}

impl Default for TerminalRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalRenderer {
    pub fn new() -> Self {
        Self {
            font: None,
            metrics: None,
        }
    }

    pub fn clear_resources(&mut self) {
        if let Some(send_h_font) = self.font.take() {
            unsafe {
                let _ = DeleteObject(HGDIOBJ(send_h_font.0 .0));
            }
            log::info!("TerminalRenderer: Cached font handle deleted");
        }
    }

    pub fn get_metrics(&self) -> Option<&TerminalMetrics> {
        self.metrics.as_ref()
    }

    fn ensure_font(&mut self, hdc: HDC) {
        if self.font.is_none() {
            unsafe {
                let h_font = CreateFontW(
                    16,
                    0,
                    0,
                    0,
                    FW_NORMAL.0 as i32,
                    0,
                    0,
                    0,
                    DEFAULT_CHARSET.0 as u32,
                    OUT_DEFAULT_PRECIS.0 as u32,
                    CLIP_DEFAULT_PRECIS.0 as u32,
                    DEFAULT_QUALITY.0 as u32,
                    (FIXED_PITCH.0 | FF_MODERN.0) as u32,
                    w!("Consolas"),
                );

                if h_font.0.is_null() {
                    log::error!("TerminalRenderer: Failed to create font 'Consolas'");
                    return;
                }

                self.font = Some(SendHFONT(h_font));

                let old_font = SelectObject(hdc, HGDIOBJ(h_font.0));
                let mut tm = TEXTMETRICW::default();
                let _ = GetTextMetricsW(hdc, &mut tm);

                let zero_utf16: &[u16] = &[0x0030]; // "0"
                let mut size = SIZE::default();
                let _ = GetTextExtentPoint32W(hdc, zero_utf16, &mut size);

                self.metrics = Some(TerminalMetrics {
                    char_height: tm.tmHeight,
                    base_width: size.cx,
                });

                log::info!(
                    "TerminalRenderer: Font initialized: Consolas, Height={}, BaseWidth={}",
                    tm.tmHeight,
                    size.cx
                );
                let _ = SelectObject(hdc, old_font);
            }
        }
    }

    fn color_to_colorref(&self, color: Color) -> COLORREF {
        match color {
            Color::Default => COLORREF(0x00FFFFFF), // White
            Color::Ansi(n) => match n {
                0 => COLORREF(0x00000000),  // Black
                1 => COLORREF(0x00000080),  // Red
                2 => COLORREF(0x00008000),  // Green
                3 => COLORREF(0x00008080),  // Yellow
                4 => COLORREF(0x00800000),  // Blue
                5 => COLORREF(0x00800080),  // Magenta
                6 => COLORREF(0x00808000),  // Cyan
                7 => COLORREF(0x00C0C0C0),  // White/Gray
                8 => COLORREF(0x00808080),  // Bright Black (Gray)
                9 => COLORREF(0x000000FF),  // Bright Red
                10 => COLORREF(0x0000FF00), // Bright Green
                11 => COLORREF(0x0000FFFF), // Bright Yellow
                12 => COLORREF(0x00FF0000), // Bright Blue
                13 => COLORREF(0x00FF00FF), // Bright Magenta
                14 => COLORREF(0x00FFFF00), // Bright Cyan
                15 => COLORREF(0x00FFFFFF), // Bright White
                _ => COLORREF(0x00C0C0C0),
            },
            Color::Rgb(r, g, b) => COLORREF(r as u32 | ((g as u32) << 8) | ((b as u32) << 16)),
        }
    }

    pub fn render(
        &mut self,
        hdc: HDC,
        client_rect: &RECT,
        buffer: &TerminalBuffer,
        composition: Option<&CompositionData>,
    ) {
        self.ensure_font(hdc);

        let (font, metrics) = match (&self.font, &self.metrics) {
            (Some(f), Some(m)) => (f.0, m),
            _ => return,
        };

        unsafe {
            let old_font = SelectObject(hdc, HGDIOBJ(font.0));
            let char_height = metrics.char_height;
            let base_width = metrics.base_width;

            // 背景色を黒に設定
            SetBkColor(hdc, COLORREF(0x00000000));

            let mut current_y = 0;
            let (cursor_x, cursor_y) = buffer.get_cursor_pos();

            for (idx, line) in buffer.get_lines().iter().enumerate() {
                let mut x_offset = 0;
                let mut cell_idx = 0;

                while cell_idx < line.len() {
                    let start_color = line[cell_idx].fg_color;
                    let mut run_text = String::new();
                    let mut run_dx = Vec::new();

                    while cell_idx < line.len() && line[cell_idx].fg_color == start_color {
                        let cell = &line[cell_idx];
                        run_text.push(cell.c);
                        let w = TerminalBuffer::char_display_width(cell.c) as i32 * base_width;
                        run_dx.push(w);
                        run_dx.extend(std::iter::repeat_n(0, cell.c.len_utf16().saturating_sub(1)));
                        cell_idx += 1;
                    }

                    let wide_run: Vec<u16> = run_text.encode_utf16().collect();
                    let run_pixel_width: i32 = run_dx.iter().sum();

                    let run_rect = RECT {
                        left: x_offset,
                        top: current_y,
                        right: x_offset + run_pixel_width,
                        bottom: current_y + char_height,
                    };

                    SetTextColor(hdc, self.color_to_colorref(start_color));
                    let _ = ExtTextOutW(
                        hdc,
                        x_offset,
                        current_y,
                        ETO_OPTIONS(ETO_OPAQUE.0),
                        Some(&run_rect),
                        PCWSTR(wide_run.as_ptr()),
                        wide_run.len() as u32,
                        Some(run_dx.as_ptr()),
                    );

                    x_offset += run_pixel_width;
                }

                // 行の残りを背景色で塗りつぶす
                if x_offset < client_rect.right {
                    let fill_rect = RECT {
                        left: x_offset,
                        top: current_y,
                        right: client_rect.right,
                        bottom: current_y + char_height,
                    };
                    let _ = ExtTextOutW(
                        hdc,
                        x_offset,
                        current_y,
                        ETO_OPTIONS(ETO_OPAQUE.0),
                        Some(&fill_rect),
                        PCWSTR::null(),
                        0,
                        None,
                    );
                }

                if idx == cursor_y {
                    let display_cols = buffer.get_display_width_up_to(cursor_y, cursor_x);
                    let cursor_pixel_x = display_cols as i32 * base_width;

                    if let Some(comp) = composition {
                        self.render_composition(
                            hdc,
                            cursor_pixel_x,
                            current_y,
                            char_height,
                            base_width,
                            comp,
                        );
                    } else if buffer.is_cursor_visible() {
                        let rect = RECT {
                            left: cursor_pixel_x,
                            top: current_y,
                            right: cursor_pixel_x + 2,
                            bottom: current_y + char_height,
                        };
                        let _ = InvertRect(hdc, &rect);
                    }
                }
                current_y += char_height;
            }
            let _ = SelectObject(hdc, old_font);
        }
    }

    fn render_composition(
        &self,
        hdc: HDC,
        x: i32,
        y: i32,
        char_height: i32,
        base_width: i32,
        comp: &CompositionData,
    ) {
        let comp_wide: Vec<u16> = comp.text.encode_utf16().collect();
        let mut comp_dx = Vec::with_capacity(comp_wide.len());
        let mut pixel_width = 0;
        for c in comp.text.chars() {
            let w = TerminalBuffer::char_display_width(c) as i32 * base_width;
            comp_dx.push(w);
            comp_dx.extend(std::iter::repeat_n(0, c.len_utf16().saturating_sub(1)));
            pixel_width += w;
        }

        let comp_rect = RECT {
            left: x,
            top: y,
            right: x + pixel_width,
            bottom: y + char_height,
        };

        unsafe {
            let _ = ExtTextOutW(
                hdc,
                x,
                y,
                ETO_OPTIONS(ETO_OPAQUE.0),
                Some(&comp_rect),
                PCWSTR(comp_wide.as_ptr()),
                comp_wide.len() as u32,
                Some(comp_dx.as_ptr()),
            );

            let pen = windows::Win32::Graphics::Gdi::CreatePen(
                windows::Win32::Graphics::Gdi::PS_SOLID,
                1,
                COLORREF(0x00FF0000),
            );

            if !pen.0.is_null() {
                let old_pen = SelectObject(hdc, HGDIOBJ(pen.0));
                let _ = windows::Win32::Graphics::Gdi::MoveToEx(
                    hdc,
                    comp_rect.left,
                    comp_rect.bottom - 1,
                    None,
                );
                let _ = windows::Win32::Graphics::Gdi::LineTo(
                    hdc,
                    comp_rect.right,
                    comp_rect.bottom - 1,
                );
                let _ = SelectObject(hdc, old_pen);
                let _ = DeleteObject(HGDIOBJ(pen.0));
            } else {
                log::error!("TerminalRenderer: Failed to create pen for composition underline");
            }
        }
    }
}
