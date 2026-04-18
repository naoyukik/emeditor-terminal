use windows::Win32::Foundation::{HWND, RECT, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, InvalidateRect, UpdateWindow, PAINTSTRUCT};
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, DestroyWindow, GetClientRect, IsWindow, PostMessageW};
use crate::domain::model::window_id_value::WindowId;
use crate::gui::resolver::terminal_window_resolver::get_terminal_data;
use crate::gui::common::SendHWND;
use crate::infra::driver::conpty_io_driver::{ConptyIoDriver, SendHandle};
use crate::infra::driver::emeditor_io_driver::is_system_dark_mode;
use std::thread;

// Custom message for repaint from background thread
const WM_APP: u32 = 0x8000;
const WM_APP_REPAINT: u32 = WM_APP + 1;

/// Win32 ウィンドウ操作をカプセル化するドライバ
pub(crate) struct WindowGuiDriver;

impl WindowGuiDriver {
    /// 既存のウィンドウが存在し有効な場合、フォーカスを当てる。
    pub(crate) fn focus_existing_window(window_id: WindowId) -> bool {
        let hwnd = HWND(window_id.0 as _);
        unsafe {
            if IsWindow(Some(hwnd)).as_bool() {
                let _ = SetFocus(Some(hwnd));
                true
            } else {
                false
            }
        }
    }

    /// 指定されたウィンドウを安全に破棄する。
    pub(crate) fn destroy_window(window_id: WindowId) {
        let hwnd = HWND(window_id.0 as _);
        unsafe {
            if IsWindow(Some(hwnd)).as_bool() {
                let _ = DestroyWindow(hwnd);
            }
        }
    }

    /// 指定されたウィンドウの更新を強制する。
    pub(crate) fn update_window(window_id: WindowId) {
        let hwnd = HWND(window_id.0 as _);
        unsafe {
            if IsWindow(Some(hwnd)).as_bool() {
                let _ = UpdateWindow(hwnd);
            }
        }
    }

    /// ウィンドウの再描画を要求する。
    pub(crate) fn invalidate_rect(window_id: WindowId, erase: bool) {
        let hwnd = HWND(window_id.0 as _);
        unsafe {
            let _ = InvalidateRect(Some(hwnd), None, erase);
        }
    }

    /// 標準のウィンドウプロシージャを呼び出す。
    pub(crate) fn default_window_proc(window_id: WindowId, msg: u32, wparam: usize, lparam: isize) -> isize {
        let hwnd = HWND(window_id.0 as _);
        unsafe {
            DefWindowProcW(hwnd, msg, WPARAM(wparam), LPARAM(lparam)).0
        }
    }

    /// システムショートカットであるかを判定する。
    pub(crate) fn is_system_shortcut(vk_code: u16, alt_pressed: bool) -> bool {
        use windows::Win32::UI::Input::KeyboardAndMouse::{VK_ESCAPE, VK_F4, VK_SPACE, VK_TAB};
        alt_pressed
            && (vk_code == VK_F4.0
                || vk_code == VK_TAB.0
                || vk_code == VK_SPACE.0
                || vk_code == VK_ESCAPE.0)
    }

    /// 描画処理を実行するラッパー。
    pub(crate) fn perform_paint<F>(window_id: WindowId, f: F)
    where
        F: FnOnce(super::terminal_gui_driver::TerminalGuiDriverContext),
    {
        let hwnd = HWND(window_id.0 as _);
        unsafe {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            
            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);

            f(super::terminal_gui_driver::TerminalGuiDriverContext {
                hdc,
                rect,
            });

            let _ = EndPaint(hwnd, &ps);
        }
    }

    /// ターミナルのリサイズと初期化をハンドリングする。
    pub(crate) fn handle_resize(window_id: WindowId, lparam: isize) {
        let width = (lparam & 0xFFFF) as i32;
        let height = ((lparam >> 16) & 0xFFFF) as i32;

        if width <= 0 || height <= 0 { return; }

        let data_arc = get_terminal_data();
        let mut window_data = data_arc.lock().unwrap();

        if !window_data.is_conpty_started {
            drop(window_data);

            let hwnd = HWND(window_id.0 as _);
            let hwnd_editor = match unsafe { windows::Win32::UI::WindowsAndMessaging::GetParent(hwnd) } {
                Ok(h) => h,
                Err(_) => return,
            };
            let parent_id = WindowId(hwnd_editor.0 as isize);

            let config_repo = crate::infra::repository::emeditor_config_repository_impl::EmEditorConfigRepositoryImpl::new(parent_id);
            let config = crate::domain::repository::configuration_repository::ConfigurationRepository::load(&config_repo);

            let mut window_data = data_arc.lock().unwrap();
            // Update metrics using hdc
            unsafe {
                let hdc = windows::Win32::Graphics::Gdi::GetDC(Some(hwnd));
                if !hdc.is_invalid() {
                    window_data.renderer.update_metrics(hdc, &config);
                    let _ = windows::Win32::Graphics::Gdi::ReleaseDC(Some(hwnd), hdc);
                }
            }

            let (char_width, char_height) = if let Some(metrics) = window_data.renderer.get_metrics() {
                (metrics.base_width, metrics.char_height)
            } else {
                (8, 16)
            };

            let cols = (width / char_width).max(1) as i16;
            let rows = (height / char_height).max(1) as i16;

            drop(window_data);
            if crate::gui::window::ensure_conpty_started(hwnd, hwnd_editor, cols, rows) {
                log::info!("ConPTY started successfully");
            }
        } else {
            let (char_width, char_height) = if let Some(metrics) = window_data.renderer.get_metrics() {
                (metrics.base_width, metrics.char_height)
            } else {
                (8, 16)
            };
            let cols = (width / char_width).max(1) as i16;
            let rows = (height / char_height).max(1) as i16;
            window_data.service.resize(cols as usize, rows as usize);
        }
    }
}

pub(crate) fn cleanup_terminal() {
    let data_arc = get_terminal_data();
    let mut window_data = data_arc.lock().unwrap();
    
    use crate::infra::repository::conpty_repository_impl::DummyOutputRepository;
    use crate::infra::repository::emeditor_config_repository_impl::EmEditorConfigRepositoryImpl;
    
    let output_repo = Box::new(DummyOutputRepository);
    let config_repo = Box::new(EmEditorConfigRepositoryImpl::new(WindowId(0)));
    let is_dark = is_system_dark_mode();
    let service = crate::application::TerminalWorkflow::new(80, 25, output_repo, config_repo, is_dark);
    
    window_data.reset_service(service);
}
