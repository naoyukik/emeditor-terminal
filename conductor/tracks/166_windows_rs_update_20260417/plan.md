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
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 2' (基盤刷新の動作確認)
- [ ] Task: フェーズ成果物のコミット

## フェーズ 3: マイルストーン B - コード生成とリンク方式の標準化 (v0.62 -> v0.69)
- **主な変更**: `BOOLEAN` -> `bool` リマップ, `Ref`/`OutRef` 導入, `windows-link` 完全移行。
- [ ] Task: `Cargo.toml` を v0.69 相当へ更新。
- [ ] Task: `windows-targets` から `windows-link` への完全移行。
- [ ] Task: COM インターフェース引数の `Ref`/`OutRef` 対応。
- [ ] Task: レジストリ操作を `windows-registry` クレート (Safe API) へ移行開始。
- [ ] Task: コンパイル確認 (`cargo check`)。
- [ ] Task: `cargo clippy` および `cargo fmt` の実行。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 3' (標準化後の動作確認)
- [ ] Task: フェーズ成果物のコミット

## フェーズ 4: マイルストーン C - 最終アップデートと厳密化 (v0.69 -> v0.73)
- **主な変更**: ポインタ型の厳密化 (`*const`/`*mut`), `Display` トレイト削除への対応。
- [ ] Task: `Cargo.toml` を v0.73 (最新) へ更新。
- [ ] Task: Win32 API 呼び出し箇所のポインタ型整合の修正。
- [ ] Task: `log!` 等における `HSTRING`/`BSTR` の明示的な `.to_string()` 変換。
- [ ] Task: レジストリ操作の Safe API 移行を完了。
- [ ] Task: `cargo check` および全テスト実行。
- [ ] Task: `cargo clippy` および `cargo fmt` の実行。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 4' (最新版での全機能確認)
- [ ] Task: フェーズ成果物のコミット

## フェーズ 5: 仕上げと最終検証 (Finalization & Verification)
- [ ] Task: 実機（EmEditor）での全機能検証（描画、IME、ConPTY、設定、テーマ）。
- [ ] Task: `cargo clippy` および `cargo fmt` の実行。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 5' (最終成果物の承認)