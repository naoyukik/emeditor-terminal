use crate::domain::model::window_id_value::WindowId;
use crate::gui::driver::window_gui_driver::WindowGuiDriver;
use crate::gui::resolver::terminal_window_resolver::get_terminal_data;
use crate::infra::driver::conpty_io_driver::SendHandle;
use crate::infra::driver::emeditor_io_driver::{
    CUSTOM_BAR_BOTTOM, CUSTOM_BAR_INFO, EE_CUSTOM_BAR_OPEN,
};
use crate::infra::driver::emeditor_io_driver::{MB_ICONERROR, MB_OK, MessageBoxW, SendMessageW};
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::HBRUSH;
use windows::Win32::Storage::FileSystem::ReadFile;
use windows::Win32::UI::WindowsAndMessaging::{
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, PostMessageW,
    RegisterClassW, WINDOW_EX_STYLE, WM_APP, WM_CHAR, WM_DESTROY, WM_ERASEBKGND, WM_GETDLGCODE,
    WM_IME_COMPOSITION, WM_IME_ENDCOMPOSITION, WM_IME_SETCONTEXT, WM_IME_STARTCOMPOSITION,
    WM_KEYDOWN, WM_KEYUP, WM_KILLFOCUS, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_PAINT, WM_RBUTTONDOWN, WM_RBUTTONUP,
    WM_SETFOCUS, WM_SIZE, WM_SYSCHAR, WM_SYSCOMMAND, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_VSCROLL,
    WNDCLASSW, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_VISIBLE,
};
use windows::core::{PCWSTR, w};

use crate::gui::resolver::window_message_resolver as handlers;

/// 描画更新を通知するメッセージ
const WM_APP_REPAINT: u32 = WM_APP + 1;

static CLASS_REGISTERED: AtomicBool = AtomicBool::new(false);
const CLASS_NAME: PCWSTR = w!("EmEditorTerminalClass");

/// IME が変換中であるかを確認する
pub fn is_ime_composing(hwnd: HWND) -> bool {
    crate::gui::driver::ime_gui_driver::is_composing(WindowId(hwnd.0 as isize))
}

pub fn ensure_conpty_started(hwnd_client: HWND, hwnd_editor: HWND, cols: i16, rows: i16) -> bool {
    let data_arc = get_terminal_data();
    {
        let window_data = data_arc.lock().unwrap();
        if window_data.is_conpty_started {
            return true;
        }
    }

    let parent_id = WindowId(hwnd_editor.0 as isize);
    let config_repo = crate::infra::repository::emeditor_config_repository_impl::EmEditorConfigRepositoryImpl::new(
        parent_id,
    );
    let config = crate::domain::repository::configuration_repository::ConfigurationRepository::load(
        &config_repo,
    );

    let shell_path = config.shell_path.clone();
    log::info!("Starting ConPTY with shell: {}", shell_path);

    match crate::infra::driver::conpty_io_driver::ConptyIoDriver::new(&shell_path, None, cols, rows)
    {
        Ok(conpty) => {
            let output_handle: SendHandle = conpty.get_output_handle();

            {
                let mut window_data = data_arc.lock().unwrap();
                let output_repo = Box::new(
                    crate::infra::repository::conpty_repository_impl::ConptyRepositoryImpl::new(
                        conpty,
                    ),
                );

                let is_dark = crate::infra::driver::emeditor_io_driver::is_system_dark_mode();
                let translator = Box::new(
                    crate::domain::service::vt_sequence_translator_domain_service::VtSequenceTranslatorDomainService::new(),
                );

                window_data.service = crate::application::TerminalWorkflow::new(
                    cols as usize,
                    rows as usize,
                    output_repo,
                    Box::new(
                        crate::infra::repository::emeditor_config_repository_impl::EmEditorConfigRepositoryImpl::new(
                            parent_id,
                        ),
                    ),
                    translator,
                    is_dark,
                );
                window_data.is_conpty_started = true;
            }

            let send_hwnd = crate::gui::common::SendHWND(hwnd_client);

            thread::spawn(move || {
                let output_handle = output_handle;
                let send_hwnd = send_hwnd;
                let mut buffer = [0u8; 1024];
                let mut bytes_read = 0;
                loop {
                    // SAFETY: 有効なパイプハンドルに対して同期読み取りを行う。
                    // 読み取り結果は bytes_read に格納される。
                    let read_result = unsafe {
                        ReadFile(
                            output_handle.0,
                            Some(&mut buffer),
                            Some(&mut bytes_read),
                            None,
                        )
                    };

                    if read_result.is_err() {
                        break;
                    }
                    if bytes_read == 0 {
                        break;
                    }

                    let raw_bytes = &buffer[..bytes_read as usize];
                    {
                        let data = get_terminal_data();
                        let mut window_data = data.lock().unwrap();
                        window_data.service.process_output(raw_bytes);
                    }

                    // SAFETY: 有効なウィンドウハンドルに対して描画更新を通知する。
                    // PostMessageW はスレッドセーフである。
                    unsafe {
                        let _ =
                            PostMessageW(Some(send_hwnd.0), WM_APP_REPAINT, WPARAM(0), LPARAM(0));
                    }
                }
            });
            true
        }
        Err(e) => {
            log::error!("Failed to start ConPTY: {}", e);
            false
        }
    }
}

