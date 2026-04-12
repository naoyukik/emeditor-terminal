# Implementation Plan: Fix Issue 135 (Terminal text overflow at startup)

## Phase 1: 問題の把握と詳細設計 (Discovery & Detailed Design)
- [ ] Task: `autonomous-researcher` スキルによる詳細調査と `evidence_report.md` の作成
    - [ ] Sub-task: ターミナル生成時（`WM_CREATE` や ConPTY 初期化周辺）での画面サイズ（クライアント領域）取得ロジックと文字幅（フォントメトリクス）取得の同期タイミングを調査する。
    - [ ] Sub-task: 推奨案（起動時サイズの厳密化）の実装可能性を検証し、具体的な変更箇所（ファイル名と行数）を特定する。
    - [ ] Sub-task: 代替案（サイズ確定まで遅延）の実現性についても、影響範囲（UIのちらつき等）を評価し特定しておく。
- [ ] Task: 調査結果に基づいた `plan.md` の以降のタスクの具体化
    - [ ] Sub-task: `evidence_report.md` の結果をもとに、修正するソースコードとテストケースを決定し、フェーズ2のタスクリストを書き換える。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 1' (調査結果と具体的プランの承認)

## Phase 2: 推奨案の実装と検証 (Implement Primary Approach)
- [ ] Task: `TerminalGuiDriver` のメトリクス先行更新メソッドの実装
    - [ ] Sub-task: `src/gui/driver/terminal_gui_driver.rs` に `update_metrics(&mut self, hdc: HDC, config: &TerminalConfig)` メソッドを追加し、`get_font_for_style` 内のメトリクス計算ロジックを共通化する。
- [ ] Task: `open_custom_bar` におけるサイズ計算の厳密化
    - [ ] Sub-task: `src/gui/window/mod.rs` の `open_custom_bar` 内で、`EE_CUSTOM_BAR_OPEN` 送信後に `GetClientRect` を呼び出し、実際の物理サイズを取得する。
    - [ ] Sub-task: `renderer.update_metrics(hdc, &config)` を呼び出し、起動時のフォント設定に基づいた正確な文字幅を取得する。
    - [ ] Sub-task: 正確なピクセルサイズと文字幅から `initial_cols`, `initial_rows` を算出し、ConPTY 起動引数として渡す。
- [ ] Task: 静的解析とフォーマットの実行
    - [ ] Sub-task: `cargo clippy` と `cargo fmt` を実行し、警告を解消する。
- [ ] Task: 修正内容のコミット
    - [ ] Sub-task: `AGENTS.md` の規約に従い `fix(gui): 起動時のフォントメトリクス取得とサイズ計算の厳密化` 等でコミットする。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 2' (Issue 135 および起動直後のはみ出し修正の確認。※もし推奨案で解消しない場合は、フェーズ3の代替案へ移行する判断を仰ぐ)

## Phase 3: 代替案の実装と検証 (Fallback Approach - ※フェーズ2で解決した場合はスキップ)
- [ ] Task: サイズ確定（WM_SIZE）までのターミナル初期化遅延ロジックの実装
    - [ ] Sub-task: ウィンドウの `WM_SIZE` メッセージを受信してからターミナル内部グリッドを初期化し、描画を開始するよう処理フローを変更。
- [ ] Task: 静的解析とフォーマットの実行
    - [ ] Sub-task: `cargo clippy` と `cargo fmt` を実行し、警告を解消する。
- [ ] Task: 修正内容のコミット
    - [ ] Sub-task: `AGENTS.md` の規約に従いコミットする。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 3' (Issue 135 および起動直後のはみ出し修正の確認)

## Phase: Review Fixes
- [x] Task: Apply review suggestions 6c4055f
