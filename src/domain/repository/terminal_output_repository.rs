use std::io;

pub trait TerminalOutputRepository: Send + Sync {
    fn send_input(&self, data: &[u8]) -> io::Result<()>;
    fn resize(&self, cols: u16, rows: u16) -> io::Result<()>;
}
