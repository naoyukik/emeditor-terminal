# Installation Instructions

## Prerequisites
- EmEditor (64-bit)
- Rust Toolchain (cargo)

## Building and Installing
To build and install the plugin automatically to your EmEditor plugins folder:

1. Open a PowerShell terminal in the project root.
2. Run the install script:
   ```powershell
   .\install.ps1
   ```
   For a release build:
   ```powershell
   .\install.ps1 --release
   ```

## Manual Installation
1. Build the project:
   ```bash
   cargo build --release
   ```
2. Copy `target/release/emeditor_terminal.dll` to:
   `$env:LOCALAPPDATA\Programs\EmEditor\PlugIns`

3. Restart EmEditor.
