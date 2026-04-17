# Implementation Plan

## フェーズ 1: 問題の把握と詳細設計 (Discovery & Detailed Design)
- [ ] Task: `autonomous-researcher` による詳細調査と `evidence_report.md` の作成
- [ ] Task: 調査結果に基づいた `plan.md` の以降のタスクの具体化。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 1' (調査結果と具体的プランの承認)

## フェーズ 2: 依存関係の更新とビルドエラーの解決 (Dependencies Update & Fix Build Errors)
- [ ] Task: `Cargo.toml` の `windows` クレートのバージョンを 0.73 に更新し、必要な `windows-core`, `windows-registry` などのクレートを追加する。
- [ ] Task: `cargo check` を実行し、発生したコンパイルエラーを解決する（ポインタ型の厳密化、文字列マクロ、モジュールパスの修正など）。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 2' (Protocol in workflow.md)

## フェーズ 3: 動作検証と微調整 (Validation & Refinement)
- [ ] Task: 実機環境（EmEditor）でのテスト実行（描画の安定性、ConPTYプロセス、IME入力、GUI設定の確認）。
- [ ] Task: `cargo fmt` および `cargo clippy` の実行によるコード整形と警告修正。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 3' (Protocol in workflow.md)