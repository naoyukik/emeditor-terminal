# Implementation Plan

## フェーズ 1: 問題の把握と詳細設計 (Discovery & Detailed Design)
- [x] Task: `autonomous-researcher` による詳細調査と `evidence_report.md` の作成
- [x] Task: 調査結果に基づいた `plan.md` の以降のタスクの具体化。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 1' (調査結果と具体的プランの承認)

## フェーズ 2: マイルストーン A - 基盤刷新 (v0.58 -> v0.62)
- [x] Task: `Cargo.toml` を v0.62 相当へ更新。
- [x] Task: インポートパスの修正。
- [x] Task: `HSTRING` の `Deref` 実装に伴うコード簡略化。
- [x] Task: コンパイル確認 (`cargo check`) とユニットテスト。
- [x] Task: `cargo clippy` および `cargo fmt` の実行。
- [x] Task: `@arch-auditor` によるレビューを実施。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 2' (基盤刷新の動作確認)
- [x] Task: フェーズ成果物のコミット

## フェーズ 3: マイルストーン B - Safe API 刷新と Windows-Link 完全移行
- [x] Task: `windows-targets` への依存を排除し、`windows-link` へ移行。
- [x] Task: レジストリ操作を `windows-registry` クレートへ移行。
- [x] Task: コンパイル確認 (`cargo check`)。
- [x] Task: `@arch-auditor` によるレビューを実施。
- [x] Task: `cargo clippy` および `cargo fmt` の実行。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 3' (レジストリ操作とビルドの安定性確認)
- [x] Task: フェーズ成果物のコミット

## フェーズ 4: マイルストーン C - アーキテクチャ是正 (arch-auditor 指摘対応)
- [x] Task: `window_message_resolver.rs` 内の Win32 API 呼び出しを Driver へカプセル化。
- [x] Task: `terminal_buffer_entity.rs` から `vte::Perform` 実装を分離・分割。
- [x] Task: コンパイル確認 (`cargo check`)。
- [x] Task: `@arch-auditor` による最終アーキテクチャ監査。
- [x] Task: `cargo clippy` および `cargo fmt` の実行。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 4' (リファクタリング後の全機能確認)
- [x] Task: フェーズ成果物のコミット

## フェーズ 5: 仕上げと最終検証 (Finalization & Verification)
- [x] Task: 実機（EmEditor）での全機能検証（描画、IME、ConPTY、設定、テーマ）。
- [x] Task: `cargo clippy` および `cargo fmt` の実行。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 5' (最終成果物の承認)

## フェーズ 6: PRレビュー対応 (PR Review Fixes)
- [x] Task: `WM_DESTROY` ハンドラでのキーボードフック解除とリソース解放の追加。
- [x] Task: `AnsiParserDomainService::parse` のバイト処理効率化とフラッシュ漏れ修正.
- [x] Task: 削除されたドメインサービスのユニットテストの復元と調整。
- [x] Task: ドキュメントおよび PR 説明文のバージョン表記を 0.62 に統一。
- [x] Task: 未使用の `WindowGuiDriver::cleanup_terminal()` の整理。
- [x] Task: 不要な重複ファイル `terminal_protocol_handler_domain_service.rs` の削除。
- [x] Task: `lib.rs`, `conpty_repository_impl.rs`, `conpty_io_driver.rs`, `gui/window/mod.rs` への `// SAFETY:` コメント追加。
