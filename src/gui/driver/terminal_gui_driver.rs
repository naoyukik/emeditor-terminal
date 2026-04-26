use crate::domain::model::terminal_buffer_entity::{
    CursorStyle, TerminalBufferEntity, TerminalColor,
};
use crate::gui::common::points_to_pixels_from_hdc;
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;
use windows::Win32::Foundation::{COLORREF, RECT, SIZE};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CLIP_DEFAULT_PRECIS, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontIndirectW,
    CreateSolidBrush, DEFAULT_CHARSET, DEFAULT_QUALITY, DeleteDC, DeleteObject, ETO_OPAQUE,
    ETO_OPTIONS, ExtTextOutW, FF_MODERN, FIXED_PITCH, FONT_CHARSET, FONT_CLIP_PRECISION,
    FONT_OUTPUT_PRECISION, FONT_QUALITY, FillRect, GetTextExtentPoint32W, GetTextMetricsW, HDC,
    HFONT, HGDIOBJ, InvertRect, LOGFONTW, OUT_DEFAULT_PRECIS, SRCCOPY, SelectObject, SetBkColor,
    SetTextColor, TEXTMETRICW,
};
use windows::core::PCWSTR;

pub struct TerminalGuiDriverContext {
    pub hdc: HDC,
    pub rect: RECT,
}

