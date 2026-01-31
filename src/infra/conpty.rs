use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::null_mut;
use windows::core::{PWSTR, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, BOOL, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::Console::{
    ClosePseudoConsole, CreatePseudoConsole, ResizePseudoConsole, COORD, HPCON,
};
use windows::Win32::System::Pipes::CreatePipe;
use windows::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, InitializeProcThreadAttributeList,
    UpdateProcThreadAttribute, EXTENDED_STARTUPINFO_PRESENT, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, STARTUPINFOEXW,
};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SendHPCON(pub HPCON);
unsafe impl Send for SendHPCON {}
unsafe impl Sync for SendHPCON {}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SendHandle(pub HANDLE);
unsafe impl Send for SendHandle {}
unsafe impl Sync for SendHandle {}

pub struct ConPTY {
    h_pcon: SendHPCON,
    h_pipe_in_write: SendHandle,
    h_pipe_out_read: SendHandle,
    h_process: SendHandle,
    h_thread: SendHandle,
}

impl ConPTY {
    pub fn new(cmd_line: &str, width: i16, height: i16) -> Result<Self, String> {
        let mut h_pipe_pt_in = INVALID_HANDLE_VALUE;
        let mut h_pipe_in_write = INVALID_HANDLE_VALUE;
        let mut h_pipe_pt_out = INVALID_HANDLE_VALUE;
        let mut h_pipe_out_read = INVALID_HANDLE_VALUE;

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
                "ConPTY pipes created: in_read={:?}, in_write={:?}, out_read={:?}, out_write={:?}",
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
            let mut si_ex = STARTUPINFOEXW::default();
            si_ex.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;

            let mut size: usize = 0;
            let _ = InitializeProcThreadAttributeList(
                LPPROC_THREAD_ATTRIBUTE_LIST(null_mut()),
                1,
                0,
                &mut size,
            );

            let mut attr_list_buffer = vec![0u8; size];
            let lp_attribute_list =
                LPPROC_THREAD_ATTRIBUTE_LIST(attr_list_buffer.as_mut_ptr() as *mut c_void);

            if InitializeProcThreadAttributeList(lp_attribute_list, 1, 0, &mut size).is_err() {
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

            si_ex.lpAttributeList = lp_attribute_list;

            let mut pi = PROCESS_INFORMATION::default();
            // Convert cmd_line to wide string (null terminated)
            let mut cmd_line_w: Vec<u16> =
                cmd_line.encode_utf16().chain(std::iter::once(0)).collect();

            // Get USERPROFILE for current directory
            let current_dir = std::env::var("USERPROFILE").ok();
            let current_dir_w: Vec<u16> = if let Some(dir) = current_dir {
                dir.encode_utf16().chain(std::iter::once(0)).collect()
            } else {
                Vec::new()
            };
            
            let lp_current_directory = if current_dir_w.is_empty() {
                PCWSTR::null()
            } else {
                PCWSTR(current_dir_w.as_ptr())
            };

            let success = CreateProcessW(
                None,
                PWSTR(cmd_line_w.as_mut_ptr()),
                None,
                None,
                BOOL(0),
                EXTENDED_STARTUPINFO_PRESENT,
                None,
                lp_current_directory,
                &si_ex.StartupInfo,
                &mut pi,
            );

            DeleteProcThreadAttributeList(lp_attribute_list);

            if success.is_err() {
                ClosePseudoConsole(h_pcon);
                let _ = CloseHandle(h_pipe_in_write);
                let _ = CloseHandle(h_pipe_out_read);
                return Err("Failed to create process".to_string());
            }

            Ok(ConPTY {
                h_pcon: SendHPCON(h_pcon),
                h_pipe_in_write: SendHandle(h_pipe_in_write),
                h_pipe_out_read: SendHandle(h_pipe_out_read),
                h_process: SendHandle(pi.hProcess),
                h_thread: SendHandle(pi.hThread),
            })
        }
    }

    pub fn get_output_handle(&self) -> SendHandle {
        self.h_pipe_out_read
    }

    pub fn get_input_handle(&self) -> SendHandle {
        self.h_pipe_in_write
    }

    pub fn resize(&self, width: i16, height: i16) -> Result<(), String> {
        let size = COORD {
            X: width,
            Y: height,
        };
        unsafe {
            ResizePseudoConsole(self.h_pcon.0, size)
                .map_err(|e| format!("Failed to resize pseudo console: {}", e))
        }
    }
}

unsafe impl Send for ConPTY {}
unsafe impl Sync for ConPTY {}

impl Drop for ConPTY {
    fn drop(&mut self) {
        log::info!("ConPTY dropping... closing handles.");
        unsafe {
            if self.h_process.0 != INVALID_HANDLE_VALUE {
                let _ = CloseHandle(self.h_process.0);
            }
            if self.h_thread.0 != INVALID_HANDLE_VALUE {
                let _ = CloseHandle(self.h_thread.0);
            }
            ClosePseudoConsole(self.h_pcon.0);
            let _ = CloseHandle(self.h_pipe_in_write.0);
            let _ = CloseHandle(self.h_pipe_out_read.0);
        }
        log::info!("ConPTY dropped.");
    }
}
