use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{BOOL, HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, InvalidateRect, COLOR_WINDOW, HBRUSH, PAINTSTRUCT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateCaret, CreateWindowExW, DefWindowProcW, DestroyCaret, LoadCursorW, PostMessageW,
    RegisterClassW, SendMessageW, CS_HREDRAW, CS_VREDRAW, DLGC_WANTALLKEYS, IDC_ARROW, WM_CHAR,
    WM_DESTROY, WM_GETDLGCODE, WM_IME_COMPOSITION, WM_IME_ENDCOMPOSITION, WM_IME_SETCONTEXT,
    WM_IME_STARTCOMPOSITION, WM_KEYDOWN, WM_KEYUP, WM_KILLFOCUS, WM_LBUTTONDOWN, WM_MOUSEWHEEL,
    WM_PAINT, WM_SETFOCUS, WM_SIZE, WM_SYSCHAR, WM_SYSCOMMAND, WM_SYSKEYDOWN, WM_SYSKEYUP,
    WM_VSCROLL, WNDCLASSW, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_VISIBLE,
};
const ISC_SHOWUICOMPOSITIONWINDOW: u32 = 0x80000000;
use crate::application::TerminalService;
use crate::gui::renderer::{CompositionData, TerminalRenderer};
use crate::gui::scroll::{ScrollAction, ScrollManager};
use crate::infra::conpty::ConPTY;
use crate::infra::input::KeyboardHook;
use std::cell::RefCell;
use std::ffi::c_void;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use windows::Win32::Storage::FileSystem::ReadFile;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SetFocus, VK_ESCAPE, VK_F4, VK_MENU, VK_SPACE, VK_TAB,
};

// Constants from EmEditor SDK
const WM_USER: u32 = 0x0400;
const EE_FIRST: u32 = WM_USER + 0x400;
const EE_CUSTOM_BAR_OPEN: u32 = EE_FIRST + 73;

// Custom message for repaint from background thread
const WM_APP: u32 = 0x8000;
const WM_APP_REPAINT: u32 = WM_APP + 1;

// Scroll Bar Constants (Self-defined to avoid dependency issues)
const SB_VERT: i32 = 1;

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

// Custom Bar Positions
// const CUSTOM_BAR_LEFT: i32 = 0;
// const CUSTOM_BAR_TOP: i32 = 1;
// const CUSTOM_BAR_RIGHT: i32 = 2;
const CUSTOM_BAR_BOTTOM: i32 = 3;

#[repr(C)]
#[allow(non_snake_case)]
struct CUSTOM_BAR_INFO {
    cbSize: usize,
    hwndCustomBar: HWND,
    hwndClient: HWND,
    pszTitle: PCWSTR,
    iPos: i32,
}

static CLASS_REGISTERED: AtomicBool = AtomicBool::new(false);
const CLASS_NAME: PCWSTR = w!("EmEditorTerminalClass");

static TERMINAL_DATA: OnceLock<Arc<Mutex<TerminalData>>> = OnceLock::new();

// Keyboard hook wrapper
thread_local! {
    static KEYBOARD_HOOK_WRAPPER: RefCell<Option<KeyboardHook>> = const { RefCell::new(None) };
    static TERMINAL_HWND: RefCell<Option<HWND>> = const { RefCell::new(None) };
}

#[derive(Clone, Copy)]
/// Wrapper around a Windows `HWND` handle that is treated as `Send` and `Sync`.
///
/// On Windows, many operations on `HWND` (such as `PostMessageW`) are documented
/// as cross-thread safe, but some operations must only be performed on the thread
/// that created/owns the window (for example, most UI updates and message loops).
pub struct SendHWND(pub HWND);

/// SAFETY:
/// - The `HWND` handle value itself may be moved across threads.
/// - Callers must only perform operations from other threads that the Windows
///   API documents as thread-safe for `HWND` (e.g., `PostMessageW`).
/// - Thread-affine operations must still be invoked on the thread that owns the window.
unsafe impl Send for SendHWND {}

/// SAFETY:
/// - Sharing an `HWND` between threads does not in itself cause data races, as
///   long as all threads confine thread-affine operations to the owning thread
///   and only perform cross-thread-safe operations from other threads.
unsafe impl Sync for SendHWND {}

