# Implementation Plan - Track: refactor: custom_bar.rs から IME ハンドリングロジックを抽出 (Issue #46)

## Phase 1: セットアップとロジック抽出
IMEハンドリングロジックを `ime.rs` に切り出し、`custom_bar.rs` から利用できるようにする。

- [x] Task: `src/gui/ime.rs` モジュールの作成
    - [x] `src/gui/ime.rs` ファイルを作成する。
    - [x] `src/gui/mod.rs` に `mod ime;` を追加する。
    - [x] 必要なインポート（Win32 API, 内部構造体）を `custom_bar.rs` からコピーする。
- [x] Task: ヘルパー関数の抽出
    - [x] `update_ime_window_position` のロジックを `ime::update_window_position` に移動する。
    - [x] `is_ime_composing` のロジックを `ime::is_composing` に移動する。
    - [x] `custom_bar.rs` がこれらの新しい関数を使用するように修正する。
    - [x] コンパイル確認 (`cargo check`)。
- [x] Task: メッセージハンドラの抽出
    - [x] `WM_IME_COMPOSITION` を処理する `ime::handle_composition` を実装する。
    - [x] `WM_IME_STARTCOMPOSITION` を処理する `ime::handle_start_composition` を実装する。
    - [x] `ime::handle_end_composition` を実装する（クリーンアップロジックが必要な場合）。
- [x] Task: `custom_bar.rs` への統合
    - [x] `wnd_proc` 内のインラインロジック（`WM_IME_*` ケース）を `ime::*` 関数への呼び出しに置き換える。
    - [x] `wnd_proc` 内でのロック管理とデータ渡しが適切に行われていることを確認する。
- [x] Task: Conductor - ユーザー手動検証 'Phase 1' (Protocol in workflow.md)
    - [x] コードがエラーなくコンパイルできることを確認する。
    - [x] `custom_bar.rs` のコードが大幅に整理されたことを確認する。

## Phase 2: 検証とクリーンアップ
リファクタリング後の動作確認と、不要なコードの削除を行う。

- [x] Task: `custom_bar.rs` のクリーンアップ
    - [x] `custom_bar.rs` から未使用のインポート（`Imm*` 関数、定数など）を削除する。
    - [x] `cargo clippy` を実行し、警告を修正する。
- [ ] Task: 手動検証 (IME機能)
    - [ ] EmEditor でプラグインを起動する。
    - [ ] IME入力（入力、変換、確定）をテストする。
    - [ ] 候補ウィンドウ（Candidate Window）がカーソル位置に追従することを確認する。
    - [ ] 基本的なターミナル入力に退行（リグレッション）がないことを確認する。
- [ ] Task: Conductor - ユーザー手動検証 'Phase 2' (Protocol in workflow.md)
    - [ ] すべての受け入れ基準が満たされていることを確認する。
