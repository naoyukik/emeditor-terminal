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
- [ ] Task: 起動時の正確なサイズ算出ロジックの実装
    - [ ] Sub-task: フォント初期化とウィンドウサイズ取得を ConPTY プロセス起動前に確実に実行するよう処理順序を整理する。
    - [ ] Sub-task: 論理カラム数・行数の計算処理を修正し、初期の物理サイズに完全に一致させる。
- [ ] Task: 静的解析とフォーマットの実行
    - [ ] Sub-task: `cargo clippy` と `cargo fmt` を実行し、警告を解消する。
- [ ] Task: 修正内容のコミット
    - [ ] Sub-task: `AGENTS.md` の規約に従い `fix` または `refactor` プレフィックスでコミットする。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 2' (Issue 135 および起動直後のはみ出し修正の確認。※もし推奨案で解消しない場合は、フェーズ3の代替案へ移行する判断を仰ぐ)

## Phase 3: 代替案の実装と検証 (Fallback Approach - ※フェーズ2で解決した場合はスキップ)
- [ ] Task: サイズ確定（WM_SIZE）までのターミナル初期化遅延ロジックの実装
    - [ ] Sub-task: ウィンドウの `WM_SIZE` メッセージを受信してからターミナル内部グリッドを初期化し、描画を開始するよう処理フローを変更。
- [ ] Task: 静的解析とフォーマットの実行
    - [ ] Sub-task: `cargo clippy` と `cargo fmt` を実行し、警告を解消する。
- [ ] Task: 修正内容のコミット
    - [ ] Sub-task: `AGENTS.md` の規約に従いコミットする。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 3' (Issue 135 および起動直後のはみ出し修正の確認)