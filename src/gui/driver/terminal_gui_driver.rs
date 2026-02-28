use crate::domain::model::terminal_buffer_entity::{
    CursorStyle, TerminalBufferEntity, TerminalColor,
};
use unicode_width::UnicodeWidthStr;
use std::collections::HashMap;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, RECT, SIZE};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontIndirectW, CreateSolidBrush,
    DeleteDC, DeleteObject, ExtTextOutW, FillRect, GetTextExtentPoint32W, GetTextMetricsW,
    InvertRect, SelectObject, SetBkColor, SetTextColor, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET,
    DEFAULT_QUALITY, ETO_OPAQUE, ETO_OPTIONS, FF_MODERN, FIXED_PITCH, FONT_CHARSET,
    FONT_CLIP_PRECISION, FONT_OUTPUT_PRECISION, FONT_QUALITY, FW_BOLD, FW_NORMAL, HDC, HFONT,
    HGDIOBJ, LOGFONTW, OUT_DEFAULT_PRECIS, SRCCOPY, TEXTMETRICW,
};

#[derive(Clone, Debug)]
pub struct CompositionInfo {
    pub text: String,
}

pub struct TerminalMetrics {
    pub char_height: i32,
    pub base_width: i32,
}

pub struct SendHFONT(pub HFONT);
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
        if !self.0 .0.is_null() {
            unsafe {
                let _ = DeleteDC(self.0);
            }
        }
    }
}

struct GdiObjectGuard(HGDIOBJ);
impl Drop for GdiObjectGuard {
    fn drop(&mut self) {
        if !self.0 .0.is_null() {
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
    metrics: Option<TerminalMetrics>,
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
            unsafe {
                let _ = DeleteObject(HGDIOBJ(send_h_font.0 .0));
            }
        }
        log::info!("TerminalGuiDriver: All cached font handles deleted");
    }

    pub fn get_metrics(&self) -> Option<&TerminalMetrics> {
        self.metrics.as_ref()
    }

    fn apply_dim_effect(color: COLORREF) -> COLORREF {
        let r = (color.0 & 0xFF) as u8;
        let g = ((color.0 >> 8) & 0xFF) as u8;
        let b = ((color.0 >> 16) & 0xFF) as u8;
        COLORREF(((r / 2) as u32) | (((g / 2) as u32) << 8) | (((b / 2) as u32) << 16))
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
                lfItalic: if (style_mask & STYLE_ITALIC) != 0 { 1 } else { 0 },   
                lfUnderline: if (style_mask & STYLE_UNDERLINE) != 0 { 1 } else { 0 },
                lfStrikeOut: if (style_mask & STYLE_STRIKEOUT) != 0 { 1 } else { 0 },
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
                log::error!("CreateFontIndirectW failed for style_mask={:#x}", style_mask);
                if style_mask != 0 { return self.get_font_for_style(hdc, 0); }    
                return HFONT::default();
            }

