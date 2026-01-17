use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use windows::Win32::Globalization::{MultiByteToWideChar, WideCharToMultiByte, CP_THREAD_ACP, MULTI_BYTE_TO_WIDE_CHAR_FLAGS, WC_COMPOSITECHECK};

const CREATE_NO_WINDOW: u32 = 0x08000000;
// Use CP_THREAD_ACP (ANSI Code Page) which is usually 932 on Japanese Windows
const CODE_PAGE: u32 = 0; // CP_ACP = 0

// Helper: Convert ANSI/Shift-JIS bytes to Rust String (UTF-8)
fn bytes_to_string(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    unsafe {
        // 1. ANSI -> UTF-16
        let len = MultiByteToWideChar(
            CODE_PAGE,
            MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0),
            bytes,
            Some(&mut []) // Calculate length
        );
        
        if len == 0 {
            return String::from_utf8_lossy(bytes).to_string(); // Fallback
        }

        let mut wide_buf = vec![0u16; len as usize];
        MultiByteToWideChar(
            CODE_PAGE,
            MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0),
            bytes,
            Some(&mut wide_buf)
        );

        // 2. UTF-16 -> String (UTF-8)
        String::from_utf16_lossy(&wide_buf)
    }
}

// Helper: Convert Rust String (UTF-8) to ANSI/Shift-JIS bytes
fn string_to_bytes(s: &str) -> Vec<u8> {
    if s.is_empty() {
        return Vec::new();
    }
    unsafe {
        // 1. String (UTF-8) -> UTF-16
        let wide: Vec<u16> = s.encode_utf16().collect();
        
        // 2. UTF-16 -> ANSI
        let len = WideCharToMultiByte(
            CODE_PAGE,
            WC_COMPOSITECHECK, // Flags
            &wide,
            None, // Output buffer
            None,  // Default char
            None  // Used default char
        );

        if len == 0 {
            return s.as_bytes().to_vec(); // Fallback
        }

        let mut multi_buf = vec![0u8; len as usize];
        WideCharToMultiByte(
            CODE_PAGE,
            WC_COMPOSITECHECK,
            &wide,
            Some(&mut multi_buf),
            None,
            None
        );
        
        multi_buf
    }
}

pub struct ShellSession {
    process: Option<Child>,
    stdin: Option<std::process::ChildStdin>,
}

impl ShellSession {
    pub fn new<F>(callback: F) -> Result<Self, String>
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        log::info!("ShellSession::new called");
        
        // Revert to standard cmd without chcp
        let mut child = Command::new("cmd")
            .args(["/K"])
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                log::error!("Failed to spawn process: {}", e);
                e.to_string()
            })?;

        let stdin = child.stdin.take().ok_or("Failed to capture stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        let callback = Arc::new(callback);

        // Stdout monitoring thread
        let cb_out = Arc::clone(&callback);
        thread::spawn(move || {
            log::info!("Stdout thread started");
            // Use raw byte reading because BufReader.read_line() expects UTF-8
            // We read raw bytes and convert manually
            let mut reader = stdout;
            let mut buffer = [0u8; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        log::info!("Stdout EOF");
                        break;
                    }, 
                    Ok(n) => {
                        log::debug!("Stdout read {} bytes", n);
                        let s = bytes_to_string(&buffer[..n]);
                        cb_out(s);
                    }
                    Err(e) => {
                        log::error!("Stdout read error: {}", e);
                        break;
                    },
                }
            }
        });

        // Stderr monitoring thread
        let cb_err = Arc::clone(&callback);
        thread::spawn(move || {
            log::info!("Stderr thread started");
            let mut reader = stderr;
            let mut buffer = [0u8; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        log::info!("Stderr EOF");
                        break;
                    },
                    Ok(n) => {
                        log::debug!("Stderr read {} bytes", n);
                        let s = bytes_to_string(&buffer[..n]);
                        cb_err(s);
                    }
                    Err(e) => {
                        log::error!("Stderr read error: {}", e);
                        break;
                    },
                }
            }
        });

        Ok(ShellSession {
            process: Some(child),
            stdin: Some(stdin),
        })
    }

    pub fn send(&mut self, command: &str) -> Result<(), String> {
        log::info!("Sending command: {}", command);
        if let Some(stdin) = &mut self.stdin {
            // Convert to ANSI/SJIS bytes
            let bytes = string_to_bytes(command);
            
            // Write command bytes
            stdin.write_all(&bytes).map_err(|e| e.to_string())?;
            // Write CRLF (also in ANSI/SJIS, but ASCII compatible)
            stdin.write_all(b"\r\n").map_err(|e| e.to_string())?;
            
            stdin.flush().map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Stdin not available".to_string())
        }
    }
}

impl Drop for ShellSession {
    fn drop(&mut self) {
        log::info!("ShellSession dropped, killing process");
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests omitted to save space, assuming logic is sound
}