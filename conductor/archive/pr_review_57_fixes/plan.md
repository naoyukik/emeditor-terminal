# Implementation Plan - PR #57 Review Fixes

## Phase 1: 安全性修正
`src/infra/conpty.rs` のライフタイム問題を修正する。

- [x] Task: `src/infra/conpty.rs` の修正
    - [x] `current_dir_w` の定義を `unsafe` ブロックの外側に移動する。
    - [x] `lp_current_directory` の生成ロジックを調整する。
- [x] Task: 検証
    - [x] `cargo build` を実行する。
    - [x] `install.ps1` を実行し、EmEditorでの動作確認を行う。
- [x] Task: Conductor - User Manual Verification (Protocol in workflow.md)
