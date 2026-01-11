# Tech Stack - emeditor-terminal

## Programming Language
- **Rust**: プラグインの主要なロジックおよびターミナル実装に使用。
  - `cdylib` 形式でコンパイルし、C ABI を通じて EmEditor SDK と連携。

## Core Technologies
- **EmEditor SDK (C/C++ Interface)**: プラグインのプラグアンドプレイを実現するためのエントリポイント。
- **Windows Pseudo Console (ConPTY)**: ターミナルのバックエンド処理。標準入出力とエスケープシーケンスの制御に使用。

## Libraries & Frameworks (Rust Crates)
- **windows-rs / winapi**: Windows API (ConPTY 等) へのアクセス。
- **libc**: FFI 境界での基本的なデータ型とメモリ操作。

## Build Tools & Environment
- **Cargo**: Rust のビルドおよび依存関係管理。
- **MSVC Toolchain**: Windows ネイティブ DLL の生成に使用。
- **Clippy**: Rust の静的解析ツール（リンター）。
- **rustfmt**: Rust のコードフォーマッタ。

## Architecture
- **FFI Layer**: EmEditor SDK と Rust コードを繋ぐ、最小限の `unsafe` 境界。
- **Core Logic (Rust)**: PTY の制御、バッファ管理、非同期入出力処理。
- **UI Glue**: EmEditor のウィンドウハンドル (HWND) にターミナル画面をレンダリングするためのブリッジ層。
