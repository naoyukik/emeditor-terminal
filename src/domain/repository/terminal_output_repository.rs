use std::io;

#[allow(dead_code)]
pub trait TerminalOutputRepository: Send + Sync {
    fn send_input(&self, input_bytes: &[u8]) -> io::Result<()>;
    fn resize(&self, cols: u16, rows: u16) -> io::Result<()>;
}
