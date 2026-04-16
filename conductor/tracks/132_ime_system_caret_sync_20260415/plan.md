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