pub fn open_custom_bar(hwnd_editor: HWND) -> bool {
    // SAFETY: ウィンドウクラスの登録、ウィンドウの作成、およびメッセージ送信は
    // Win32 API の標準的な手順に従っており、有効なハンドルとリソースを使用する。
    unsafe {
        let h_instance = crate::get_instance_handle();

        let data_arc = get_terminal_data();
        let existing_hwnd = {
            let window_data = data_arc.lock().unwrap();
            window_data.window_handle.map(|h| h.0)
        };

        if let Some(hwnd) = existing_hwnd {
            if WindowGuiDriver::focus_existing_window(WindowId(hwnd.0 as isize)) {
                return false;
            } else {
                WindowGuiDriver::destroy_window(WindowId(hwnd.0 as isize));
                let mut window_data = data_arc.lock().unwrap();
                window_data.window_handle = None;
            }
        }

        if !CLASS_REGISTERED.load(Ordering::SeqCst) {
            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: h_instance,
                lpszClassName: CLASS_NAME,
                hbrBackground: HBRUSH(std::ptr::null_mut()),
                ..Default::default()
            };

            if RegisterClassW(&wc) == 0 {
                let err = windows::Win32::Foundation::GetLastError();
                log::error!("Failed to register window class: {:?}", err);
                return false;
            }
            CLASS_REGISTERED.store(true, Ordering::SeqCst);
        }

        // WM_SIZE での初期化に備え、CreateWindowExW 呼び出し前に親ハンドルを保存する
        {
            let mut window_data = data_arc.lock().unwrap();
            window_data.editor_handle = Some(crate::gui::common::SendHWND(hwnd_editor));
        }

        let hwnd_client_result = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            CLASS_NAME,
            w!("Terminal"),
            WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            Some(hwnd_editor),
            None,
            Some(h_instance),
            None,
        );

        match hwnd_client_result {
            Ok(hwnd_client) => {
                let mut window_data = data_arc.lock().unwrap();
                window_data.window_handle = Some(crate::gui::common::SendHWND(hwnd_client));
                drop(window_data);

                let mut info = CUSTOM_BAR_INFO {
                    cbSize: size_of::<CUSTOM_BAR_INFO>(),
                    hwndCustomBar: HWND::default(), // EmEditor 側でセットされる
                    hwndClient: hwnd_client,
                    pszTitle: w!("Terminal"),
                    iPos: CUSTOM_BAR_BOTTOM,
                };

                let _ = SendMessageW(
                    hwnd_editor,
                    EE_CUSTOM_BAR_OPEN,
                    Some(WPARAM(0)),
                    Some(LPARAM(&mut info as *mut _ as isize)),
                );

                WindowGuiDriver::focus_existing_window(WindowId(hwnd_client.0 as isize));
                true
            }
            Err(e) => {
                MessageBoxW(
                    Some(hwnd_editor),
                    w!("Failed to create terminal window."),
                    w!("Terminal Error"),
                    MB_ICONERROR | MB_OK,
                );
                log::error!("CreateWindowExW failed: {:?}", e);
                false
            }
        }
    }
}

