# Tech Stack - emeditor-terminal

## Programming Language
- **Rust**: プラグインの主要なロジックおよびターミナル実装に使用。
  - `cdylib` 形式でコンパイルし、C ABI を通じて EmEditor SDK と連携。

## Core Technologies
- **EmEditor SDK (C/C++ Interface)**: プラグインのプラグアンドプレイを実現するためのエントリポイント。
- **Windows Pseudo Console (ConptyIoDriver)**: モダンなターミナル機能を提供するためのバックエンド。`CreatePseudoConsole` を利用し、リッチなTUIをサポート。

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
**厳格な物理隔離レイヤードアーキテクチャ (Strict Rigid Architecture)** を採用し、ファイル名と配置によって境界を強制している。
- **Dependency Injection (DI)**: コンストラクタ注入により依存関係を管理し、テスト容易性と結合度の低下を実現。
- **Domain 層 (`src/domain/`)**: `windows` クレートに依存しない Pure Rust 領域。
    - **Entity / Value Object**: `_entity.rs` / `_value.rs`
    - **Domain Service**: `_domain_service.rs`
    - **Repository (IF)**: `_repository.rs`
- **Application 層 (`src/application/`)**: ユースケースの調整。`_workflow.rs`
- **Infrastructure 層 (`src/infra/`)**: OS/外部 I/O。
    - **Repository Impl**: `_repository_impl.rs`
    - **IO Driver**: `_io_driver.rs` (Win32 API を封印)
- **Presentation / GUI 層 (`src/gui/`)**:
    - **Resolver**: `_resolver.rs` (OSメッセージ解釈・変換)
    - **GUI Driver**: `_gui_driver.rs` (描画・IME・Win32操作を封印)
- **FFI 境界 (`src/lib.rs`)**: EmEditor SDK と Rust の仲介役。