pub struct TerminalData {
    pub service: TerminalService,
    pub renderer: TerminalRenderer,
    pub window_handle: Option<SendHWND>,
    pub composition: Option<CompositionData>,
    pub scroll_manager: ScrollManager,
}

pub fn get_terminal_data() -> Arc<Mutex<TerminalData>> {
    TERMINAL_DATA
        .get_or_init(|| {
            Arc::new(Mutex::new(TerminalData {
                service: TerminalService::new(80, 25),
                renderer: TerminalRenderer::new(),
                window_handle: None,
                composition: None,
                scroll_manager: ScrollManager::new(),
            }))
        })
        .clone()
}


fn update_scroll_info(hwnd: HWND) {
    let data_arc = get_terminal_data();
    let mut data = data_arc.lock().unwrap();

    let history_count = data.service.get_history_count() as i32;
    let viewport_offset = data.service.get_viewport_offset() as i32;
    let height = data.service.buffer.height as i32;

    // Update ScrollManager state
    data.scroll_manager.min = 0;
    // The scrollable range is [0, history_count]
    // The ScrollManager uses nMax = history + page - 1 logic internally if needed, or we set it here.
    // Let's align with existing logic: nMax = history_count + page_size - 1
    // And nPos = history_count - viewport_offset
    
    let page_size = height;
    data.scroll_manager.max = history_count + page_size - 1;
    data.scroll_manager.page = page_size as u32;
    data.scroll_manager.pos = history_count - viewport_offset;

    let si = SCROLLINFO {
        cbSize: size_of::<SCROLLINFO>() as u32,
        fMask: SIF_ALL | SIF_DISABLENOSCROLL,
        nMin: data.scroll_manager.min,
        nMax: data.scroll_manager.max,
        nPage: data.scroll_manager.page,
        nPos: data.scroll_manager.pos,
        nTrackPos: 0,
    };

    unsafe {
        SetScrollInfo(hwnd, SB_VERT, &si, BOOL(1));
    }
}

// Helper to check if IME is composing
pub fn is_ime_composing(hwnd: HWND) -> bool {
    crate::gui::ime::is_composing(hwnd)
}

