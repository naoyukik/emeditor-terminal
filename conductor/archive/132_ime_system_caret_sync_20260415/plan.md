# Implementation Plan: Issue 132 - IME System Caret Sync (Clean-Slate Experimentation)

## Phase 1: 問題の把握と詳細設計 (Discovery & Detailed Design)
- [x] Task: `autonomous-researcher` スキルを使用し、関連コードを詳細に調査し、`evidence_report.md` を作成する。
- [x] Task: 調査結果に基づき、`plan.md` の Phase 2 以降のタスクを具体化する。
- [x] Task: Conductor - ユーザー手動検証 'Phase 1'

## Phase 2: 実験基盤の構築 (Foundation & Debug Harness)
- [x] Task: `sync_system_caret` に詳細なログを追加する。
- [x] Task: IME 関連ハンドラにメッセージ受信を知らせるログを追加する。
- [x] Task: この時点での変更を `chore: add debug harness for IME caret sync` としてコミットする。
- [x] Task: Conductor - ユーザー手動検証 'Phase 2'

## Phase 3: 反復検証サイクル (Iterative Experimentation)
- [x] Task: **Hypothesis A/B/C**: 反転属性(SGR 7)を追跡する論理アンカー方式の実装。
- [x] Task: Gemini CLI および Edit での実機検証。
- [x] Task: Conductor - ユーザー手動検証 'Phase 3' (成功を確認)

## Phase 4: 最終統合とクリーンアップ (Final Integration & Cleanup)
- [x] Task: 不要なデバッグログやテスト機構を削除。
- [x] Task: `cargo clippy` および `cargo fmt` を実行。
- [x] Task: 成功した実装内容をコミットする。
- [x] Task: Conductor - ユーザー手動検証 'Phase 4'

## Phase: Review Fixes
- [x] Task: Apply review suggestions 6c6033e
- [x] Task: Restore SGR 4 (Underline) handler in terminal_buffer_entity.rs 83891e3
- [x] Task: Add IsWindow check to WindowGuiDriver::update_window for safety 0da881c
- [x] Task: Implement clamping in get_ime_anchor_pos to handle resize safely 1a9ecc4
- [x] Task: Update on_ime_composition to use get_ime_anchor_pos a3caa0f
- [x] Task: Optimize UpdateWindow: call only during IME composition or when position changes 992dbe9
- [x] Task: Lower log levels for IME/Composition messages from info to debug 8fbd7a1
- [x] Task: Final verification and push updated branch 8fbd7a1
