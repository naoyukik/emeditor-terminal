# Implementation Plan: Opus 4.6 レビュー指摘修正

## Phase 1: RAII ガードの拡張
`src/gui/driver/terminal_gui_driver.rs` に必要なガードと定数を追加する。

- [x] Task: `SelectedObjectGuard` 構造体と `Drop` 実装を追加。
- [x] Task: `HGDI_ERROR_VALUE` 定数を追加。
- [x] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)
- [x] Task: Commit: `feat(gui): Add SelectedObjectGuard and GDI constants`

## Phase 2: レンダラーの修正
RAII ガードを適用してリソース管理を堅牢化する。

- [x] Task: `render` メソッド内で `SelectedObjectGuard` を使用するように修正。
- [x] Task: `render_internal` 内のブラシ管理を `GdiObjectGuard` に変更。
- [x] Task: `SelectObject` の失敗判定に `HGDI_ERROR_VALUE` を使用。
- [x] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)
- [x] Task: Commit: `fix(gui): Ensure correct GDI object cleanup order using RAII`

## Phase 3: 最終確認
- [ ] Task: 全体ビルドと動作確認を実施。
- [ ] Task: PR #80 への反映（プッシュ）。
- [ ] Task: ファイルサイズ超過とクラススタイル冗長性について、将来のタスクとして記録。
- [ ] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)
- [ ] Task: Commit: `chore(conductor): Complete Opus 4.6 review fixes`
