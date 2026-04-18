# Implementation Plan

## フェーズ 1: 問題の把握と詳細設計 (Discovery & Detailed Design)
- [x] Task: `autonomous-researcher` による詳細調査と `evidence_report.md` の作成
- [x] Task: 調査結果に基づいた `plan.md` の以降のタスクの具体化。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 1' (調査結果と具体的プランの承認)

## フェーズ 2: マイルストーン A - 基盤刷新 (v0.58 -> v0.62)
- **主な変更**: クレート分割 (`numerics`, `future`, `collections`), `BOOL` の `result` への移動, `windows-link` の導入。
- [x] Task: `Cargo.toml` を v0.62 相当へ更新。
- [x] Task: インポートパスの修正（`BOOL` 等を `windows-result` / `windows-core` へ）。
- [x] Task: `HSTRING` の `Deref` 実装に伴うコード簡略化。
- [x] Task: コンパイル確認 (`cargo check`) とユニットテスト。
- [x] Task: `cargo clippy` および `cargo fmt` の実行。
- [x] Task: `@arch-auditor` によるレビューを実施。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 2' (基盤刷新の動作確認)
- [x] Task: フェーズ成果物のコミット

## フェーズ 3: マイルストーン B - Safe API 刷新と Windows-Link 完全移行
- **主な変更**: `windows-link` への完全移行、`windows-registry` による Safe レジストリ操作への刷新。
- [x] Task: `windows-targets` への依存を完全に排除し、`windows-link` によるリンク方式を標準化。
- [x] Task: `src/infra/repository/emeditor_config_repository_impl.rs` 等のレジストリ操作を `windows-registry` クレートへ移行。
- [x] Task: コンパイル確認 (`cargo check`)。
- [x] Task: `@arch-auditor` によるレビューを実施（Win32 API 隔離状況の確認）。
- [x] Task: `cargo clippy` および `cargo fmt` の実行。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 3' (レジストリ操作とビルドの安定性確認)
- [x] Task: フェーズ成果物のコミット

## フェーズ 4: マイルストーン C - アーキテクチャ是正 (arch-auditor 指摘対応)
- **主な変更**: Resolver 層の Win32 隠蔽、Domain 層のプロトコル解析分離。
- [x] Task: `window_message_resolver.rs` 内の Win32 API 呼び出し（`BeginPaint`, `GetDC` 等）を `_gui_driver.rs` へカプセル化。
- [x] Task: `terminal_buffer_entity.rs` から `vte::Perform` 実装と解析ロジックを `terminal_protocol_handler.rs` へ隔離。
- [x] Task: コンパイル確認 (`cargo check`)。
- [x] Task: `@arch-auditor` による最終アーキテクチャ監査。
- [x] Task: `cargo clippy` および `cargo fmt` の実行。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 4' (リファクタリング後の全機能確認)
- [x] Task: フェーズ成果物のコミット

## フェーズ 5: 仕上げと最終検証 (Finalization & Verification)
- [x] Task: 実機（EmEditor）での全機能検証（描画、IME、ConPTY、設定、テーマ）。
- [x] Task: `cargo clippy` および `cargo fmt` の実行。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 5' (最終成果物の承認)
