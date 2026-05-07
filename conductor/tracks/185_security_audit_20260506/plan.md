# Implementation Plan: Gemini CLIによるセキュリティチェック

## フェーズ 1: 脆弱性のスキャンと詳細設計 (Discovery & Detailed Design)
- [x] Task: `/security:analyze` コマンドを実行し、プロジェクト全体 (`src/`) のセキュリティスキャンを実施する。
- [x] Task: スキャン結果を分析し、重要度 (Severity) が High/Critical の脆弱性を特定する。
    - [x] 検出結果と推奨される修正方針を `evidence_report.md` にまとめる。
- [x] Task: `evidence_report.md` に基づき、フェーズ 2 の修正タスク群（どのファイルのどの行をどう修正するか）を具体化する（plan.md を更新）。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 1' (Protocol in workflow.md)

## フェーズ 2: 脆弱性の修正と検証 (Remediation & Validation)
- [x] Task: 依存関係の脆弱性修正 (pnpm-lock.yaml)
    - [x] `pnpm update jws picomatch qs` を実行し、脆弱性が修正されたバージョンに更新する。
- [x] Task: ソースコードの安全性向上 (Safety Comments & Review)
    - [x] `src/gui/driver/terminal_gui_driver.rs` の `unsafe` ブロックに Safety Comment を追加・検証する。
    - [x] `src/gui/window/mod.rs` の `unsafe` ブロックに Safety Comment を追加・検証する。
    - [x] その他、`Safety Comment` が欠落している全ての `unsafe` 箇所を補完する。
- [x] Task: 再スキャンと自動検証を実施する。
    - [x] 再度 `/security:analyze` を実行し、対象の脆弱性が解消されたことを確認する。
    - [x] `cargo clippy` および `cargo fmt` を実行し、警告がないことを確認する。
    - [x] `architecture-validator` などを活用し、アーキテクチャ規約に違反していないか確認する。
- [x] Task: 実機での動作確認を実施する。
    - [x] `install.ps1` を実行し、ターミナル機能が正常に動作するか確認する。 (※ユーザー承認待ち)
- [x] Task: 修正内容をコミットする（`AGENTS.md` の規約に従う）。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 2' (Protocol in workflow.md)

## フェーズ 3: PR 191 レビュー指摘の反映
- [x] Task: `src/gui/window/mod.rs` の `open_custom_bar` に対する PR コメントを再評価する。
    - [x] `CLASS_REGISTERED` の `load`→`RegisterClassW`→`store` 競合を、`compare_exchange` または `ERROR_CLASS_ALREADY_EXISTS` 許容で解消する方針を確定する。
    - [x] 実装を変えない場合でも、`// SAFETY:` コメントを「単一スレッド前提」など実際の保証範囲に合わせて修正する。
- [x] Task: `src/gui/driver/terminal_gui_driver.rs` の `BitBlt` 周辺の `// SAFETY:` コメントを修正する。
    - [x] `hdc` の有効性が呼び出し元前提であることと、その前提の成立箇所を明記する。
- [x] Task: PR 191 レビュー対応後に自動検証を再実行する。
    - [x] `cargo fmt`
    - [x] `cargo clippy`
