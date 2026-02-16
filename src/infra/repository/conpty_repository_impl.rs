use crate::domain::repository::terminal_output_repository::TerminalOutputRepository;
use crate::infra::conpty::ConPTY;
use std::io;
use windows::Win32::Storage::FileSystem::WriteFile;

#[allow(dead_code)]
pub struct ConptyRepositoryImpl {
    conpty: ConPTY,
}

#[allow(dead_code)]
impl ConptyRepositoryImpl {
    pub fn new(conpty: ConPTY) -> Self {
        Self { conpty }
    }

    pub fn get_conpty(&self) -> &ConPTY {
        &self.conpty
    }
}

impl TerminalOutputRepository for ConptyRepositoryImpl {
    fn send_input(&self, input_bytes: &[u8]) -> io::Result<()> {
        let handle = self.conpty.get_input_handle();
        let mut bytes_written = 0;
        unsafe {
            WriteFile(handle.0, Some(input_bytes), Some(&mut bytes_written), None)
                .map_err(|e| io::Error::other(e.to_string()))
        }
    }

    fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
        self.conpty
            .resize(cols as i16, rows as i16)
            .map_err(io::Error::other)
    }
}

/// 起動時など、ConPTYがまだ準備できていない場合に使用するダミーリポジトリ
pub struct DummyOutputRepository;

impl TerminalOutputRepository for DummyOutputRepository {
    fn send_input(&self, _data: &[u8]) -> io::Result<()> {
        Ok(())
    }

    fn resize(&self, _cols: u16, _rows: u16) -> io::Result<()> {
        Ok(())
    }
}