use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

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
        let mut child = Command::new("cmd")
            .args(["/K"]) // Keep session open
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()) // Merge stderr? Or separate? Let's use separate threads or merge.
            .spawn()
            .map_err(|e| e.to_string())?;

        let stdin = child.stdin.take();
        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        let callback = Arc::new(callback);

        // Stdout monitoring thread
        let cb_out = Arc::clone(&callback);
        thread::spawn(move || {
            let mut reader = std::io::BufReader::new(stdout);
            let mut buffer = [0; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        // Ideally convert from Shift-JIS (CP932) to UTF-8 here
                        // For now, use lossy utf8
                        let s = String::from_utf8_lossy(&buffer[..n]).to_string();
                        cb_out(s);
                    }
                    Err(_) => break,
                }
            }
        });

        // Stderr monitoring thread
        let cb_err = Arc::clone(&callback);
        thread::spawn(move || {
            let mut reader = std::io::BufReader::new(stderr);
            let mut buffer = [0; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        let s = String::from_utf8_lossy(&buffer[..n]).to_string();
                        cb_err(s);
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(ShellSession {
            process: Some(child),
            stdin,
        })
    }

    pub fn send(&mut self, command: &str) -> Result<(), String> {
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

        // Wait for initial prompt
        thread::sleep(Duration::from_millis(500));
        
        // Send echo command
        session.send("echo HelloRust").expect("Failed to send command");

        // Collect output for a short duration
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
