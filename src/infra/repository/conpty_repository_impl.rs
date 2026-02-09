use crate::domain::repository::terminal_output_repository::TerminalOutputRepository;
use crate::infra::conpty::ConPTY;
use std::io;
use windows::Win32::Storage::FileSystem::WriteFile;

pub struct ConptyRepositoryImpl {
    conpty: ConPTY,
}

impl ConptyRepositoryImpl {
    pub fn new(conpty: ConPTY) -> Self {
        Self { conpty }
    }

    /// ConPTYのインスタンスを取得する（Application層での移行期間用、または特定の用途用）
    pub fn get_conpty(&self) -> &ConPTY {
        &self.conpty
    }
}

impl TerminalOutputRepository for ConptyRepositoryImpl {
    fn send_input(&self, data: &[u8]) -> io::Result<()> {
        let handle = self.conpty.get_input_handle();
        let mut bytes_written = 0;
        unsafe {
            WriteFile(handle.0, Some(data), Some(&mut bytes_written), None)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
        }
    }

    fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
        self.conpty
            .resize(cols as i16, rows as i16)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}
