# Implementation Plan: PR #65 Review Fixes

## Phase 1: Resource Management & Safety
- [x] `src/gui/window/mod.rs` の `cleanup_terminal` を修正し、`TerminalService` を drop する
- [x] `src/gui/window/handlers.rs` の `on_destroy` (WM_DESTROY) で `cleanup_terminal` を呼び出す
- [x] `src/gui/terminal_data.rs` の `SendHWND` に `SAFETY` コメントを追加する
- [x] `src/application/service.rs` の `resize` にエラーハンドリング（ログ出力）を追加する

## Phase 2: Documentation Cleanup
- [x] `conductor/archive/50_repository_pattern_di_20260208/spec.md` のファイル名を修正する
- [x] `conductor/archive/50_repository_pattern_di_20260208/plan.md` のテスト項目のチェックを外す

## Phase 3: Verification
- [x] ビルドが通ることを確認する (`cargo build`)
- [x] 既存のテストがパスすることを確認する (`cargo test`)
- [x] アプリケーションを起動し、ウィンドウを閉じた際に ConPTY プロセスが終了することを確認する（手動確認）