            if self.metrics.is_none() {
                let old_font = SelectObject(hdc, HGDIOBJ(h_font.0));
                let _font_guard = SelectedObjectGuard::new(hdc, old_font);
                let mut tm = TEXTMETRICW::default();
                let _ = GetTextMetricsW(hdc, &mut tm);
                let zero_utf16: &[u16] = &[0x0030];
                let mut size = SIZE::default();
                let _ = GetTextExtentPoint32W(hdc, zero_utf16, &mut size);
                self.metrics = Some(TerminalMetrics {
                    char_height: tm.tmHeight,
                    base_width: size.cx,
                });
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
                let rgb = if is_background { &theme.default_bg } else { &theme.default_fg };
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
                    let r = if (idx / 36) > 0 { (idx / 36) * 40 + 55 } else { 0 };
                    let g = if ((idx % 36) / 6) > 0 { ((idx % 36) / 6) * 40 + 55 } else { 0 };
                    let b = if (idx % 6) > 0 { (idx % 6) * 40 + 55 } else { 0 };  
                    COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16)) 
                }
                232..=255 => {
                    let val = (*n - 232) * 10 + 8;
                    COLORREF((val as u32) | ((val as u32) << 8) | ((val as u32) << 16))
                }
            },
            TerminalColor::Rgb(r, g, b) => COLORREF((*r as u32) | ((*g as u32) << 8) | ((*b as u32) << 16)),
        }
    }

    pub fn render(
        &mut self,
        hdc: HDC,
        client_rect: &RECT,
        buffer: &TerminalBufferEntity,
        composition: Option<&CompositionInfo>,
        theme: &crate::domain::model::color_theme_value::ColorTheme,
    ) {
        let width = client_rect.right - client_rect.left;
        let height = client_rect.bottom - client_rect.top;
        if width <= 0 || height <= 0 { return; }

        unsafe {
            let h_mem_dc = CreateCompatibleDC(hdc);
            if h_mem_dc.0.is_null() { return; }
            let _dc_guard = CreatedDcGuard(h_mem_dc);
            let h_bm = CreateCompatibleBitmap(hdc, width, height);
            if h_bm.0.is_null() { return; }
            let _bm_guard = GdiObjectGuard(HGDIOBJ(h_bm.0));
            let h_old_bm = SelectObject(h_mem_dc, HGDIOBJ(h_bm.0));
            let _bm_select_guard = SelectedObjectGuard::new(h_mem_dc, h_old_bm);  

            self.render_internal(h_mem_dc, client_rect, buffer, composition, theme);

            let _ = BitBlt(hdc, client_rect.left, client_rect.top, width, height, h_mem_dc, 0, 0, SRCCOPY);
        }
    }

    fn render_internal(
        &mut self,
        hdc: HDC,
        client_rect: &RECT,
        buffer: &TerminalBufferEntity,
        composition: Option<&CompositionInfo>,
        theme: &crate::domain::model::color_theme_value::ColorTheme,
    ) {
        let width = client_rect.right - client_rect.left;
        let height = client_rect.bottom - client_rect.top;
        let relative_rect = RECT { left: 0, top: 0, right: width, bottom: height };

        let bg_colorref = self.color_to_colorref(&TerminalColor::Default, true, theme);
        unsafe {
            let h_brush = CreateSolidBrush(bg_colorref);
            if !h_brush.0.is_null() {
                let _brush_guard = GdiObjectGuard(HGDIOBJ(h_brush.0));
                FillRect(hdc, &relative_rect, h_brush);
            }
        }

        let _ = self.get_font_for_style(hdc, 0);
        let metrics = match &self.metrics { Some(m) => m, None => return };       

        unsafe {
            let char_height = metrics.char_height;
            let base_width = metrics.base_width;
            let mut current_y = 0;
            let (cursor_x, cursor_y) = buffer.get_cursor_pos();
            let viewport_offset = buffer.get_viewport_offset();

            for visual_row in 0..buffer.get_height() {
                let mut x_offset = 0;
                if let Some(line) = buffer.get_line_at_visual_row(visual_row) {   
                    let mut cell_idx = 0;
                    while cell_idx < buffer.get_width() {
                        let cell = match line.get(cell_idx) { Some(c) => c, None => break };
                        if cell.is_wide_continuation { cell_idx += 1; continue; } 

                        // レビュー指摘修正: ループ内での clone 回避（参照を利用）
                        let start_attr = &cell.attribute;
                        let mut run_text = String::new();
                        let mut run_dx = Vec::new();

                        while cell_idx < buffer.get_width() {
                            let c = match line.get(cell_idx) { Some(c) => c, None => break };
                            if c.is_wide_continuation { cell_idx += 1; continue; }
                            if &c.attribute != start_attr { break; }

                            run_text.push_str(&c.text);
                            // セル内テキストの UTF-16 長を事前に計算（最適化）
                            let utf16_len = c.text.encode_utf16().count();
                            
                            // レビュー指摘修正: 物理表示幅を 1〜2 に制限
                            let display_width = std::cmp::min(std::cmp::max(c.text.width(), 1), 2);
                            let w = display_width as i32 * base_width;
                            run_dx.push(w);
                            run_dx.extend(std::iter::repeat(0).take(utf16_len.saturating_sub(1)));
                            cell_idx += 1;
                        }

                        let wide_run: Vec<u16> = run_text.encode_utf16().collect();
                        let run_pixel_width: i32 = run_dx.iter().sum();

                        if !wide_run.is_empty() {
                            let mut style_mask = 0;
                            if start_attr.is_bold { style_mask |= STYLE_BOLD; }   
                            if start_attr.is_italic { style_mask |= STYLE_ITALIC; }
                            if start_attr.is_underline { style_mask |= STYLE_UNDERLINE; }
                            if start_attr.is_strikethrough { style_mask |= STYLE_STRIKEOUT; }

                            let h_font = self.get_font_for_style(hdc, style_mask);
                            let old_font = SelectObject(hdc, HGDIOBJ(h_font.0));  
                            let _font_guard = SelectedObjectGuard::new(hdc, old_font);

                            let mut fg_colorref = self.color_to_colorref(&start_attr.fg, false, theme);
                            let mut bg_colorref = self.color_to_colorref(&start_attr.bg, true, theme);

                            if start_attr.is_inverse { std::mem::swap(&mut fg_colorref, &mut bg_colorref); }

                            if !start_attr.is_inverse && start_attr.bg != TerminalColor::Default && start_attr.fg == TerminalColor::Default {
                                fg_colorref = self.color_to_colorref(&TerminalColor::Default, true, theme);
                            }

                            if start_attr.is_dim { fg_colorref = Self::apply_dim_effect(fg_colorref); }

                            SetTextColor(hdc, fg_colorref);
                            SetBkColor(hdc, bg_colorref);

                            let run_rect = RECT { left: x_offset, top: current_y, right: x_offset + run_pixel_width, bottom: current_y + char_height };

                            // セキュリティリスク修正: 生テキストのログ出力を削除
                            
                            let _ = ExtTextOutW(hdc, x_offset, current_y, ETO_OPTIONS(ETO_OPAQUE.0), Some(&run_rect), PCWSTR(wide_run.as_ptr()), wide_run.len() as u32, Some(run_dx.as_ptr()));
                        }
                        x_offset += run_pixel_width;
                    }
                }

                if viewport_offset == 0 && visual_row == cursor_y {
                    let safe_cursor_x = std::cmp::min(cursor_x, buffer.get_width().saturating_sub(1));
                    let cursor_pixel_x = safe_cursor_x as i32 * base_width;       
                    if let Some(comp) = composition {
                        let ctx = RenderContext { x: cursor_pixel_x, y: current_y, char_height, base_width };
                        self.render_composition(hdc, &ctx, comp, theme);
                    } else if buffer.is_cursor_visible() {
                        let style = buffer.get_cursor_style();
                        let display_width = if let Some(line) = buffer.get_line_at_visual_row(visual_row) {
                            line.get(safe_cursor_x).map(|cell| {
                                // レビュー指摘修正: カーソル幅も 1〜2 に制限
                                let w = cell.text.width();
                                std::cmp::min(std::cmp::max(w, 1), 2)
                            }).unwrap_or(1)
                        } else { 1 };

                        let rect_width = display_width as i32 * base_width;       
                        let rect = match style {
                            CursorStyle::BlinkingBlock | CursorStyle::SteadyBlock => RECT { left: cursor_pixel_x, top: current_y, right: cursor_pixel_x + rect_width, bottom: current_y + char_height },
                            CursorStyle::BlinkingUnderline | CursorStyle::SteadyUnderline => RECT { left: cursor_pixel_x, top: current_y + char_height - 2, right: cursor_pixel_x + rect_width, bottom: current_y + char_height },
                            CursorStyle::BlinkingBar | CursorStyle::SteadyBar => RECT { left: cursor_pixel_x, top: current_y, right: cursor_pixel_x + 2, bottom: current_y + char_height },
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
        ctx: &RenderContext,
        comp: &CompositionInfo,
        theme: &crate::domain::model::color_theme_value::ColorTheme,
    ) {
        let comp_wide: Vec<u16> = comp.text.encode_utf16().collect();
        let mut comp_dx = Vec::with_capacity(comp_wide.len());
        let mut pixel_width = 0;
        for c in comp.text.chars() {
            let text = c.to_string();
            let w = text.width() as i32 * ctx.base_width;
            comp_dx.push(w);
            comp_dx.extend(std::iter::repeat(0).take(text.encode_utf16().count().saturating_sub(1)));
            pixel_width += w;
        }
        let comp_rect = RECT { left: ctx.x, top: ctx.y, right: ctx.x + pixel_width, bottom: ctx.y + ctx.char_height };
        unsafe {
            SetBkColor(hdc, Self::rgb_to_colorref(&theme.default_bg));
            SetTextColor(hdc, Self::rgb_to_colorref(&theme.default_fg));
            let _ = ExtTextOutW(hdc, ctx.x, ctx.y, ETO_OPTIONS(ETO_OPAQUE.0), Some(&comp_rect), PCWSTR(comp_wide.as_ptr()), comp_wide.len() as u32, Some(comp_dx.as_ptr()));

            if pixel_width > 0 && ctx.char_height > 0 {
                let underline_height: i32 = 1;
                let underline_top = ctx.y + ctx.char_height - underline_height;   
                let underline_bottom = ctx.y + ctx.char_height;
                let underline_rect = RECT { left: ctx.x, top: underline_top, right: ctx.x + pixel_width, bottom: underline_bottom };
                let underline_color = Self::rgb_to_colorref(&theme.default_fg);   
                let h_underline_brush = CreateSolidBrush(underline_color);        
                if !h_underline_brush.0.is_null() {
                    let _underline_brush_guard = GdiObjectGuard(HGDIOBJ(h_underline_brush.0));
                    let _ = FillRect(hdc, &underline_rect, h_underline_brush);
                }
            }
        }
    }
}
