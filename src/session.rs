use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

const CREATE_NO_WINDOW: u32 = 0x08000000;

// Callback type for handling output
type OutputCallback = Box<dyn Fn(String) + Send + Sync>;

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
        let mut child = Command::new("cmd")
            .args(["/K"]) // Keep session open
            .creation_flags(CREATE_NO_WINDOW) // Prevent window from appearing
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                log::error!("Failed to spawn process: {}", e);
                e.to_string()
            })?;

        let stdin = child.stdin.take();
        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        let callback = Arc::new(callback);

        // Stdout monitoring thread
        let cb_out = Arc::clone(&callback);
        thread::spawn(move || {
            log::info!("Stdout thread started");
            let mut reader = std::io::BufReader::new(stdout);
            let mut buffer = [0; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        log::info!("Stdout EOF");
                        break;
                    }, 
                    Ok(n) => {
                        log::debug!("Stdout read {} bytes", n);
                        // Ideally convert from Shift-JIS (CP932) to UTF-8 here
                        let s = String::from_utf8_lossy(&buffer[..n]).to_string();
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
            let mut reader = std::io::BufReader::new(stderr);
            let mut buffer = [0; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        log::info!("Stderr EOF");
                        break;
                    },
                    Ok(n) => {
                        log::debug!("Stderr read {} bytes", n);
                        let s = String::from_utf8_lossy(&buffer[..n]).to_string();
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
            stdin,
        })
    }

    pub fn send(&mut self, command: &str) -> Result<(), String> {
        log::info!("Sending command: {}", command);
        if let Some(stdin) = &mut self.stdin {
            writeln!(stdin, "{}", command).map_err(|e| e.to_string())?;
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
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn test_shell_session_echo() {
        let (tx, rx) = mpsc::channel();
        let tx = Arc::new(Mutex::new(tx));

        let mut session = ShellSession::new(move |s| {
            let _ = tx.lock().unwrap().send(s);
        }).expect("Failed to create session");

        thread::sleep(Duration::from_millis(500));
        session.send("echo HelloRust").expect("Failed to send command");

        let start = std::time::Instant::now();
        let mut found = false;
        
        while start.elapsed() < Duration::from_secs(2) {
            if let Ok(msg) = rx.try_recv() {
                if msg.contains("HelloRust") {
                    found = true;
                    break;
                }
            }
            thread::sleep(Duration::from_millis(50));
        }

        assert!(found, "Did not receive echoed output");
    }
}