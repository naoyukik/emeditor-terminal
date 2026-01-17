use windows::core::PWSTR;
use windows::Win32::Foundation::{BOOL, HANDLE, INVALID_HANDLE_VALUE, CloseHandle};
use windows::Win32::System::Console::{
    CreatePseudoConsole, ClosePseudoConsole, COORD, HPCON,
};
use windows::Win32::System::Pipes::CreatePipe;
use windows::Win32::System::Threading::{
    CreateProcessW, InitializeProcThreadAttributeList, UpdateProcThreadAttribute,
    DeleteProcThreadAttributeList, PROCESS_INFORMATION, STARTUPINFOEXW,
    EXTENDED_STARTUPINFO_PRESENT, LPPROC_THREAD_ATTRIBUTE_LIST, PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE,
};
use std::ptr::null_mut;
use std::mem::size_of;
use std::ffi::c_void;

pub struct ConPTY {
    h_pcon: HPCON,
    h_pipe_in_write: HANDLE,
    h_pipe_out_read: HANDLE,
    process_info: PROCESS_INFORMATION,
}

unsafe impl Send for ConPTY {}

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
                CloseHandle(h_pipe_pt_in);
                CloseHandle(h_pipe_in_write);
                return Err("Failed to create output pipe".to_string());
            }

            let size = COORD { X: width, Y: height };
            let h_pcon = match CreatePseudoConsole(size, h_pipe_pt_in, h_pipe_pt_out, 0) {
                Ok(h) => h,
                Err(e) => {
                    CloseHandle(h_pipe_pt_in);
                    CloseHandle(h_pipe_in_write);
                    CloseHandle(h_pipe_out_read);
                    CloseHandle(h_pipe_pt_out);
                    return Err(format!("Failed to create pseudo console: {}", e));
                }
            };

            // Close the PTY-side pipe handles as the PTY now owns them
            CloseHandle(h_pipe_pt_in);
            CloseHandle(h_pipe_pt_out);

            // Prepare Startup Info
            let mut si_ex = STARTUPINFOEXW::default();
            si_ex.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
            
            let mut size: usize = 0;
            let _ = InitializeProcThreadAttributeList(LPPROC_THREAD_ATTRIBUTE_LIST(null_mut()), 1, 0, &mut size);
            
            let mut attr_list_buffer = vec![0u8; size];
            let lp_attribute_list = LPPROC_THREAD_ATTRIBUTE_LIST(attr_list_buffer.as_mut_ptr() as *mut c_void);
            
            if InitializeProcThreadAttributeList(lp_attribute_list, 1, 0, &mut size).is_err() {
                 ClosePseudoConsole(h_pcon);
                 CloseHandle(h_pipe_in_write);
                 CloseHandle(h_pipe_out_read);
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
            ).is_err() {
                DeleteProcThreadAttributeList(lp_attribute_list);
                ClosePseudoConsole(h_pcon);
                CloseHandle(h_pipe_in_write);
                CloseHandle(h_pipe_out_read);
                return Err("Failed to update attribute list".to_string());
            }
            
            si_ex.lpAttributeList = lp_attribute_list;

            let mut pi = PROCESS_INFORMATION::default();
            // Convert cmd_line to wide string (null terminated)
            let mut cmd_line_w: Vec<u16> = cmd_line.encode_utf16().chain(std::iter::once(0)).collect();

            let success = CreateProcessW(
                None,
                PWSTR(cmd_line_w.as_mut_ptr()), // PWSTR expects *mut u16
                None,
                None,
                BOOL(0),
                EXTENDED_STARTUPINFO_PRESENT,
                None,
                None,
                &si_ex.StartupInfo,
                &mut pi,
            );

            DeleteProcThreadAttributeList(lp_attribute_list);

            if success.is_err() {
                ClosePseudoConsole(h_pcon);
                CloseHandle(h_pipe_in_write);
                CloseHandle(h_pipe_out_read);
                return Err("Failed to create process".to_string());
            }

            Ok(ConPTY {
                h_pcon,
                h_pipe_in_write,
                h_pipe_out_read,
                process_info: pi,
            })
        }
    }
    
    pub fn get_output_handle(&self) -> HANDLE {
        self.h_pipe_out_read
    }
}

impl Drop for ConPTY {
    fn drop(&mut self) {
        unsafe {
            if self.process_info.hProcess != INVALID_HANDLE_VALUE {
                CloseHandle(self.process_info.hProcess);
            }
            if self.process_info.hThread != INVALID_HANDLE_VALUE {
                CloseHandle(self.process_info.hThread);
            }
            ClosePseudoConsole(self.h_pcon);
            CloseHandle(self.h_pipe_in_write);
            CloseHandle(self.h_pipe_out_read);
        }
    }
}