pub fn open_custom_bar(hwnd_editor: HWND) -> bool {
    unsafe {
        let h_instance = crate::get_instance_handle();

        // Check if already open
        let data_arc = get_terminal_data();
        {
            let data = data_arc.lock().unwrap();
            if let Some(h) = data.window_handle {
                let _ = SetFocus(h.0);
                return false;
            }
        }

        if !CLASS_REGISTERED.load(Ordering::Relaxed) {
            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: h_instance,
                lpszClassName: CLASS_NAME,
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as isize as *mut c_void),
                ..Default::default()
            };
            RegisterClassW(&wc);
            CLASS_REGISTERED.store(true, Ordering::Relaxed);
        }

        let hwnd_client_result = CreateWindowExW(
            Default::default(),
            CLASS_NAME,
            w!("Terminal"),
            WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
            0,
            0,
            0,
            0,
            hwnd_editor,
            None,
            h_instance,
            None,
        );

        match hwnd_client_result {
            Ok(hwnd_client) => {
                if hwnd_client.0.is_null() {
                    log::error!("Failed to create custom bar window: Handle is NULL");
                    return false;
                }

                // Store window handle immediately
                {
                    let mut data = data_arc.lock().unwrap();
                    // Double check if another window was created concurrently (unlikely in UI thread but safe)
                    if let Some(h) = data.window_handle {
                        // Another window exists, destroy this one and focus the existing one
                        let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd_client);
                        let _ = SetFocus(h.0);
                        return false;
                    }
                    data.window_handle = Some(SendHWND(hwnd_client));
                }

                let mut info = CUSTOM_BAR_INFO {
                    cbSize: size_of::<CUSTOM_BAR_INFO>(),
                    hwndCustomBar: HWND::default(),
                    hwndClient: hwnd_client,
                    pszTitle: w!("Terminal"),
                    iPos: CUSTOM_BAR_BOTTOM,
                };

                SendMessageW(
                    hwnd_editor,
                    EE_CUSTOM_BAR_OPEN,
                    WPARAM(0),
                    LPARAM(&mut info as *mut _ as isize),
                );

                // Calculate initial size
                let mut client_rect = windows::Win32::Foundation::RECT::default();
                let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(
                    hwnd_client,
                    &mut client_rect,
                );
                let width_px = client_rect.right - client_rect.left;
                let height_px = client_rect.bottom - client_rect.top;

                let (initial_cols, initial_rows) = if width_px > 0 && height_px > 0 {
                    let data = data_arc.lock().unwrap();
                    let (char_width, char_height) =
                        if let Some(metrics) = data.renderer.get_metrics() {
                            (metrics.base_width, metrics.char_height)
                        } else {
                            (8, 16) // Fallback
                        };
                    (
                        (width_px / char_width).max(1) as i16,
                        (height_px / char_height).max(1) as i16,
                    )
                } else {
                    (80, 25)
                };

                // Start ConPTY
                match ConPTY::new("pwsh.exe", initial_cols, initial_rows) {
                    Ok(conpty) => {
                        log::info!(
                            "ConPTY started successfully with size {}x{}",
                            initial_cols,
                            initial_rows
                        );

                        let data_arc = get_terminal_data();
                        let output_handle = conpty.get_output_handle();
                        let output_handle_raw = output_handle.0 .0 as usize;
                        {
                            let mut data = data_arc.lock().unwrap();
                            data.service.set_conpty(conpty);
                            // Sync buffer size with ConPTY
                            data.service
                                .resize(initial_cols as usize, initial_rows as usize);
                        }

                        let hwnd_client_ptr = hwnd_client.0 as usize;

                        thread::spawn(move || {
                            let mut buffer = [0u8; 1024];
                            let mut bytes_read = 0;
                            loop {
                                if let Err(e) = ReadFile(
                                    HANDLE(output_handle_raw as *mut _),
                                    Some(&mut buffer),
                                    Some(&mut bytes_read),
                                    None,
                                ) {
                                    log::error!("ReadFile failed: {}", e);
                                    break;
                                }
                                if bytes_read == 0 {
                                    log::info!("ReadFile returned 0 bytes (EOF)");
                                    break;
                                }

                                let raw_bytes = &buffer[..bytes_read as usize];
                                let hex_output: String =
                                    raw_bytes.iter().map(|b| format!("{:02X} ", b)).collect();
                                log::debug!(
                                    "ConPTY Raw Output ({} bytes): {}",
                                    bytes_read,
                                    hex_output
                                );
                                let output = String::from_utf8_lossy(raw_bytes);
                                log::debug!("ConPTY Output: {}", output);

                                {
                                    let mut data = data_arc.lock().unwrap();
                                    data.service.process_output(&output);
                                }

                                // Trigger repaint via PostMessage (thread-safe)
                                let _ = PostMessageW(
                                    HWND(hwnd_client_ptr as *mut _),
                                    WM_APP_REPAINT,
                                    WPARAM(0),
                                    LPARAM(0),
                                );
                            }
                            log::info!("ConPTY output thread finished");
                        });
                        true
                    }
                    Err(e) => {
                        log::error!("Failed to start ConPTY: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to create custom bar window: {}", e);
                false
            }
        }
    }
}

// vk_to_vt_sequence removed

/// 繧ｷ繧ｹ繝Β繧ｷ繝ｧ繝ｼ繝医き繝ヨlt+Tab 遲会ｼ峨〒縺ゅｋ縺九ｒ蛻､螳壹☆繧
pub fn is_system_shortcut(vk_code: u16, alt_pressed: bool) -> bool {
    alt_pressed
        && (vk_code == VK_F4.0
            || vk_code == VK_TAB.0
            || vk_code == VK_SPACE.0
            || vk_code == VK_ESCAPE.0)
}

#[allow(dead_code)]
pub fn send_input(text: &str) {
    let data_arc = get_terminal_data();
    let data = data_arc.lock().unwrap();
    let _ = data.service.send_input(text.as_bytes());
    // 改行を送る
    let _ = data.service.send_input(b"\r");
}

