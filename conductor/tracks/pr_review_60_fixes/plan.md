# Implementation Plan - Track: PR #60 Review Fixes

## Phase 1: Fixes
レビュー指摘事項の修正。

- [ ] Task: `src/gui/custom_bar.rs` の `SB_VERT` 定数修正
    - [ ] `SB_VERT` 独自定義とコメントを削除。
    - [ ] `windows` クレートから `SB_VERT` をインポート。
    - [ ] `SB_VERT.0` を使用するように修正（`SCROLLBAR_CONSTANTS` 型のため）。
- [ ] Task: Conductor - ユーザー手動検証
    - [ ] `cargo check`, `cargo test` を実行。
