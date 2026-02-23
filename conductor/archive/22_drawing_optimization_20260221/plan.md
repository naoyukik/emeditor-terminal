# Implementation Plan: Issue #22 描画の最適化とちらつき防止 (GDI ダブルバッファリング)

## Phase 0: 調査とドキュメント参照
Microsoft Learn の公式ドキュメントから、GDI ダブルバッファリングと `WM_ERASEBKGND` に関するベストプラクティスを調査する。

- [x] Task: `microsoft_docs_search` を使用して、GDI のダブルバッファリング、`WM_PAINT` におけるメモリDC、`WM_ERASEBKGND` によるフリッカー回避に関する情報を収集する。
- [x] Task: 調査結果を元に、実装における注意点（リソースリーク防止、`BitBlt` 等）を整理する。
- [x] Task: Conductor - User Manual Verification 'Phase 0' (Protocol in workflow.md)
- [x] Task: Commit: `docs(conductor): Phase 0 - Research results`

## Phase 1: 準備とメッセージハンドルの修正
描画最適化の基盤として、背景消去の抑制（WM_ERASEBKGND）を導入する。

- [x] Task: `src/gui/window/mod.rs` において `WM_ERASEBKGND` をハンドルし、背景消去を抑制する。
- [x] Task: `src/gui/resolver/window_message_resolver.rs` に必要な GDI 関数のインポートを追加する。
- [x] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)
- [x] Task: Commit: `fix(gui): Handle WM_ERASEBKGND to suppress flicker`

## Phase 2: ダブルバッファリングの実装
`TerminalGuiDriver::render` をダブルバッファリングに対応させる。

- [x] Task: `TerminalGuiDriver::render` にダブルバッファリングロジックを実装し、オフスクリーン描画を実現する。
- [x] Task: `src/gui/resolver/window_message_resolver.rs` の `on_paint` を修正し、新しいレンダリング・フローを適用する。
- [x] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)
- [ ] Task: Commit: `feat(gui): Implement GDI double buffering in renderer`

## Phase 3: 検証と最適化
描画品質とパフォーマンスの最終確認を行う。

- [x] Task: 実機テストを実施し、フリッカー消失、リサイズ時の品質、リソースリークがないかを確認する。
- [x] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)
- [x] Task: Commit: `chore(conductor): Complete track #22`

## Phase: Review Fixes (指摘事項の修正)
コードレビューで指摘されたリソース管理の堅牢化を行う。

- [x] Task: GDI オブジェクト用の RAII ラッパー（`MemoryDcGuard` 等）を導入し、リソースリークを防止する。
- [x] Task: `BitBlt` の座標指定に関する意図をコメントで明確化する。
- [ ] Task: `render_internal` の冒頭でメモリ DC 全体をデフォルト背景色で塗りつぶし、描画ノイズを完全に排除する。
- [ ] Task: `MemoryDcGuard` を `CreatedDcGuard` にリネームし、`DeleteDC` 専用であることを明示する。
- [ ] Task: Apply review suggestions