#[derive(Clone, Debug)]
pub struct CompositionInfo {
    pub text: String,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TerminalMetrics {
    pub char_height: i32,
    pub base_width: i32,
}

pub struct SendHFONT(pub HFONT);

// SAFETY:
// - SendHFONT は GDI フォントオブジェクト HFONT のラッパーであり、自身はハンドル値のみを保持する。
// - この型は「HFONT の所有権を表す値」をスレッド間で転送したり共有したりする目的でのみ用いる。
unsafe impl Send for SendHFONT {}
unsafe impl Sync for SendHFONT {}

const STYLE_BOLD: u32 = 1 << 0;
const STYLE_ITALIC: u32 = 1 << 1;
const STYLE_UNDERLINE: u32 = 1 << 2;
const STYLE_STRIKEOUT: u32 = 1 << 3;

const HGDI_ERROR_VALUE: isize = -1;

struct CreatedDcGuard(HDC);
impl Drop for CreatedDcGuard {
    fn drop(&mut self) {
        if !self.0.0.is_null() {
            // SAFETY: 生成された DC を確実に削除し、リソースリークを防ぐ。
            unsafe {
                let _ = DeleteDC(self.0);
            }
        }
    }
}

struct GdiObjectGuard(HGDIOBJ);
impl Drop for GdiObjectGuard {
    fn drop(&mut self) {
        if !self.0.0.is_null() {
            // SAFETY: 生成された GDI オブジェクトを確実に削除し、リソースリークを防ぐ。
            unsafe {
                let _ = DeleteObject(self.0);
            }
        }
    }
}

struct SelectedObjectGuard {
    hdc: HDC,
    prev_obj: HGDIOBJ,
}

impl SelectedObjectGuard {
    fn new(hdc: HDC, prev_obj: HGDIOBJ) -> Self {
        Self { hdc, prev_obj }
    }
}

impl Drop for SelectedObjectGuard {
    fn drop(&mut self) {
        if !self.prev_obj.0.is_null() && self.prev_obj.0 != HGDI_ERROR_VALUE as *mut _ {
            // SAFETY: デバイスコンテキストに元のオブジェクトを再選択し、状態を復元する。
            unsafe {
                let _ = SelectObject(self.hdc, self.prev_obj);
            }
        }
    }
}

struct RenderContext {
    x: i32,
    y: i32,
    char_height: i32,
    base_width: i32,
}

pub struct TerminalGuiDriver {
    fonts: HashMap<u32, SendHFONT>,
    pub(crate) metrics: Option<TerminalMetrics>,
}

impl Default for TerminalGuiDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalGuiDriver {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            metrics: None,
        }
    }

    fn rgb_to_colorref(rgb: &crate::domain::model::color_theme_value::RgbColor) -> COLORREF {
        COLORREF(rgb.r as u32 | ((rgb.g as u32) << 8) | ((rgb.b as u32) << 16))
    }

    pub fn clear_resources(&mut self) {
        for (_, send_h_font) in self.fonts.drain() {
            // SAFETY: キャッシュされていたフォントハンドルを破棄する。
            unsafe {
                let _ = DeleteObject(HGDIOBJ(send_h_font.0.0));
            }
        }
        self.metrics = None;
        log::info!("TerminalGuiDriver: All cached font handles and metrics cleared");
    }

    pub fn get_metrics(&self) -> Option<&TerminalMetrics> {
        self.metrics.as_ref()
    }

    pub fn update_metrics(
        &mut self,
        hdc: HDC,
        config: &crate::domain::model::terminal_config_value::TerminalConfig,
    ) -> bool {
        if hdc.0.is_null() {
            return false;
        }
        let h_font = self.get_font_for_style(hdc, 0, config);
        if h_font.0.is_null() {
            return false;
        }
        // SAFETY: 有効な HDC に対してフォントを選択し、テキストメトリクスを取得する。
        // SelectedObjectGuard により、元のフォントは確実に復元される。
        unsafe {
            let old_font = SelectObject(hdc, HGDIOBJ(h_font.0));
            let _font_guard = SelectedObjectGuard::new(hdc, old_font);
            let mut tm = TEXTMETRICW::default();
            if !GetTextMetricsW(hdc, &mut tm).as_bool() {
                return false;
            }
            let zero_utf16: &[u16] = &[0x0030];
            let mut size = SIZE::default();
            if !GetTextExtentPoint32W(hdc, zero_utf16, &mut size).as_bool() {
                return false;
            }

            if tm.tmHeight <= 0 || size.cx <= 0 {
                log::error!(
                    "TerminalGuiDriver: Invalid metrics obtained: height={}, width={}",
                    tm.tmHeight,
                    size.cx
                );
                return false;
            }

            let new_metrics = TerminalMetrics {
                char_height: tm.tmHeight,
                base_width: size.cx,
            };

            if self.metrics != Some(new_metrics) {
                self.metrics = Some(new_metrics);
                log::debug!(
                    "TerminalGuiDriver: Metrics updated: char_height={}, base_width={}",
                    tm.tmHeight,
                    size.cx
                );
            }
            true
        }
    }

    pub fn cell_to_pixel(&self, x: usize, y: usize) -> Option<(i32, i32)> {
        let metrics = self.metrics.as_ref()?;
        Some((
            x as i32 * metrics.base_width,
            y as i32 * metrics.char_height,
        ))
    }

    fn apply_dim_effect(color: COLORREF) -> COLORREF {
        let r = (color.0 & 0xFF) as u8;
        let g = ((color.0 >> 8) & 0xFF) as u8;
        let b = ((color.0 >> 16) & 0xFF) as u8;
        COLORREF(((r / 2) as u32) | (((g / 2) as u32) << 8) | (((b / 2) as u32) << 16))
    }

    pub(crate) fn get_font_for_style(
        &mut self,
        hdc: HDC,
        style_mask: u32,
        config: &crate::domain::model::terminal_config_value::TerminalConfig,
    ) -> HFONT {
        if let Some(font) = self.fonts.get(&style_mask) {
            return font.0;
        }

        // SAFETY: 指定された設定に基づき Win32 API で論理フォントを作成する。
        // 作成されたハンドルは self.fonts で管理され、clear_resources で破棄される。
        unsafe {
            let mut lf = LOGFONTW {
                lfHeight: points_to_pixels_from_hdc(hdc, config.font_size),
                lfWeight: if (style_mask & STYLE_BOLD) != 0 {
                    std::cmp::max(config.font_weight, 700)
                } else {
                    config.font_weight
                },
                lfItalic: if (style_mask & STYLE_ITALIC) != 0 || config.font_italic {
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

            let face_name_wide: Vec<u16> = config.font_face.encode_utf16().collect();
            let len = std::cmp::min(face_name_wide.len(), lf.lfFaceName.len() - 1);
            lf.lfFaceName[..len].copy_from_slice(&face_name_wide[..len]);
            lf.lfFaceName[len] = 0;

            let h_font = CreateFontIndirectW(&lf);
            if h_font.0.is_null() {
                log::error!(
                    "CreateFontIndirectW failed for style_mask={:#x}, face={}",
                    style_mask,
                    config.font_face
                );
                if style_mask != 0 {
                    return self.get_font_for_style(hdc, 0, config);
                }
                return HFONT::default();
            }

            self.fonts.insert(style_mask, SendHFONT(h_font));
            h_font
        }
    }

    fn color_to_colorref(
        &self,
        color: &TerminalColor,
        is_background: bool,
        theme: &crate::domain::model::color_theme_value::ColorTheme,
    ) -> COLORREF {
        match color {
            TerminalColor::Default => {
                let rgb = if is_background {
                    &theme.default_bg
                } else {
                    &theme.default_fg
                };
                Self::rgb_to_colorref(rgb)
            }
            TerminalColor::Ansi(n) => {
                let idx = if *n < 16 { *n as usize } else { 15 };
                Self::rgb_to_colorref(&theme.ansi_palette[idx])
            }
            TerminalColor::Xterm(n) => match *n {
                0..=15 => Self::rgb_to_colorref(&theme.ansi_palette[*n as usize]),
                16..=231 => {
                    let idx = *n - 16;
                    let r = if (idx / 36) > 0 {
                        (idx / 36) * 40 + 55
                    } else {
                        0
                    };
                    let g = if ((idx % 36) / 6) > 0 {
                        ((idx % 36) / 6) * 40 + 55
                    } else {
                        0
                    };
                    let b = if (idx % 6) > 0 {
                        (idx % 6) * 40 + 55
                    } else {
                        0
                    };
                    COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16))
                }
                232..=255 => {
                    let val = (*n - 232) * 10 + 8;
                    COLORREF((val as u32) | ((val as u32) << 8) | ((val as u32) << 16))
                }
            },
            TerminalColor::Rgb(r, g, b) => {
                COLORREF((*r as u32) | ((*g as u32) << 8) | ((*b as u32) << 16))
            }
        }
    }

    pub fn render(
        &mut self,
        hdc: HDC,
        client_rect: &RECT,
        buffer: &TerminalBufferEntity,
        composition: Option<&CompositionInfo>,
        theme: &crate::domain::model::color_theme_value::ColorTheme,
        config: &crate::domain::model::terminal_config_value::TerminalConfig,
    ) {
        let width = client_rect.right - client_rect.left;
        let height = client_rect.bottom - client_rect.top;
        if width <= 0 || height <= 0 {
            return;
        }

        // SAFETY: ダブルバッファリングのためのメモリ DC およびビットマップの生成。
        // CreatedDcGuard および GdiObjectGuard によりリソースは確実に解放される。
        unsafe {
            let h_mem_dc = CreateCompatibleDC(Some(hdc));
            if h_mem_dc.0.is_null() {
                return;
            }
            let _dc_guard = CreatedDcGuard(h_mem_dc);
            let h_bm = CreateCompatibleBitmap(hdc, width, height);
            if h_bm.0.is_null() {
                return;
            }
            let _bm_guard = GdiObjectGuard(HGDIOBJ(h_bm.0));
            let h_old_bm = SelectObject(h_mem_dc, HGDIOBJ(h_bm.0));
            let _bm_select_guard = SelectedObjectGuard::new(h_mem_dc, h_old_bm);

            self.render_internal(h_mem_dc, client_rect, buffer, composition, theme, config);

            let _ = BitBlt(
                hdc,
                client_rect.left,
                client_rect.top,
                width,
                height,
                Some(h_mem_dc),
                0,
                0,
                SRCCOPY,
            );
        }
    }

    fn render_internal(
        &mut self,
        hdc: HDC,
        client_rect: &RECT,
        buffer: &TerminalBufferEntity,
        composition: Option<&CompositionInfo>,
        theme: &crate::domain::model::color_theme_value::ColorTheme,
        config: &crate::domain::model::terminal_config_value::TerminalConfig,
    ) {
        let width = client_rect.right - client_rect.left;
        let height = client_rect.bottom - client_rect.top;
        let relative_rect = RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };

        let bg_colorref = self.color_to_colorref(&TerminalColor::Default, true, theme);
        // SAFETY: 背景塗りつぶし用のブラシ作成と描画。
        unsafe {
            let h_brush = CreateSolidBrush(bg_colorref);
            if !h_brush.0.is_null() {
                let _brush_guard = GdiObjectGuard(HGDIOBJ(h_brush.0));
                FillRect(hdc, &relative_rect, h_brush);
            }
        }

        if self.metrics.is_none() {
            self.update_metrics(hdc, config);
        }
        let metrics = match &self.metrics {
            Some(m) => m,
            None => return,
        };

        // SAFETY: 各セルの描画およびカーソル反転。
        unsafe {
            let char_height = metrics.char_height;
            let base_width = metrics.base_width;
            let mut current_y = 0;
            let (cursor_x, cursor_y) = if composition.is_some() {
                buffer.get_ime_anchor_pos()
            } else {
                buffer.get_cursor_pos()
            };
            let viewport_offset = buffer.get_viewport_offset();

            for visual_row in 0..buffer.get_height() {
                let mut x_offset = 0;
                if let Some(line) = buffer.get_line_at_visual_row(visual_row) {
                    let mut cell_idx = 0;
                    while cell_idx < buffer.get_width() {
                        let cell = match line.get(cell_idx) {
                            Some(c) => c,
                            None => break,
                        };
                        if cell.is_wide_continuation {
                            cell_idx += 1;
                            continue;
                        }

                        let start_attr = &cell.attribute;
                        let mut run_text = String::new();
                        let mut run_dx = Vec::new();

                        while cell_idx < buffer.get_width() {
                            let c = match line.get(cell_idx) {
                                Some(c) => c,
                                None => break,
                            };
                            if c.is_wide_continuation || &c.attribute != start_attr {
                                break;
                            }
                            run_text.push_str(&c.text);
                            let utf16_len = c.text.encode_utf16().count();
                            let w = c.text.width().clamp(1, 2) as i32 * base_width;
                            run_dx.push(w);
                            run_dx.extend(std::iter::repeat_n(0, utf16_len.saturating_sub(1)));
                            cell_idx += 1;
                        }

                        let wide_run: Vec<u16> = run_text.encode_utf16().collect();
                        let run_pixel_width: i32 = run_dx.iter().sum();

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

                            let h_font = self.get_font_for_style(hdc, style_mask, config);
                            let old_font = SelectObject(hdc, HGDIOBJ(h_font.0));
                            let _font_guard = SelectedObjectGuard::new(hdc, old_font);

                            let mut fg = self.color_to_colorref(&start_attr.fg, false, theme);
                            let mut bg = self.color_to_colorref(&start_attr.bg, true, theme);
                            if start_attr.is_inverse {
                                std::mem::swap(&mut fg, &mut bg);
                            }
                            if !start_attr.is_inverse
                                && start_attr.bg != TerminalColor::Default
                                && start_attr.fg == TerminalColor::Default
                            {
                                fg = self.color_to_colorref(&TerminalColor::Default, true, theme);
                            }
                            if start_attr.is_dim {
                                fg = Self::apply_dim_effect(fg);
                            }

                            SetTextColor(hdc, fg);
                            SetBkColor(hdc, bg);
                            let run_rect = RECT {
                                left: x_offset,
                                top: current_y,
                                right: x_offset + run_pixel_width,
                                bottom: current_y + char_height,
                            };
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
                        }
                        x_offset += run_pixel_width;
                    }
                }

                if viewport_offset == 0 && visual_row == cursor_y {
                    let safe_x = std::cmp::min(cursor_x, buffer.get_width().saturating_sub(1));
                    let px_x = safe_x as i32 * base_width;
                    if let Some(comp) = composition {
                        let ctx = RenderContext {
                            x: px_x,
                            y: current_y,
                            char_height,
                            base_width,
                        };
                        self.render_composition(hdc, &ctx, comp, theme, config);
                    } else if buffer.is_cursor_visible() {
                        let style = buffer.get_cursor_style();
                        let dw = if let Some(line) = buffer.get_line_at_visual_row(visual_row) {
                            line.get(safe_x)
                                .map(|cell| cell.text.width().clamp(1, 2))
                                .unwrap_or(1)
                        } else {
                            1
                        };
                        let rw = dw as i32 * base_width;
                        let rect = match style {
                            CursorStyle::BlinkingBlock | CursorStyle::SteadyBlock => RECT {
                                left: px_x,
                                top: current_y,
                                right: px_x + rw,
                                bottom: current_y + char_height,
                            },
                            CursorStyle::BlinkingUnderline | CursorStyle::SteadyUnderline => RECT {
                                left: px_x,
                                top: current_y + char_height - 2,
                                right: px_x + rw,
                                bottom: current_y + char_height,
                            },
                            CursorStyle::BlinkingBar | CursorStyle::SteadyBar => RECT {
                                left: px_x,
                                top: current_y,
                                right: px_x + 2,
                                bottom: current_y + char_height,
                            },
                        };
                        let _ = InvertRect(hdc, &rect);
                    }
                }
                current_y += char_height;
            }
        }
    }

    fn render_composition(
        &mut self,
        hdc: HDC,
        ctx: &RenderContext,
        comp: &CompositionInfo,
        theme: &crate::domain::model::color_theme_value::ColorTheme,
        config: &crate::domain::model::terminal_config_value::TerminalConfig,
    ) {
        let comp_wide: Vec<u16> = comp.text.encode_utf16().collect();
        let mut comp_dx = Vec::with_capacity(comp_wide.len());
        let mut pixel_width = 0;
        for c in comp.text.chars() {
            let text = c.to_string();
            let w = (text.width().clamp(1, 2) as i32) * ctx.base_width;
            comp_dx.push(w);
            comp_dx.extend(std::iter::repeat_n(
                0,
                text.encode_utf16().count().saturating_sub(1),
            ));
            pixel_width += w;
        }
        let comp_rect = RECT {
            left: ctx.x,
            top: ctx.y,
            right: ctx.x + pixel_width,
            bottom: ctx.y + ctx.char_height,
        };
        // SAFETY: IME 変換中の文字列描画。GDI リソースの選択と復元は SelectedObjectGuard が行う。
        unsafe {
            let h_font = self.get_font_for_style(hdc, 0, config);
            let old_font = SelectObject(hdc, HGDIOBJ(h_font.0));
            let _font_guard = SelectedObjectGuard::new(hdc, old_font);

            SetBkColor(hdc, Self::rgb_to_colorref(&theme.default_bg));
            SetTextColor(hdc, Self::rgb_to_colorref(&theme.default_fg));
            let _ = ExtTextOutW(
                hdc,
                ctx.x,
                ctx.y,
                ETO_OPTIONS(ETO_OPAQUE.0),
                Some(&comp_rect),
                PCWSTR(comp_wide.as_ptr()),
                comp_wide.len() as u32,
                Some(comp_dx.as_ptr()),
            );

            if pixel_width > 0 && ctx.char_height > 0 {
                let underline_rect = RECT {
                    left: ctx.x,
                    top: ctx.y + ctx.char_height - 1,
                    right: ctx.x + pixel_width,
                    bottom: ctx.y + ctx.char_height,
                };
                let h_brush = CreateSolidBrush(Self::rgb_to_colorref(&theme.default_fg));
                if !h_brush.0.is_null() {
                    let _brush_guard = GdiObjectGuard(HGDIOBJ(h_brush.0));
                    FillRect(hdc, &underline_rect, h_brush);
                }
            }
        }
    }
}
