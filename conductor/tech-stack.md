# Tech Stack - emeditor-terminal

## Programming Language
- **Rust (Edition 2024)**: プラグインの主要なロジックおよびターミナル実装に使用。
  - **最新仕様への準拠**: `unsafe extern` ブロック、`#[unsafe(no_mangle)]` 属性、`if let` 連結など、Edition 2024 の新構文を全面的に適用。
  - `cdylib` 形式でコンパイルし、C ABI を通じて EmEditor SDK と連携。

## Core Technologies
- **EmEditor SDK (C/C++ Interface)**: プラグインのプラグアンドプレイを実現するためのエントリポイント。
- **Windows Pseudo Console (ConptyIoDriver)**: モダンなターミナル機能を提供するためのバックエンド。`CreatePseudoConsole` を利用し、リッチなTUIをサポート。

## Libraries & Frameworks (Rust Crates)
- **windows-rs (v0.62 / Release 73)**: Windows API へのアクセス。
  - **クレート細分化**: `windows-core`, `windows-registry`, `windows-result`, `windows-strings` などの独立クレートを活用し、依存関係の最小化と安全性を向上。
  - **windows-link**: `windows-targets` から移行し、標準化された FFI リンク方式を採用。
  - `Win32_UI_WindowsAndMessaging`: メッセージ送信、ウィンドウプロシージャによるシステムメッセージ (`WM_SYSCOMMAND`, `WM_SYSKEYDOWN`, `WM_ERASEBKGND`) の捕捉と抑制、ダイアログ表示、キャレット制御、**ウィンドウ同期 (`UpdateWindow`)**、**生存確認と破棄 (`IsWindow`, `DestroyWindow`)**。
  - `Win32_Graphics_Gdi`: メモリ DC と互換ビットマップを用いたダブルバッファリング描画の実装。
  - `Win32_Globalization`: 文字コード変換 (CP932 <-> UTF-8)。
  - `Win32_UI_Input_Ime`: IME制御 (Composition String, Candidate Window)。
  - `Win32_UI_Controls`, `Win32_UI_Controls_Dialogs`: リソースベースのダイアログ、および標準フォント選択ダイアログの制御。
  - **windows-registry**: OS のダークモード設定の検出に `windows-rs` 本体の API ではなく、より安全な `windows-registry` クレートを使用。
- **simplelog / log**: デバッグログ出力。
- **vte**: ANSI/VT エスケープシーケンスのパース。業界標準のステートマシン実装により、高信頼・高性能なパースを実現。
- **unicode-width / unicode-segmentation**: 高精度なテキスト測定と書記素クラスター境界判定に使用。
- **which**: システムパスから実行ファイルの絶対パスを探索するために導入。シェル起動の安定性を向上。
  - **Grapheme Clusters 判定**: `unicode-segmentation` により、ユーザーが「1文字」と認識する最小単位を正確に識別。
  - **物理表示幅の正規化**: `unicode-width` に基づきつつ、物理カラムへの割り当てを 1〜2 に制限することで、複雑な絵文字の描画崩れを防止。

## Build Tools & Environment
- **Cargo**: Rust のビルドおよび依存関係管理。
- **embed-resource**: Win32 リソーススクリプト (`.rc`) をビルドプロセスに統合し、バイナリにリソースを埋め込むために使用。
- **Resource ID Sync**: `resource.h` と `build.rs` を連携させ、C 形式のリソースヘッダーから Rust の定数ファイルを自動生成。GUI とロジック間の ID 不整合を防止。
- **MSVC Toolchain**: Windows ネイティブ DLL の生成に使用。
- **Clippy**: Rust の静的解析ツール（リンター）。
- **rustfmt**: Rust のコードフォーマッタ。
  - **style_edition**: 2024 エディションのフォーマット規則に準拠。

## Architecture
**厳格な物理隔離レイヤードアーキテクチャ (Strict Rigid Architecture)** を採用し、ファイル名と配置によって境界を強制している。
- **書記素クラスターベース・物理グリッド管理**: セルごとに可変長文字列を保持し、`pending_cluster` バッファによるストリーム判定を統合。端末標準の CUF/CUB カラム単位移動を維持しつつ、ワイド文字境界の整合性を自動修復する保護ロジックを搭載。
- **Dependency Injection (DI)**: コンストラクタ注入により依存関係を管理し、テスト容易性と結合度の低下を実現。
- **Domain 層 (`src/domain/`)**: `windows` クレートに依存しない Pure Rust 領域。
    - **Entity / Value Object**: `_entity.rs` / `_value.rs`
    - **Configuration**: `TerminalConfig` による構成管理。
    - **Domain Service**: `_domain_service.rs`
    - **Repository (IF)**: `_repository.rs`
- **Application 層 (`src/application/`)**: ユースケースの調整。`_workflow.rs`
- **Infrastructure 層 (`src/infra/`)**: OS/外部 I/O。
    - **Repository Impl**: `_repository_impl.rs`
    - **IO Driver**: `_io_driver.rs` (Win32 API を封印)
- **Presentation / GUI 層 (`src/gui/`)**:
    - **Resolver**: `_resolver.rs` (OSメッセージ解釈・変換)
    - **GUI Driver**: `_gui_driver.rs` (描画・IME・Win32操作を封印)
      - **`window_gui_driver.rs`**: ウィンドウの生存確認、フォーカス、破棄、および親ハンドル (editor_handle) の適切な管理を担う低層ドライバ。
- **FFI 境界 (`src/lib.rs`)**: EmEditor SDK と Rust の仲介役。

