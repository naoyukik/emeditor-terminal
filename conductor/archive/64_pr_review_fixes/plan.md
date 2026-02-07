# 計画: PR #62 Review Fixes

## Phase 1: 安全性と整合性の修正
レビュー指摘事項を順次修正する。

- [ ] Task: `src/infra/input.rs` の `WM_APP_REPAINT` に関するコメントを修正する。
- [ ] Task: `src/gui/window/mod.rs` の `RegisterClassW` 呼び出しにエラーハンドリングを追加する。
- [ ] Task: `src/gui/window/mod.rs` の `start_conpty_and_reader_thread` を修正し、`SendHWND` を使用してスレッドにハンドルを渡すようにする。
- [ ] Task: Conductor - User Manual Verification 'Phase 1: 安全性と整合性の修正' (Protocol in workflow.md)
