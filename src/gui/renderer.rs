use crate::domain::terminal::{TerminalBuffer, TerminalColor};
use std::collections::HashMap;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, RECT, SIZE};
use windows::Win32::Graphics::Gdi::{
    CreateFontIndirectW, DeleteObject, ExtTextOutW, GetTextExtentPoint32W, GetTextMetricsW,
    InvertRect, SelectObject, SetBkColor, SetTextColor, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET,
    DEFAULT_QUALITY, ETO_OPAQUE, ETO_OPTIONS, FF_MODERN, FIXED_PITCH, FONT_CHARSET,
    FONT_CLIP_PRECISION, FONT_OUTPUT_PRECISION, FONT_QUALITY, FW_BOLD, FW_NORMAL, HDC, HFONT,
    HGDIOBJ, LOGFONTW, OUT_DEFAULT_PRECIS, TEXTMETRICW,
};

#[derive(Clone, Debug)]
pub struct CompositionData {
    pub text: String,
}

pub struct TerminalMetrics {
    pub char_height: i32,
    pub base_width: i32,
}

/// Wrapper around a Windows `HFONT` handle that is treated as `Send` and `Sync`.
///
/// この型は Win32 GDI のフォントハンドル `HFONT` をラップし、Rust の型システム上は
/// `Send` / `Sync` として扱えるようにするためのものです。これは **ハンドル値を別スレッドに
/// 移動・共有できる** ことを許可するだけであり、GDI オブジェクト自体のスレッド安全性を
/// 高めるものではありません。
///
/// # Safety
///
/// - `HFONT` を含む GDI オブジェクトは、一般に「作成・選択・描画・破棄」といった GDI 関数の
///   呼び出しを UI スレッド（メッセージループを持つスレッド）から行うことが前提です。
///   `SendHFONT` 自体はスレッド間で移動可能ですが、**GDI 関数の呼び出しは UI スレッドに
///   制限される** という前提を守る必要があります。
/// - `SendHFONT` は `HFONT` の所有権を表す単純なラッパーであり、`DeleteObject` による解放は
///   呼び出し側の責務です。`HFONT` を最後に使用したスレッド / コンテキストにおいて、
///   必ず `DeleteObject` を一度だけ呼び出してください。
/// - `Send` / `Sync` を付与していても、複数スレッドから同じ `HFONT` を同時に GDI API に渡すことは
///   想定していません。実際に GDI に対してフォントを選択したり描画に使用したりする操作は、
///   適切に同期された単一の UI スレッドで行ってください。
/// - この型を利用するコード（例: `TerminalRenderer::fonts` など）は、上記の制約を前提として設計されている
///   必要があります。もし将来的にスレッドモデルや GDI の利用方法を変更する場合は、
///   ここで述べた前提条件が依然として満たされているか再検証してください。
pub struct SendHFONT(pub HFONT);
unsafe impl Send for SendHFONT {}
unsafe impl Sync for SendHFONT {}

const STYLE_BOLD: u32 = 1 << 0;
const STYLE_ITALIC: u32 = 1 << 1;
const STYLE_UNDERLINE: u32 = 1 << 2;
const STYLE_STRIKEOUT: u32 = 1 << 3;