pub fn cleanup_terminal() {
    log::info!("cleanup_terminal: Starting cleanup");
    let data_arc = get_terminal_data();
    let mut data = data_arc.lock().unwrap();
    if let Some(_conpty) = data.service.take_conpty() {
        log::info!("ConPTY instance found, will be dropped and cleaned up");
        // Drop happens automatically
    } else {
        log::info!("No ConPTY instance to clean up");
    }
}

// Old functions removed: vk_to_vt_sequence, send_key_to_conpty, keyboard_hook_proc

fn install_keyboard_hook(hwnd: HWND) {
    KEYBOARD_HOOK_WRAPPER.with(|hook| {
        let mut hook_ref = hook.borrow_mut();
        if hook_ref.is_none() {
            let hook_instance = KeyboardHook::new(hwnd);
            hook_instance.install();
            *hook_ref = Some(hook_instance);
            log::info!("Keyboard hook wrapper installed via infra layer");
        }
    });
}

fn uninstall_keyboard_hook() {
    KEYBOARD_HOOK_WRAPPER.with(|hook| {
        let mut hook_ref = hook.borrow_mut();
        if let Some(h) = hook_ref.take() {
            h.uninstall(); // Explicit uninstall, though Drop would handle it
            log::info!("Keyboard hook wrapper uninstalled via infra layer");
        }
    });
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_VSCROLL => {
            let request = wparam.0 & 0xFFFF;
            let mut delta = 0isize;
            let mut absolute_pos: Option<usize> = None;

            let data_arc = get_terminal_data();

            match request as i32 {
                SB_LINEUP => delta = 1,
                SB_LINEDOWN => delta = -1,
                SB_PAGEUP => {
                    let data = data_arc.lock().unwrap();
                    delta = data.service.buffer.height as isize;
                }
                SB_PAGEDOWN => {
                    let data = data_arc.lock().unwrap();
                    delta = -(data.service.buffer.height as isize);
                }
                SB_THUMBTRACK | SB_THUMBPOSITION => {
                    let pos = (wparam.0 >> 16) & 0xFFFF;

                    // nPos = history_count - viewport_offset
                    // viewport_offset = history_count - nPos
                    let data = data_arc.lock().unwrap();
                    let history_count = data.service.get_history_count();
                    let target_pos = pos;

                    // Allow scroll beyond history count (active buffer)?
                    // No, our scroll bar maps 0..history_count.
                    if target_pos <= history_count {
                        absolute_pos = Some(history_count - target_pos);
                    } else {
                        absolute_pos = Some(0);
                    }
                }
                SB_TOP => {
                    let data = data_arc.lock().unwrap();
                    absolute_pos = Some(data.service.get_history_count());
                }
                SB_BOTTOM => absolute_pos = Some(0),
                _ => {}
            }

            {
                let mut data = data_arc.lock().unwrap();
                if let Some(pos) = absolute_pos {
                    data.service.scroll_to(pos);
                } else if delta != 0 {
                    data.service.scroll_lines(delta);
                }
            }

            update_scroll_info(hwnd);
            unsafe {
                let _ = InvalidateRect(hwnd, None, BOOL(0));
            }
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            let z_delta = (wparam.0 >> 16) as i16;
            // 3 lines per notch
            let lines = (z_delta as f32 / 120.0 * 3.0) as isize;

            if lines != 0 {
                let data_arc = get_terminal_data();
                let mut data = data_arc.lock().unwrap();
                data.service.scroll_lines(lines);
            }

            update_scroll_info(hwnd);
            unsafe {
                let _ = InvalidateRect(hwnd, None, BOOL(0));
            }
            LRESULT(0)
        }
        WM_PAINT => {
            unsafe {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);

                let data_arc = get_terminal_data();
                let mut data = data_arc.lock().unwrap();

                let mut client_rect = windows::Win32::Foundation::RECT::default();
                let _ =
                    windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut client_rect);

                let TerminalData {
                    ref service,
                    ref mut renderer,
                    ref composition,
                    ..
                } = *data;

                renderer.render(hdc, &client_rect, &service.buffer, composition.as_ref());

                let _ = EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            log::info!("WM_LBUTTONDOWN: Setting focus");
            unsafe {
                let _ = SetFocus(hwnd);
            }
            LRESULT(0)
        }
        WM_SETFOCUS => {
            log::info!("WM_SETFOCUS: Focus received, installing keyboard hook");
            TERMINAL_HWND.with(|h| {
                *h.borrow_mut() = Some(hwnd);
            });

            // Create a caret so SetCaretPos works
            let data_arc = get_terminal_data();
            let data = data_arc.lock().unwrap();
            let char_height = data
                .renderer
                .get_metrics()
                .map(|m| m.char_height)
                .unwrap_or(16);
            unsafe {
                let _ = CreateCaret(
                    hwnd,
                    windows::Win32::Graphics::Gdi::HBITMAP::default(),
                    2,
                    char_height,
                );
                // We don't call ShowCaret(hwnd) because we draw our own cursor overlay
            }

            install_keyboard_hook(hwnd);
            LRESULT(0)
        }
        WM_KILLFOCUS => {
            log::info!("WM_KILLFOCUS: Focus lost, uninstalling keyboard hook");
            unsafe {
                let _ = DestroyCaret();
            }
            uninstall_keyboard_hook();
            LRESULT(0)
        }
        WM_KEYDOWN => {
            let vk_code = wparam.0 as u16;
            log::debug!("WM_KEYDOWN received: 0x{:04X}", vk_code);

            // Note: Key processing is primarily handled by WM_APP_KEYINPUT (via Hook)
            // or WM_CHAR (for standard text).
            // We pass to DefWindowProcW to ensure standard behaviors (like generating WM_CHAR) are preserved.

            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_SYSKEYDOWN => {
            let vk_code = wparam.0 as u16;
            log::debug!("WM_SYSKEYDOWN received: 0x{:04X}", vk_code);

            // Exclusion list for system shortcuts that should be handled by Windows/EmEditor
            if is_system_shortcut(vk_code, true) {
                return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
            }

            if vk_code == VK_MENU.0 {
                return LRESULT(0);
            }

            // Note: Alt combinations are handled by WM_APP_KEYINPUT.
            // But if we want to suppress system menu beeps for handled Alt keys, we might need logic here.
            // For now, let's pass to DefWindowProcW, but WM_SYSCHAR will suppress the beep if needed.

            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_SYSKEYUP => {
            let vk_code = wparam.0 as u16;
            log::debug!("WM_SYSKEYUP received: 0x{:04X}", vk_code);

            if vk_code == VK_MENU.0 {
                log::debug!(
                    "Alt key (VK_MENU) released in WM_SYSKEYUP, suppressing DefWindowProcW"
                );
                return LRESULT(0);
            }

            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_KEYUP => {
            let vk_code = wparam.0 as u16;
            if vk_code == VK_MENU.0 {
                log::debug!("WM_KEYUP received for VK_MENU (Alt)");
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_SYSCHAR => {
            let char_code = wparam.0 as u16;
            log::debug!("WM_SYSCHAR received: 0x{:04X}", char_code);

            // Allow Alt+Space (System Menu) to pass through
            if char_code == 0x0020 {
                return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
            }

            // Alt+Key combinations often generate WM_SYSCHAR. Suppress it to prevent system beeps or menus.
            LRESULT(0)
        }
        WM_SYSCOMMAND => {
            let cmd = wparam.0 & 0xFFF0; // Low 4 bits are reserved
            log::debug!("WM_SYSCOMMAND received: 0x{:04X}", cmd);
            if cmd == 0xF100 {
                // SC_KEYMENU
                // When SC_KEYMENU is sent by a keystroke, lParam contains the char code in low word.
                let key_char = (lparam.0 & 0xFFFF) as u16;
                if key_char == 0x0020 {
                    log::debug!("SC_KEYMENU from Alt+Space: passing to DefWindowProcW");
                    return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
                }

                log::debug!("SC_KEYMENU received (Menu activation blocked)");
                return LRESULT(0);
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_GETDLGCODE => {
            log::debug!("WM_GETDLGCODE received");
            LRESULT(DLGC_WANTALLKEYS as isize)
        }
        WM_CHAR => {
            let char_code = wparam.0 as u16;
            log::debug!(
                "WM_CHAR received: 0x{:04X} ({})",
                char_code,
                String::from_utf16_lossy(&[char_code])
            );

            // Skip characters that were already handled by WM_KEYDOWN as special keys
            // This includes Enter (0x0D), Tab (0x09), Escape (0x1B), Backspace (0x08)
            if char_code == 0x0D || char_code == 0x09 || char_code == 0x1B || char_code == 0x08 {
                log::debug!(
                    "WM_CHAR: Skipping char 0x{:04X} (already handled by WM_KEYDOWN)",
                    char_code
                );
                return LRESULT(0);
            }

            {
                let data_arc = get_terminal_data();
                let mut data = data_arc.lock().unwrap();
                data.service.reset_viewport();
                let s = String::from_utf16_lossy(&[char_code]);
                let _ = data.service.send_input(s.as_bytes());
            }
            update_scroll_info(hwnd);
            unsafe {
                let _ = InvalidateRect(hwnd, None, BOOL(0));
            }

            LRESULT(0)
        }
        msg if msg == WM_APP_REPAINT => {
            // Handle repaint request from background thread
            update_scroll_info(hwnd);
            unsafe {
                let _ = InvalidateRect(hwnd, None, BOOL(0));
            }
            LRESULT(0)
        }
        WM_SIZE => {
            let width = (lparam.0 & 0xFFFF) as i32;
            let height = ((lparam.0 >> 16) & 0xFFFF) as i32;
            log::info!("WM_SIZE: width={}, height={}", width, height);

            {
                let data_arc = get_terminal_data();
                let mut data = data_arc.lock().unwrap();

                // Use cached metrics if available, otherwise approximation
                let (char_width, char_height) = if let Some(metrics) = data.renderer.get_metrics() {
                    (metrics.base_width, metrics.char_height)
                } else {
                    (8, 16) // Fallback approximation
                };

                // Convert pixel dimensions to console character dimensions
                let cols = (width / char_width).max(1) as i16;
                let rows = (height / char_height).max(1) as i16;

                log::info!("Resizing ConPTY to cols={}, rows={}", cols, rows);

                data.service.resize(cols as usize, rows as usize);
            }

            update_scroll_info(hwnd);
            LRESULT(0)
        }
        WM_IME_SETCONTEXT => {
            log::debug!(
                "WM_IME_SETCONTEXT: wparam={:?}, lparam={:?}",
                wparam,
                lparam
            );
            // We want to draw the composition string ourselves, so we clear the ISC_SHOWUICOMPOSITIONWINDOW flag
            let mut lparam = lparam;
            lparam.0 &= !(ISC_SHOWUICOMPOSITIONWINDOW as isize);
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_IME_STARTCOMPOSITION => {
            {
                let data_arc = get_terminal_data();
                let mut data = data_arc.lock().unwrap();
                crate::gui::ime::handle_start_composition(hwnd, &mut data.service);
            }
            // Lock must be released before calling update_scroll_info as it acquires the lock internally
            update_scroll_info(hwnd);

            LRESULT(0)
        }
        WM_IME_COMPOSITION => {
            let handled = {
                let data_arc = get_terminal_data();
                let mut data = data_arc.lock().unwrap();
                let data_inner = &mut *data;

                crate::gui::ime::handle_composition(
                    hwnd,
                    lparam,
                    &mut data_inner.service,
                    &data_inner.renderer,
                    &mut data_inner.composition,
                )
            };

            if !handled {
                unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
            } else {
                LRESULT(0)
            }
        }
        WM_IME_ENDCOMPOSITION => {
            {
                let data_arc = get_terminal_data();
                let mut data = data_arc.lock().unwrap();
                crate::gui::ime::handle_end_composition(hwnd, &mut data.composition);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            log::info!("WM_DESTROY: Cleaning up terminal resources");
            uninstall_keyboard_hook();

            // Clean up ConPTY and renderer resources
            let data_arc = get_terminal_data();
            let mut data = data_arc.lock().unwrap();

            data.window_handle = None;
            data.renderer.clear_resources();

            if let Some(_conpty) = data.service.take_conpty() {
                log::info!("ConPTY will be dropped and cleaned up");
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
