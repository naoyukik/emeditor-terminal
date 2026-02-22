# Implementation Plan: PR #80 レビュー指摘修正

## Phase 1: レンダラーの堅牢化
`TerminalGuiDriver::render` 内部の GDI 操作に対するチェックを追加する。

- [ ] Task: `SelectObject` の戻り値を検証し、失敗時にログ出力して早期リターンする。
- [ ] Task: 背景クリア処理において、ブラシ作成失敗時のフォールバック処理を実装する。
- [ ] Task: `BitBlt` の戻り値をチェックし、失敗時にログ出力する。
- [ ] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)
- [ ] Task: Commit: `fix(gui): Strengthen GDI error handling in renderer`

## Phase 2: メッセージ処理の最適化
ウィンドウメッセージハンドラ側の修正を行う。

- [ ] Task: `src/gui/resolver/window_message_resolver.rs` の `on_size` 内の `InvalidateRect` 引数を修正する。
- [ ] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)
- [ ] Task: Commit: `perf(gui): Optimize InvalidateRect calls in WM_SIZE`

## Phase 3: 最終確認
- [ ] Task: 全体ビルドと動作確認を実施。
- [ ] Task: PR #80 への反映。
- [ ] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)
- [ ] Task: Commit: `chore(conductor): Complete PR #80 review fixes`
