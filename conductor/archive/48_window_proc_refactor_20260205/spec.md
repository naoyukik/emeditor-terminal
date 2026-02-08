# Track 48: ウィンドウプロシージャの整理とディスパッチの効率化

## Overview
`src/gui/custom_bar.rs` を `src/gui/window.rs` にリネームし、肥大化したウィンドウプロシージャ (`wnd_proc`) をリファクタリングする。
責務の分離（データ保持、定数定義、メッセージ処理）を徹底し、コードの可読性と保守性を向上させる。
特に、`mod.rs` にロジックを持たせず、状態管理を専用のモジュール (`terminal_data.rs`) に分離することを重視する。

## Functional Requirements
- **リネーム**: `src/gui/custom_bar.rs` を `src/gui/window.rs` に変更する。
- **責務の分離とモジュール構成の最適化**:
  - **ウィンドウ管理 (`src/gui/window.rs`)**:
    - Win32 ウィンドウのライフサイクル（生成・破棄）管理。
    - `wnd_proc` によるメッセージ受信と、各サブモジュールへのディスパッチ。
    - EmEditor からの要求に対する Presentation 層としての窓口。
  - **状態管理 (`src/gui/terminal_data.rs`)**:
    - `TerminalData` 構造体の定義と、グローバルな静的変数 `TERMINAL_DATA` の管理。
    - スレッドセーフなアクセサの提供。
  - **EmEditor連携 (`src/infra/editor.rs` 等へ整理)**:
    - EmEditor SDK 固有の定数（`EE_*`）や構造体の定義を適切な場所（`src/infra/editor.rs` や `src/gui/constants.rs` 等）へ移動。
  - **スレッド管理**:
    - ConPTY出力監視スレッドの起動ロジックを `window.rs` から分離し、適切なサービス層または管理層へ移譲する。

## Non-Functional Requirements
- **コード品質**: `src/gui/window.rs` のファイル行数を 300 行以内に収める。
- **モジュール性**: `mod.rs` はモジュール定義のみを行い、実ロジックを含まないこと。
- **安全性**: グローバルな状態 (`TERMINAL_DATA`) へのアクセスを安全に保つ。
- **互換性**: リファクタリング前後で、ターミナルの動作（描画、入力、IME、スクロール）に変化がないこと。

## Acceptance Criteria
- [ ] `src/gui/window.rs` が作成され、ウィンドウのライフサイクルとディスパッチのみを担当していること。
- [ ] `src/gui/terminal_data.rs` が作成され、`TerminalData` が管理されていること。
- [ ] `mod.rs` にロジックが含まれていないこと。
- [ ] `src/gui/window.rs` の行数が 300 行以内であること。
- [ ] ターミナルの基本機能（起動、コマンド入力、出力表示、スクロール、リサイズ、IME入力）が正常に動作すること。
- [ ] EmEditor 連携用の定数や構造体が整理されていること。

## Out of Scope
- ターミナルの新機能追加。
- `TerminalService` や `ConPTY` 内部ロジックの大幅な変更。
