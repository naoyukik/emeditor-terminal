use std::ffi::c_void;
use std::mem::size_of;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::Console::{
    ClosePseudoConsole, CreatePseudoConsole, ResizePseudoConsole, COORD, HPCON,
};
use windows::Win32::System::Pipes::CreatePipe;
use windows::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, InitializeProcThreadAttributeList,
    UpdateProcThreadAttribute, EXTENDED_STARTUPINFO_PRESENT, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, STARTUPINFOEXW,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SendHPCON(pub HPCON);
// SAFETY: HPCON はポインタサイズのハンドルであり、スレッド間での転送は安全。
unsafe impl Send for SendHPCON {}
unsafe impl Sync for SendHPCON {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SendHandle(pub HANDLE);
// SAFETY: HANDLE はポインタサイズのハンドルであり、スレッド間での転送は安全。
unsafe impl Send for SendHandle {}
unsafe impl Sync for SendHandle {}

pub struct ConptyIoDriver {
    pseudo_console_handle: SendHPCON,
    input_write_pipe_handle: SendHandle,
    output_read_pipe_handle: SendHandle,
    process_handle: SendHandle,
    thread_handle: SendHandle,
}

impl ConptyIoDriver {
    pub fn new(
        cmd_line: &str,
        current_dir: Option<&str>,
        width: i16,
        height: i16,
    ) -> Result<Self, String> {
        let mut h_pipe_pt_in = HANDLE::default();
        let mut h_pipe_in_write = HANDLE::default();
        let mut h_pipe_out_read = HANDLE::default();
        let mut h_pipe_pt_out = HANDLE::default();

        let current_dir_w: Vec<u16> = if let Some(dir) = current_dir {
            dir.encode_utf16().chain(std::iter::once(0)).collect()
        } else {
            Vec::new()
        };

        // SAFETY: Win32 API を使用してプロセス間通信用のパイプを作成し、
        // 擬似コンソール (ConPTY) を初期化する。すべてのハンドルは成功時に所有権が
        // 管理され、失敗時には適切にクローズされる。
        unsafe {
            if CreatePipe(&mut h_pipe_pt_in, &mut h_pipe_in_write, None, 0).is_err() {
                return Err("Failed to create input pipe".to_string());
            }

            if CreatePipe(&mut h_pipe_out_read, &mut h_pipe_pt_out, None, 0).is_err() {
                let _ = CloseHandle(h_pipe_pt_in);
                let _ = CloseHandle(h_pipe_in_write);
                return Err("Failed to create output pipe".to_string());
            }

            log::info!(
                "ConptyIoDriver pipes created: in_read={:?}, in_write={:?}, out_read={:?}, out_write={:?}",
                h_pipe_pt_in,
                h_pipe_in_write,
                h_pipe_out_read,
                h_pipe_pt_out
            );

            let size = COORD {
                X: width,
                Y: height,
            };
            let h_pcon = match CreatePseudoConsole(size, h_pipe_pt_in, h_pipe_pt_out, 0) {
                Ok(h) => h,
                Err(e) => {
                    let _ = CloseHandle(h_pipe_pt_in);
                    let _ = CloseHandle(h_pipe_in_write);
                    let _ = CloseHandle(h_pipe_out_read);
                    let _ = CloseHandle(h_pipe_pt_out);
                    return Err(format!("Failed to create pseudo console: {}", e));
                }
            };

            // Close the PTY-side pipe handles as the PTY now owns them
            let _ = CloseHandle(h_pipe_pt_in);
            let _ = CloseHandle(h_pipe_pt_out);

            // Prepare Startup Info
            let mut startup_info_ex = STARTUPINFOEXW::default();
            startup_info_ex.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;

            let mut size: usize = 0;
            let _ = InitializeProcThreadAttributeList(None, 1, Some(0), &mut size);

            let mut attr_list_buffer = vec![0u8; size];
            let lp_attribute_list =
                LPPROC_THREAD_ATTRIBUTE_LIST(attr_list_buffer.as_mut_ptr() as *mut c_void);

            if InitializeProcThreadAttributeList(Some(lp_attribute_list), 1, Some(0), &mut size)
                .is_err()
            {
                ClosePseudoConsole(h_pcon);
                let _ = CloseHandle(h_pipe_in_write);
                let _ = CloseHandle(h_pipe_out_read);
                return Err("Failed to initialize attribute list".to_string());
            }

            if UpdateProcThreadAttribute(
                lp_attribute_list,
                0,
                PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
                Some(h_pcon.0 as *mut c_void),
                size_of::<HPCON>(),
                None,
                None,
            )
            .is_err()
            {
                DeleteProcThreadAttributeList(lp_attribute_list);
                ClosePseudoConsole(h_pcon);
                let _ = CloseHandle(h_pipe_in_write);
                let _ = CloseHandle(h_pipe_out_read);
                return Err("Failed to update attribute list".to_string());
            }

            startup_info_ex.lpAttributeList = lp_attribute_list;

            let mut process_information = PROCESS_INFORMATION::default();

            // CreateProcessW に渡すコマンドラインを構築する。
            // すでに引用符で始まっている場合は、そのまま信頼して使う。
            // そうでない場合は、実行ファイルパスらしき部分のみを引用符で囲む。
            let cmd_line_final = if cmd_line.starts_with('"') {
                cmd_line.to_string()
            } else {
                // 実行ファイル拡張子を探し、そこまでを実行ファイルパスとみなす
                let lower = cmd_line.to_ascii_lowercase();
                let exts = [".exe", ".bat", ".cmd", ".com"];
                let exe_end = exts
                    .iter()
                    .filter_map(|ext| lower.find(ext).map(|idx| idx + ext.len()))
                    .min();

                if let Some(end) = exe_end {
                    let (exe_part, rest_part) = cmd_line.split_at(end);
                    let rest_part = rest_part.trim_start();
                    if rest_part.is_empty() {
                        format!("\"{}\"", exe_part)
                    } else {
                        format!("\"{}\" {}", exe_part, rest_part)
                    }
                } else {
                    // 実行ファイル拡張子が判別できない場合は、上位から渡された文字列をそのまま使用する
                    // (スペースが含まれる単一のパスなら、全体を囲む必要があるかもしれないが、
                    //  which による絶対パス解決後の場合は原則 .exe が含まれるはず)
                    if cmd_line.contains(' ') {
                        format!("\"{}\"", cmd_line)
                    } else {
                        cmd_line.to_string()
                    }
                }
            };

            // Convert cmd_line to wide string (null terminated)
            let mut cmd_line_w: Vec<u16> = cmd_line_final
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            log::debug!("Starting process with command line: {}", cmd_line_final);

            let lp_current_directory = if current_dir_w.is_empty() {
                PCWSTR::null()
            } else {
                PCWSTR(current_dir_w.as_ptr())
            };

            let success = CreateProcessW(
                None,
                Some(PWSTR(cmd_line_w.as_mut_ptr())),
                None,
                None,
                false,
                EXTENDED_STARTUPINFO_PRESENT,
                None,
                lp_current_directory,
                &startup_info_ex.StartupInfo,
                &mut process_information,
            );

            DeleteProcThreadAttributeList(lp_attribute_list);

            if success.is_err() {
                let err = windows::Win32::Foundation::GetLastError();
                log::error!(
                    "Failed to create process. Command: {}, Error: {:?}",
                    cmd_line_final,
                    err
                );
                ClosePseudoConsole(h_pcon);
                let _ = CloseHandle(h_pipe_in_write);
                let _ = CloseHandle(h_pipe_out_read);
                return Err(format!("Failed to create process: {:?}", err));
            }

            Ok(ConptyIoDriver {
                pseudo_console_handle: SendHPCON(h_pcon),
                input_write_pipe_handle: SendHandle(h_pipe_in_write),
                output_read_pipe_handle: SendHandle(h_pipe_out_read),
                process_handle: SendHandle(process_information.hProcess),
                thread_handle: SendHandle(process_information.hThread),
            })
        }
    }

