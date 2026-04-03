use crate::infra::driver::keyboard_io_driver::KeyboardIoDriver;
use windows::Win32::Foundation::HWND;

/// GUI-level driver for keyboard hook management.
/// This acts as a bridge to Infrastructure layer to satisfy architectural constraints.
pub struct KeyboardGuiDriver;

impl KeyboardGuiDriver {
    /// Installs the global keyboard hook for the specified window.
    pub fn install(hwnd: HWND) {
        log::info!("Installing global keyboard hook via KeyboardGuiDriver");
        KeyboardIoDriver::install_global(hwnd);
    }

    /// Uninstalls the global keyboard hook.
    pub fn uninstall() {
        log::info!("Uninstalling global keyboard hook via KeyboardGuiDriver");
        KeyboardIoDriver::uninstall_global();
    }
}
