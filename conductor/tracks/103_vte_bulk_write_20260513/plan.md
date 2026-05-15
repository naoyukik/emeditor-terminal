# 実装計画 (Implementation Plan): vteの一括書き込み最適化 (Issue 103)

## フェーズ 1: 調査と詳細設計 (Discovery & Detailed Design)
- [x] Task: `autonomous-researcher` を用いて、テキスト一括書き込みに向けた現在の `TerminalBufferEntity` と `TerminalProtocolHandler` の安全な改修箇所を特定し、`evidence_report.md` を作成する。
- [x] Task: 調査結果に基づき、本プランのフェーズ 3 以降のタスクを具体的な修正行レベルまで詳細化する。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 1' (調査結果と具体的プランの承認) (Protocol in workflow.md)

## フェーズ 2: ワークフロー更新とブランチ作成 (Workflow & Branch Setup)
- [x] Task: `conductor/workflow.md` を更新し、トラック作成時のブランチ作成ルールを明文化する。
- [x] Task: 本トラック名に基づく Git ブランチを作成する。
- [x] Task: `cargo clippy` および `cargo fmt` を実行し、警告を解消する。
- [x] Task: `AGENTS.md` の規約に従い、コミットを作成する。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 2' (Protocol in workflow.md)

## フェーズ 3: 一括書き込みメソッドの導入 (Bulk Write Implementation)
- [~] Task: `src/domain/model/terminal_buffer_entity.rs` に `print_string(&str)` を実装する。既存の `print_cell` ロジック（書記素クラスタ、行折り返し、反転属性位置の記録）を文字列全体に対して効率的に適用するようにリファクタリングする。
- [ ] Task: `TerminalBufferEntity::print_string` が結合文字、ワイド文字、および行末での折り返しを正しく処理できることを確認するユニットテストを追加する。
- [ ] Task: `cargo clippy` および `cargo fmt` を実行し、警告を解消する。
- [ ] Task: `AGENTS.md` の規約に従い、コミットを作成する。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 3' (Protocol in workflow.md)

## フェーズ 4: プロトコルハンドラーでのバッファリング (Buffering in Protocol Handler)
- [ ] Task: `src/domain/service/terminal_protocol_handler.rs` に `accumulator: String` を追加する。
- [ ] Task: `accumulator` を `print_string` でフラッシュする `flush_accumulator` メソッドを実装し、`Perform` トレイトの全メソッド（`print` 以外）の先頭で呼び出すよう改修する。
- [ ] Task: `TerminalProtocolHandler` が属性変更前に正しくバッファをフラッシュすることを検証するテストを追加する。
- [ ] Task: `cargo clippy` および `cargo fmt` を実行し、警告を解消する。
- [ ] Task: `AGENTS.md` の規約に従い、コミットを作成する。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 4' (Protocol in workflow.md)
