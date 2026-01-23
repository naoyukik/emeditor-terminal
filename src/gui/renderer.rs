use crate::domain::terminal::TerminalBuffer;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{RECT, SIZE};
use windows::Win32::Graphics::Gdi::{
    CreateFontW, DeleteObject, ExtTextOutW, GetTextExtentPoint32W, GetTextMetricsW, InvertRect,
    SelectObject, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_QUALITY, ETO_OPAQUE, ETO_OPTIONS,
    FF_MODERN, FIXED_PITCH, FW_NORMAL, HDC, HFONT, HGDIOBJ, OUT_DEFAULT_PRECIS, TEXTMETRICW,
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
/// # 安全性 (Safety)
///
/// `HFONT` 自体は GDI オブジェクトを指す単なるハンドルであり、その値を
/// スレッド間でコピー／ムーブすること自体は Windows の設計上許可されています。
/// そのため、「ハンドル値を他スレッドに渡す」という点だけを見れば
/// `Send` / `Sync` を実装してもただちに未定義動作にはなりません。
///
/// ただし、**GDI 操作にはスレッドアフィニティがある** ことに注意してください。
///
/// - `SelectObject` や `ExtTextOutW`、`GetTextMetricsW` などの GDI 関数は
///   特定の `HDC`（デバイスコンテキスト）に対して動作し、その `HDC` は通常
///   UI スレッド（メッセージループを持つスレッド）に紐づいています。
/// - これらの GDI 関数はスレッドセーフではなく、UI スレッド以外から
///   呼び出すと描画の破綻や未定義動作を引き起こす可能性があります。
///
/// 本ラッパー型はあくまで「`HFONT` ハンドル値をスレッド間で保持・共有する」こと
/// のみを許可するものであり、**GDI 関数の呼び出しは必ず UI スレッドで行う**
/// という前提に依存しています。
///
/// # ライフタイムと DeleteObject について
///
/// `SendHFONT` は `HFONT` のライフタイムや所有権を管理しません。
/// フォントの生成 (`CreateFontW`) および破棄 (`DeleteObject`) は別途、
/// UI スレッド側で一元管理されている前提です。
///
/// - 他スレッドがまだ `SendHFONT` を保持している可能性がある間は
///   対応する `HFONT` に対して `DeleteObject` を呼び出してはいけません。
/// - `DeleteObject` が呼ばれた後に、同じ `HFONT` を用いて `SelectObject` などの
///   GDI 操作を行うことは未定義動作になります。
///
/// したがって、呼び出し側は以下を保証する必要があります:
///
/// 1. GDI 関数の呼び出し (`SelectObject`, `ExtTextOutW`, など) は UI スレッドからのみ行う。
/// 2. `HFONT` の実際のライフタイム管理は UI スレッド側で行い、
///    他スレッドがそのハンドルを保持している間は `DeleteObject` しない。
///
/// この型自体は上記制約を静的には強制しませんが、`Send` / `Sync` の `unsafe impl` は
/// これらの不変条件が守られていることを前提としています。
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

            let mut current_y = 0;
            let (cursor_x, cursor_y) = buffer.get_cursor_pos();

            for (idx, line) in buffer.get_lines().iter().enumerate() {
                let wide_line: Vec<u16> = line.encode_utf16().collect();
                let mut dx: Vec<i32> = Vec::with_capacity(wide_line.len());
                for c in line.chars() {
                    let width = TerminalBuffer::char_display_width(c) as i32 * base_width;
                    dx.push(width);
                    for _ in 1..c.len_utf16() {
                        dx.push(0);
                    }
                }

                let line_pixel_width: i32 = dx.iter().sum();
                let bg_rect = RECT {
                    left: 0,
                    top: current_y,
                    right: std::cmp::max(line_pixel_width, client_rect.right),
                    bottom: current_y + char_height,
                };

                let _ = ExtTextOutW(
                    hdc,
                    0,
                    current_y,
                    ETO_OPTIONS(ETO_OPAQUE.0),
                    Some(&bg_rect),
                    PCWSTR(wide_line.as_ptr()),
                    wide_line.len() as u32,
                    Some(dx.as_ptr()),
                );

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
            for _ in 1..c.len_utf16() {
                comp_dx.push(0);
            }
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
                windows::Win32::Foundation::COLORREF(0x00FF0000),
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
