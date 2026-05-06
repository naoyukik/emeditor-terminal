# Implementation Plan: Gemini CLIによるセキュリティチェック

## フェーズ 1: 脆弱性のスキャンと詳細設計 (Discovery & Detailed Design)
- [ ] Task: `/security:analyze` コマンドを実行し、プロジェクト全体 (`src/`) のセキュリティスキャンを実施する。
- [ ] Task: スキャン結果を分析し、重要度 (Severity) が High/Critical の脆弱性を特定する。
    - [ ] 検出結果と推奨される修正方針を `evidence_report.md` にまとめる。
- [ ] Task: `evidence_report.md` に基づき、フェーズ 2 の修正タスク群（どのファイルのどの行をどう修正するか）を具体化する（plan.md を更新）。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 1' (Protocol in workflow.md)

## フェーズ 2: 脆弱性の修正と検証 (Remediation & Validation)
- [ ] Task: フェーズ 1 で具体化された修正タスクを実施する。
    - [ ] （※このセクションはフェーズ 1 完了後に具体的なタスクで上書きされます）
- [ ] Task: 再スキャンと自動検証を実施する。
    - [ ] 再度 `/security:analyze` を実行し、対象の脆弱性が解消されたことを確認する。
    - [ ] `cargo clippy` および `cargo fmt` を実行し、警告がないことを確認する。
    - [ ] `architecture-validator` などを活用し、アーキテクチャ規約に違反していないか確認する。
- [ ] Task: 実機での動作確認を実施する。
    - [ ] `install.ps1` を実行し、ターミナル機能が正常に動作するか確認する。
- [ ] Task: 修正内容をコミットする（`AGENTS.md` の規約に従う）。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 2' (Protocol in workflow.md)