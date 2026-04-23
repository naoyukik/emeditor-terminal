# Implementation Plan: Rust Edition 2024 Migration (Issue #156)

## フェーズ 1: 問題の把握と詳細設計 (Discovery & Detailed Design)
- [x] Task: 公式の「Rust Edition 2024 Migration Guide」を調査・熟読し、主要な変更点と移行手順を把握する
- [x] Task: `autonomous-researcher` による詳細調査と `evidence_report.md` の作成 (Edition 2024 更新時のエラー・警告の特定)
- [x] Task: 調査結果に基づいた `plan.md` の以降のタスク（具体的なコード修正箇所）の具体化
- [x] Task: Conductor - User Manual Verification 'フェーズ 1: 問題の把握と詳細設計' (Protocol in workflow.md)

## フェーズ 2: 既存の警告解消と必須依存の更新 (Resolve Warnings and Dependencies)
- [x] Task: `src/gui/common/mod.rs` 内の `pixels_to_points`, `points_to_pixels` 関連の `unsafe fn` ボディを `unsafe {}` ブロック化し、Safety Comment を付与する
- [x] Task: `src/gui/driver/config_gui_driver.rs` 内の `update_font_label`, `settings_dlg_proc` 等の `unsafe fn` ボディを `unsafe {}` ブロック化し、Safety Comment を付与する
- [x] Task: エディション移行に必須な最小限のクレート（依存関係）の更新 (`cargo update`)
- [x] Task: 依存更新・警告解消後の `cargo check` および `cargo clippy` の通過確認
- [x] Task: 修正内容のコミット（AGENTS.md コミット規約の遵守）
- [x] Task: Conductor - User Manual Verification 'フェーズ 2: 既存の警告解消と必須依存の更新' (Protocol in workflow.md)

## フェーズ 3: Edition 2024 への移行と最終回帰テスト (Edition Update and Validation)
- [x] Task: `Cargo.toml` の `edition = "2024"` への変更
- [x] Task: `src/gui/driver/scroll_gui_driver.rs:19` の `extern` ブロックを `unsafe extern` に変更
- [x] Task: `src/lib.rs` 内の `#[no_mangle]` 属性を `#[unsafe(no_mangle)]` に変更
- [x] Task: `cargo build` および `cargo clippy` の実行による最終エラー・警告の解消
- [x] Task: 実機テスト: プラグインの即時起動、初期化処理、自動フォーカスの確認
- [x] Task: 実機テスト: 日本語 IME 候補ウィンドウの正確な位置同期と描画の確認
- [x] Task: 実機テスト: ターミナルの文字描画、GDI リソースの安定性維持の確認
- [x] Task: エディション 2024 更新のコミット（AGENTS.md コミット規約の遵守）
- [x] Task: Conductor - User Manual Verification 'フェーズ 3: Edition 2024 への移行と最終回帰テスト' (Protocol in workflow.md)