# Tech Stack - emeditor-terminal

## Programming Language
- **Rust**: プラグインの主要なロジックおよびターミナル実装に使用。
  - `cdylib` 形式でコンパイルし、C ABI を通じて EmEditor SDK と連携。

## Core Technologies
- **EmEditor SDK (C/C++ Interface)**: プラグインのプラグアンドプレイを実現するためのエントリポイント。
- **Windows Pseudo Console (ConPTY)**: モダンなターミナル機能を提供するためのバックエンド。`CreatePseudoConsole` を利用し、リッチなTUIをサポート。

## Libraries & Frameworks (Rust Crates)
- **windows-rs / winapi**: Windows API へのアクセス。
  - `Win32_UI_WindowsAndMessaging`: メッセージ送信、ウィンドウプロシージャによるシステムメッセージ (`WM_SYSCOMMAND`, `WM_SYSKEYDOWN`) の捕捉と抑制、ダイアログ表示、キャレット制御。
  - `Win32_Globalization`: 文字コード変換 (CP932 <-> UTF-8)。
  - `Win32_UI_Input_Ime`: IME制御 (Composition String, Candidate Window)。
- **simplelog / log**: デバッグログ出力。

## Build Tools & Environment
- **Cargo**: Rust のビルドおよび依存関係管理。
- **MSVC Toolchain**: Windows ネイティブ DLL の生成に使用。
- **Clippy**: Rust の静的解析ツール（リンター）。
- **rustfmt**: Rust のコードフォーマッタ。

## Architecture
**レイヤードアーキテクチャ**を採用し、責務を明確に分離している。
- **Domain 層 (`src/domain/`)**: 外部に依存しない純粋なビジネスロジック。`TerminalBuffer`（状態）や `AnsiParser`（パースロジック）を管理。
- **Application 層 (`src/application/`)**: ドメインロジックの調整役。`TerminalService` がターミナルセッションのライフサイクルやスクロール制御、入力処理を統括する。
- **Infra 層 (`src/infra/`)**: OS (Win32) や外部 API (ConPTY, EmEditor SDK) との具体的な対話。
- **GUI 層 (`src/gui/`)**: ユーザーインターフェースと描画。`TerminalRenderer` による GDI レンダリングと、Windows メッセージループ（`wnd_proc`）を担当。
- **FFI 境界 (`src/lib.rs`)**: EmEditor SDK と Rust の仲介役。