pub struct TerminalRenderer {
    fonts: HashMap<u32, SendHFONT>,
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
            fonts: HashMap::new(),
            metrics: None,
        }
    }

    pub fn clear_resources(&mut self) {
        for (_, send_h_font) in self.fonts.drain() {
            unsafe {
                let _ = DeleteObject(HGDIOBJ(send_h_font.0 .0));
            }
        }
        log::info!("TerminalRenderer: All cached font handles deleted");
    }

    pub fn get_metrics(&self) -> Option<&TerminalMetrics> {
        self.metrics.as_ref()
    }

    fn get_font_for_style(&mut self, hdc: HDC, style_mask: u32) -> HFONT {
        if let Some(font) = self.fonts.get(&style_mask) {
            return font.0;
        }

        unsafe {
            let mut lf = LOGFONTW {
                lfHeight: 16,
                lfWeight: if (style_mask & STYLE_BOLD) != 0 {
                    FW_BOLD.0 as i32
                } else {
                    FW_NORMAL.0 as i32
                },
                lfItalic: if (style_mask & STYLE_ITALIC) != 0 {
                    1
                } else {
                    0
                },
                lfUnderline: if (style_mask & STYLE_UNDERLINE) != 0 {
                    1
                } else {
                    0
                },
                lfStrikeOut: if (style_mask & STYLE_STRIKEOUT) != 0 {
                    1
                } else {
                    0
                },
                lfCharSet: FONT_CHARSET(DEFAULT_CHARSET.0),
                lfOutPrecision: FONT_OUTPUT_PRECISION(OUT_DEFAULT_PRECIS.0),
                lfClipPrecision: FONT_CLIP_PRECISION(CLIP_DEFAULT_PRECIS.0),
                lfQuality: FONT_QUALITY(DEFAULT_QUALITY.0),
                lfPitchAndFamily: FIXED_PITCH.0 | FF_MODERN.0,
                ..Default::default()
            };

            let face_name = w!("Consolas");
            let len = std::cmp::min(face_name.len(), lf.lfFaceName.len() - 1);
            for i in 0..len {
                lf.lfFaceName[i] = face_name.as_wide()[i];
            }
            lf.lfFaceName[len] = 0;

            let h_font = CreateFontIndirectW(&lf);
            if h_font.0.is_null() {
                log::error!(
                    "TerminalRenderer: Failed to create font for style mask {}",
                    style_mask
                );

                // すでにデフォルトスタイル(0)のフォントがキャッシュされていれば、それにフォールバックする
                if let Some(default_font) = self.fonts.get(&0) {
                    log::warn!(
                        "TerminalRenderer: Falling back to cached default font (style mask 0)"
                    );
                    return default_font.0;
                }

                // スタイル付きフォント生成に失敗した場合は、style_mask = 0 のフォント生成を試みる
                if style_mask != 0 {
                    log::warn!(
                        "TerminalRenderer: Retrying font creation with style mask 0 as fallback"
                    );
                    return self.get_font_for_style(hdc, 0);
                }

                // style_mask = 0 でも生成に失敗した場合は、最後の手段としてデフォルトハンドルを返す
                log::error!(
                    "TerminalRenderer: Failed to create even the default font (style mask 0)"
                );
                return HFONT::default();
            }

            if self.metrics.is_none() {
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
                let _ = SelectObject(hdc, old_font);
            }

            self.fonts.insert(style_mask, SendHFONT(h_font));
            h_font
        }
    }

    fn color_to_colorref(&self, color: TerminalColor, is_background: bool) -> COLORREF {
        match color {
            TerminalColor::Default => {
                if is_background {
                    COLORREF(0x00000000) // Black
                } else {
                    COLORREF(0x00FFFFFF) // White
                }
            }
            TerminalColor::Ansi(n) => self.ansi_to_colorref(n),
            TerminalColor::Xterm(n) => self.xterm_to_colorref(n),
            TerminalColor::Rgb(r, g, b) => {
                COLORREF(r as u32 | ((g as u32) << 8) | ((b as u32) << 16))
            }
        }
    }

    fn ansi_to_colorref(&self, n: u8) -> COLORREF {
        match n {
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
            _ => COLORREF(0x00FFFFFF),
        }
    }

    fn xterm_to_colorref(&self, n: u8) -> COLORREF {
        match n {
            0..=15 => self.ansi_to_colorref(n),
            16..=231 => {
                let idx = n - 16;
                let r_idx = idx / 36;
                let g_idx = (idx % 36) / 6;
                let b_idx = idx % 6;
                let r = if r_idx > 0 { r_idx * 40 + 55 } else { 0 };
                let g = if g_idx > 0 { g_idx * 40 + 55 } else { 0 };
                let b = if b_idx > 0 { b_idx * 40 + 55 } else { 0 };
                COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16))
            }
            232..=255 => {
                let val = (n - 232) * 10 + 8;
                COLORREF((val as u32) | ((val as u32) << 8) | ((val as u32) << 16))
            }
        }
    }

    pub fn render(
        &mut self,
        hdc: HDC,
        client_rect: &RECT,
        buffer: &TerminalBuffer,
        composition: Option<&CompositionData>,
    ) {
        let _ = self.get_font_for_style(hdc, 0);

        let metrics = match &self.metrics {
            Some(m) => m,
            None => return,
        };

        unsafe {
            let char_height = metrics.char_height;
            let base_width = metrics.base_width;

            let mut current_y = 0;
            let (cursor_x, cursor_y) = buffer.get_cursor_pos();
            let viewport_offset = buffer.viewport_offset;

            for visual_row in 0..buffer.height {
                let mut x_offset = 0;

                if let Some(line) = buffer.get_line_at_visual_row(visual_row) {
                    let mut cell_idx = 0;

                    while cell_idx < line.len() {
                        let start_attr = line[cell_idx].attribute;
                        let mut run_text = String::new();
                        let mut run_dx = Vec::new();

                        while cell_idx < line.len() && line[cell_idx].attribute == start_attr {
                            let cell = &line[cell_idx];
                            run_text.push(cell.c);
                            let w = TerminalBuffer::char_display_width(cell.c) as i32 * base_width;
                            run_dx.push(w);
                            run_dx.extend(std::iter::repeat_n(
                                0,
                                cell.c.len_utf16().saturating_sub(1),
                            ));
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

                        if !wide_run.is_empty() {
                            let mut style_mask = 0;
                            if start_attr.is_bold {
                                style_mask |= STYLE_BOLD;
                            }
                            if start_attr.is_italic {
                                style_mask |= STYLE_ITALIC;
                            }
                            if start_attr.is_underline {
                                style_mask |= STYLE_UNDERLINE;
                            }
                            if start_attr.is_strikethrough {
                                style_mask |= STYLE_STRIKEOUT;
                            }

                            let h_font = self.get_font_for_style(hdc, style_mask);
                            let old_font = SelectObject(hdc, HGDIOBJ(h_font.0));

                            let fg = if start_attr.is_inverse {
                                start_attr.bg
                            } else {
                                start_attr.fg
                            };
                            let bg = if start_attr.is_inverse {
                                start_attr.fg
                            } else {
                                start_attr.bg
                            };

                            let mut fg_colorref = self.color_to_colorref(fg, false);
                            let bg_colorref = self.color_to_colorref(bg, true);

                            // Dim属性が有効な場合は、COLORREFのRGB成分を用いて輝度を低減する
                            if start_attr.is_dim {
                                let raw = fg_colorref.0;
                                // COLORREFは通常0x00BBGGRR形式
                                let r = (raw & 0x000000FF) as u8 / 2;
                                let g = ((raw & 0x0000FF00) >> 8) as u8 / 2;
                                let b = ((raw & 0x00FF0000) >> 16) as u8 / 2;
                                let dim_raw = ((b as u32) << 16) | ((g as u32) << 8) | (r as u32);
                                fg_colorref = COLORREF(dim_raw);
                            }

                            SetTextColor(hdc, fg_colorref);
                            SetBkColor(hdc, bg_colorref);

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

                            let _ = SelectObject(hdc, old_font);
                        }

                        x_offset += run_pixel_width;
                    }
                }

                if x_offset < client_rect.right {
                    let fill_rect = RECT {
                        left: x_offset,
                        top: current_y,
                        right: client_rect.right,
                        bottom: current_y + char_height,
                    };
                    SetBkColor(hdc, COLORREF(0x00000000));
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

                if viewport_offset == 0 && visual_row == cursor_y {
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
            SetBkColor(hdc, COLORREF(0x00000000));
            SetTextColor(hdc, COLORREF(0x00FFFFFF));
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
                log::error!("TerminalRenderer: Failed to create pen for composition is_underline");
            }
        }
    }
}