    pub fn get_output_handle(&self) -> SendHandle {
        self.output_read_pipe_handle
    }

    pub fn get_input_handle(&self) -> SendHandle {
        self.input_write_pipe_handle
    }

    pub fn resize(&self, width: i16, height: i16) -> Result<(), String> {
        let size = COORD {
            X: width,
            Y: height,
        };
        // SAFETY: コンソールハンドルは生存しており、リサイズ要求は安全。
        unsafe {
            ResizePseudoConsole(self.pseudo_console_handle.0, size)
                .map_err(|e| format!("Failed to resize pseudo console: {}", e))
        }
    }
}

// SAFETY: ConptyIoDriver は Win32 ハンドルの集合であり、スレッド間での転送は安全。
unsafe impl Send for ConptyIoDriver {}
unsafe impl Sync for ConptyIoDriver {}

impl Drop for ConptyIoDriver {
    fn drop(&mut self) {
        log::info!("ConptyIoDriver dropping... closing handles.");
        // SAFETY: 保持しているプロセス、スレッド、コンソール、およびパイプの
        // ハンドルを確実にクローズし、リソースリークを防ぐ。
        unsafe {
            if self.process_handle.0 != INVALID_HANDLE_VALUE {
                let _ = CloseHandle(self.process_handle.0);
            }
            if self.thread_handle.0 != INVALID_HANDLE_VALUE {
                let _ = CloseHandle(self.thread_handle.0);
            }
            ClosePseudoConsole(self.pseudo_console_handle.0);
            let _ = CloseHandle(self.input_write_pipe_handle.0);
            let _ = CloseHandle(self.output_read_pipe_handle.0);
        }
        log::info!("ConptyIoDriver dropped.");
    }
}
