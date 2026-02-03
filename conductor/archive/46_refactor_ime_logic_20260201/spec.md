# Track: refactor: custom_bar.rs から IME ハンドリングロジックを抽出 (Issue #46)

## Overview
`src/gui/custom_bar.rs` に含まれる肥大化した IME (Input Method Editor) 関連の処理を、新規モジュール `src/gui/ime.rs` に抽出・分離する。
これにより、メインウィンドウプロシージャ (`wnd_proc`) の可読性を向上させ、Win32 API に依存する複雑なロジックをカプセル化する。
本リファクタリングでは機能的な変更は行わず、既存の挙動（TUI上でのIME入力、候補ウィンドウの表示位置追従など）を維持する。

## Functional Requirements
- **IMEロジックの分離**: `custom_bar.rs` 内の以下の処理を `src/gui/ime.rs` に移動・再実装する。
    - IMEウィンドウ位置の更新 (`update_ime_window_position`)
    - IME入力中状態の判定 (`is_ime_composing`)
    - `WM_IME_COMPOSITION` 等のメッセージ処理ロジック
- **インターフェース設計**:
    - `ime.rs` はステートレスなヘルパー関数群を提供する。
    - `wnd_proc` は `TerminalData` のロックを取得した後、必要なデータ（`TerminalService`, `CompositionData` への参照など）を `ime.rs` の関数に渡す。
- **既存機能の維持**:
    - IME入力中の未確定文字列（Composition String）が正しく取得・表示されること。
    - 確定文字列（Result String）が正しく `TerminalService` に送信されること。
    - カーソル位置に合わせてIME候補ウィンドウ（Candidate Window）が追従すること。

## Technical Implementation
- **新規ファイル**: `src/gui/ime.rs` を作成。
- **データ構造**: `CompositionData` は `src/gui/renderer.rs` に維持する。
- **関数シグネチャ（例）**:
    ```rust
    // src/gui/ime.rs
    pub fn handle_composition(
        hwnd: HWND,
        lparam: LPARAM,
        service: &mut TerminalService,
        composition: &mut Option<CompositionData>
    ) -> bool; // 処理を行った場合 true, DefWindowProcへ流す場合 false

    pub fn handle_start_composition(hwnd: HWND, service: &mut TerminalService);
    pub fn update_window_position(hwnd: HWND, renderer: &TerminalRenderer, service: &TerminalService);
    ```

## Acceptance Criteria
- [ ] `src/gui/ime.rs` が作成され、IME関連のロジックが移動されていること。
- [ ] `src/gui/custom_bar.rs` から直接的な `Imm*` 系 Win32 API 呼び出しが削除（または `ime.rs` 経由に置換）されていること。
- [ ] リファクタリング後も、日本語入力（IME）が正常に機能すること。
    - 入力中の文字が表示される。
    - 変換候補ウィンドウがカーソル位置に表示される。
    - 確定した文字が端末に入力される。
- [ ] `cargo clippy` および `cargo test` がパスすること。

## Out of Scope
- `CompositionData` 構造体の定義場所変更（Application層への移動など）。
- IME以外の入力処理（キーボードフックなど）のリファクタリング。
