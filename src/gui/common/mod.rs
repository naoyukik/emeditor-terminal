use windows::Win32::Foundation::HWND;

/// Win32 HWND をラップし、スレッドセーフに送受信可能にするための構造体。
///
/// この型は Win32 ハンドルそのものを保持する。
/// 低レイヤー（Driver 等）での API 呼び出しにおいて利便性のために使用される。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SendHWND(pub HWND);

// SAFETY:
// - HWND は OS によって管理されるウィンドウハンドルであり、このラッパー型は
//   そのハンドル値を他スレッドへ送る目的のためだけに使用する。
// - 他スレッドからこの HWND に対して行ってよい操作は、PostMessageW など
//   Win32 がスレッドセーフであると明示している API の呼び出しに限定する。
// - SendHWND を介してウィンドウプロシージャを直接呼び出したり、非スレッドセーフな
//   Win32 API（例: 一部の GDI 関数やスレッドアフィニティを前提とした API）を呼び出さないこと。
// - 上記の前提を満たす限り、SendHWND / &SendHWND を複数スレッド間で送受信・共有しても、
//   Rust の意味でのデータ競合や未定義動作は発生しないものとみなせる。
unsafe impl Send for SendHWND {}
unsafe impl Sync for SendHWND {}

/// ピクセル単位の高さからポイントサイズへ変換する
pub(crate) unsafe fn pixels_to_points(
    hwnd: windows::Win32::Foundation::HWND,
    lf_height: i32,
) -> i32 {
    use windows::Win32::Graphics::Gdi::{GetDC, ReleaseDC};
    let hdc = GetDC(Some(hwnd));
    if hdc.is_invalid() {
        return 10;
    }
    let pts = pixels_to_points_from_hdc(hdc, lf_height);
    ReleaseDC(Some(hwnd), hdc);
    pts
}

/// HDC からピクセル単位の高さからポイントサイズへ変換する
pub(crate) unsafe fn pixels_to_points_from_hdc(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    lf_height: i32,
) -> i32 {
    use windows::Win32::Graphics::Gdi::{GetDeviceCaps, LOGPIXELSY};
    let dpi_y = GetDeviceCaps(Some(hdc), LOGPIXELSY);
    if dpi_y == 0 {
        return 10;
    }
    (lf_height.abs() * 72 + dpi_y / 2) / dpi_y
}

/// ポイントサイズからピクセル単位の高さへ変換する (LOGFONT用)
pub(crate) unsafe fn points_to_pixels(hwnd: windows::Win32::Foundation::HWND, points: i32) -> i32 {
    use windows::Win32::Graphics::Gdi::{GetDC, ReleaseDC};
    let hdc = GetDC(Some(hwnd));
    if hdc.is_invalid() {
        return -13;
    }
    let px = points_to_pixels_from_hdc(hdc, points);
    ReleaseDC(Some(hwnd), hdc);
    px
}

/// HDC からポイントサイズからピクセル単位の高さへ変換する (LOGFONT用)
pub(crate) unsafe fn points_to_pixels_from_hdc(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    points: i32,
) -> i32 {
    use windows::Win32::Graphics::Gdi::{GetDeviceCaps, LOGPIXELSY};
    let dpi_y = GetDeviceCaps(Some(hdc), LOGPIXELSY);
    if dpi_y == 0 {
        return -13;
    }
    -((points * dpi_y + 36) / 72)
}
