# Tech Stack - emeditor-terminal

## Programming Language
- **Rust**: プラグインの主要なロジックおよびターミナル実装に使用。
  - `cdylib` 形式でコンパイルし、C ABI を通じて EmEditor SDK と連携。

## Core Technologies
- **EmEditor SDK (C/C++ Interface)**: プラグインのプラグアンドプレイを実現するためのエントリポイント。
- **Windows Pseudo Console (ConPTY)**: *Target for future implementation.*
- **Standard Pipes (Current Implementation)**: 
  - `cmd.exe` との通信には標準パイプ (`stdin`, `stdout`, `stderr`) を使用。
  - ウィンドウ表示を抑止 (`CREATE_NO_WINDOW`) し、バックグラウンドで実行。

## Libraries & Frameworks (Rust Crates)
- **windows-rs / winapi**: Windows API へのアクセス。
  - `Win32_UI_WindowsAndMessaging`: メッセージ送信、ダイアログ表示。
  - `Win32_Globalization`: 文字コード変換 (CP932 <-> UTF-8)。
- **simplelog / log**: デバッグログ出力。

## Build Tools & Environment
- **Cargo**: Rust のビルドおよび依存関係管理。
- **MSVC Toolchain**: Windows ネイティブ DLL の生成に使用。
- **Clippy**: Rust の静的解析ツール（リンター）。
- **rustfmt**: Rust のコードフォーマッタ。

## Architecture
- **FFI Layer**: EmEditor SDK と Rust コードを繋ぐ、最小限の `unsafe` 境界 (`lib.rs`)。
- **Session Management**: `ShellSession` (`session.rs`) がバックグラウンドスレッドでプロセスを監視。
- **UI Logic**: 
  - `editor.rs`: アウトプットバーへの出力。
  - `dialog.rs`: 入力ダイアログの構築と表示（メモリ内テンプレート）。