pub fn cleanup_terminal() {
    log::info!("cleanup_terminal: Starting cleanup");
    let data_arc = get_terminal_data();
    let mut window_data = data_arc.lock().unwrap();

    // Workflow に新しいダミーサービスを注入してリセット
    use crate::domain::service::vt_sequence_translator_domain_service::VtSequenceTranslatorDomainService;
    use crate::infra::repository::conpty_repository_impl::DummyOutputRepository;
    use crate::infra::repository::emeditor_config_repository_impl::EmEditorConfigRepositoryImpl;

    let output_repo = Box::new(DummyOutputRepository);
    let config_repo = Box::new(EmEditorConfigRepositoryImpl::new(WindowId(0)));
    let translator = Box::new(VtSequenceTranslatorDomainService::new());
    let is_dark = crate::infra::driver::emeditor_io_driver::is_system_dark_mode();
    let service = crate::application::TerminalWorkflow::new(
        80,
        25,
        output_repo,
        config_repo,
        translator,
        is_dark,
    );

    window_data.reset_service(service);
}

pub extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let window_id = WindowId(hwnd.0 as isize);
    match msg {
        WM_VSCROLL => LRESULT(handlers::on_vscroll(window_id, wparam.0, lparam.0)),
        WM_MOUSEWHEEL => LRESULT(handlers::on_mousewheel(window_id, wparam.0, lparam.0)),
        WM_MOUSEHWHEEL => LRESULT(handlers::on_mousehwheel(window_id, wparam.0, lparam.0)),
        WM_PAINT => LRESULT(handlers::on_paint(window_id)),
        WM_LBUTTONDOWN => LRESULT(handlers::on_lbuttondown(window_id, wparam.0, lparam.0)),
        WM_LBUTTONUP => LRESULT(handlers::on_lbuttonup(window_id, wparam.0, lparam.0)),
        WM_RBUTTONDOWN => LRESULT(handlers::on_rbuttondown(window_id, wparam.0, lparam.0)),
        WM_RBUTTONUP => LRESULT(handlers::on_rbuttonup(window_id, wparam.0, lparam.0)),
        WM_MBUTTONDOWN => LRESULT(handlers::on_mbuttondown(window_id, wparam.0, lparam.0)),
        WM_MBUTTONUP => LRESULT(handlers::on_mbuttonup(window_id, wparam.0, lparam.0)),
        WM_MOUSEMOVE => LRESULT(handlers::on_mousemove(window_id, wparam.0, lparam.0)),
        WM_SETFOCUS => LRESULT(handlers::on_set_focus(window_id)),
        WM_KILLFOCUS => LRESULT(handlers::on_kill_focus()),
        WM_KEYDOWN => LRESULT(handlers::on_keydown(window_id, msg, wparam.0, lparam.0)),
        WM_SYSKEYDOWN => LRESULT(handlers::on_syskeydown(window_id, msg, wparam.0, lparam.0)),
        WM_SYSKEYUP => LRESULT(handlers::on_syskeyup(window_id, msg, wparam.0, lparam.0)),
        WM_KEYUP => LRESULT(handlers::on_keyup(window_id, msg, wparam.0, lparam.0)),
        WM_SYSCHAR => LRESULT(handlers::on_syschar(window_id, msg, wparam.0, lparam.0)),
        WM_SYSCOMMAND => LRESULT(handlers::on_syscommand(window_id, msg, wparam.0, lparam.0)),
        WM_GETDLGCODE => LRESULT(handlers::on_get_dlg_code()),
        WM_CHAR => LRESULT(handlers::on_char(window_id, wparam.0)),
        msg if msg == WM_APP_REPAINT => LRESULT(handlers::on_app_repaint(window_id)),
        WM_SIZE => LRESULT(handlers::on_size(window_id, lparam.0)),
        WM_IME_SETCONTEXT => LRESULT(handlers::on_ime_set_context(
            window_id, msg, wparam.0, lparam.0,
        )),
        WM_IME_STARTCOMPOSITION => LRESULT(handlers::on_ime_start_composition(window_id)),
        WM_IME_COMPOSITION => LRESULT(handlers::on_ime_composition(
            window_id, msg, wparam.0, lparam.0,
        )),
        WM_IME_ENDCOMPOSITION => LRESULT(handlers::on_ime_end_composition(window_id)),
        WM_ERASEBKGND => LRESULT(1),
        WM_DESTROY => LRESULT(handlers::on_destroy()),
        // SAFETY: 未処理のメッセージをシステムのデフォルトウィンドウプロシージャへ委ねる。
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
