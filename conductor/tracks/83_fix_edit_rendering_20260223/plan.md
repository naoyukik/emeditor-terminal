# Implementation Plan - Track 83: fix: edit等のアプリケーションにおける表示ずれの修正

## Phase 1: 調査と再現環境の構築 (Investigation \u0026 Reproduction)
- [ ] Task: `edit` (Microsoft Edit) のインストールと、問題が発生するファイルの特定
- [ ] Task: 制御シーケンスのロギングを有効にし、スクロール時のシーケンスをキャプチャする
- [ ] Task: `DECSTBM` (Scrolling Region) およびカーソル移動シーケンスの現在の挙動の不一致箇所を特定する
- [ ] Task: chore(conductor): Phase 1: 調査完了のコミット
- [ ] Task: Conductor - User Manual Verification 'Phase 1: 調査完了' (Protocol in workflow.md)

## Phase 2: バッファ管理とシーケンス処理の修正 (Core Logic Fix)
- [ ] Task: `src/domain/model/terminal_buffer.rs` (または関連モジュール) のスクロール領域制限のバグ修正
- [ ] Task: 垂直スクロール (`CSI <n> S`, `CSI <n> T`) シーケンスのハンドリングの修正
- [ ] Task: カーソル位置がスクロール領域外にある場合の例外的な挙動（改行時の扱い等）を修正
- [ ] Task: 抽出したシーケンスに基づく、既存の `AnsiParser` および `TerminalBuffer` のユニットテストの強化
- [ ] Task: fix: edit等のアプリケーションにおける表示ずれの修正（ロジック修正）
- [ ] Task: Conductor - User Manual Verification 'Phase 2: ロジック修正完了' (Protocol in workflow.md)

## Phase 3: 最終検証と品質向上 (Final Verification \u0026 Polishing)
- [ ] Task: `edit` におけるスクロール、メニュー操作、罫線描画の正常動作確認
- [ ] Task: 他の TUI アプリ (WSL/vi等) でのデグレードがないことを確認
- [ ] Task: 不要なログ出力の削除と、ソースコードのクリーンアップ (`cargo clippy`, `cargo fmt`)
- [ ] Task: style: 表示ずれ修正に伴うコードのクリーンアップ
- [ ] Task: Conductor - User Manual Verification 'Phase 3: 全工程完了' (Protocol in workflow.md)