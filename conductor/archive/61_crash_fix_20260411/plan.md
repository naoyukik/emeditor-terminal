# Implementation Plan: Issue 61 ターミナル二重起動時のクラッシュ修正

## Phase 1: 問題の把握と詳細設計 (Discovery & Detailed Design)
- [x] Task: 既存の `evidence_report.md` (`conductor/archive/61_autonomous_research_20260409/evidence_report.md`) の読み込みと内容の把握（完了済み）
- [x] Task: 調査結果に基づいた `plan.md` の作成（本計画書、完了済み）
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 1' (調査結果と具体的プランの承認)

## Phase 2: `WindowGuiDriver` の新設 (Preparation)
- [x] Task: `src/gui/driver/window_gui_driver.rs` の作成
    - [x] `WindowGuiDriver` 構造体を定義する。
    - [x] 既存ウィンドウの生存確認とフォーカスを行う `focus_existing_window(hwnd: HWND) -> bool` 静的メソッドを実装する（`IsWindow`, `SetFocus`を使用）。
    - [x] ウィンドウの安全な破棄を行う `destroy_window(hwnd: HWND)` 静的メソッドを実装する。
    - [x] `src/gui/driver/mod.rs` に `window_gui_driver` モジュールを追加して公開する。
- [x] Task: `cargo clippy` および `cargo fmt` の実行
- [x] Task: `feat(gui): Add WindowGuiDriver` のコミット
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 2' (Protocol in workflow.md)

## Phase 3: トランザクション的初期化のリファクタリング (Implementation)
- [x] Task: `open_custom_bar` の処理順序の改善 (`src/gui/window/mod.rs`)
    - [x] 冒頭の `window_handle` チェック処理で、ハンドルが存在する場合は `WindowGuiDriver::focus_existing_window` を呼び出し、成功すれば早期リターン（`false` を返す）するように修正する。
    - [x] 無効なハンドルが残っていた場合は、`WindowGuiDriver::destroy_window` を呼んでクリアする。
- [x] Task: エラー時のクリーンアップ処理の追加 (`src/gui/window/mod.rs`)
    - [x] `start_conpty_and_reader_thread` の呼び出し部分で、戻り値が `false` だった場合の処理を追加する。
    - [x] 失敗時に `WindowGuiDriver::destroy_window` を呼び出し、不完全なウィンドウを破棄する。
    - [x] `window_handle` に `None` を再代入し、ゾンビハンドル状態を解消する。
- [x] Task: `cargo clippy` および `cargo fmt` の実行
- [x] Task: `feat(gui): Refactor open_custom_bar for transactional init` のコミット
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 3' (Protocol in workflow.md)

## Phase 4: 最終検証と品質確認 (Final Validation)
- [x] Task: `cargo test` の実行（リグレッションテスト）
- [x] Task: EmEditor 実機テスト (手動)
    - [x] ターミナル起動済みの状態で、再度起動ボタンを押してもクラッシュしないか確認する。
    - [x] すでにターミナルが起動している場合は、既存のターミナルウィンドウにフォーカスが当たるか確認する。
    - [x] エラー後に再度起動ボタンを押してリカバリできるか確認する。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 4' (Protocol in workflow.md)

## Phase 5: PR Review Fixes (Implementation)
- [x] Task: PR レビュー指摘事項の修正
    - [x] `src/gui/driver/window_gui_driver.rs` のドキュメント修正 (No.1)
    - [x] `conductor/workflow.md` の書式修正 (No.2)
    - [x] `conductor/tracks/61_crash_fix_20260411/plan.md` の記述修正 (No.3)
    - [x] `conductor/tracks/61_crash_fix_20260411/metadata.json` のBOM削除とstatus更新 (No.4, 5)
- [x] Task: `cargo clippy` および `cargo fmt` の実行
- [x] Task: `fix(gui): Apply PR review suggestions for track 61` のコミット
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 5' (Protocol in workflow.md)
