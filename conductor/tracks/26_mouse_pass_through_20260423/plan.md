# Implementation Plan - Mouse Event Pass-through (SGR 1006)

## Phase 1: 調査と詳細設計 (Discovery & Detailed Design)
- [ ] Task: `autonomous-researcher` による詳細調査と `evidence_report.md` の作成
    - 既存の `TerminalInput` および `TerminalWorkflow` における入力処理フローの再確認。
    - EmEditor 本体のマウスイベント通知（`WM_LBUTTONDOWN` 等）の受信方法の特定。
- [ ] Task: 調査結果に基づいた `plan.md` の以降のタスクの具体化。
- [ ] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)

## Phase 2: Domain / Infrastructure 層の実装
- [ ] Task: `domain/model/terminal_input_entity.rs` (仮) へのマウス状態管理の追加
    - 現在のトラッキングモード (1000, 1002, 1003) およびエンコーディングモード (1006) の保持。
- [ ] Task: SGR 1006 シーケンス生成ロジックの実装
- [ ] Task: 各ロジックに対するユニットテストの追加
- [ ] Task: `cargo clippy` および `cargo fmt` の実行
- [ ] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)

## Phase 3: Application / GUI 層の統合と検証
- [ ] Task: `gui/resolver/window_message_resolver.rs` でのマウスメッセージ捕捉の実装
- [ ] Task: Shift キー押下時のバイパス処理（ローカル処理への切り替え）の実装
- [ ] Task: ConPTY 入力パイプへの VT シーケンス書き込み処理の統合
- [ ] Task: 実機（EmEditor + Vim/tmux 等）によるマウス操作の手動検証
- [ ] Task: `cargo clippy` および `cargo fmt` の実行
- [ ] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)
