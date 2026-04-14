# Implementation Plan: Fix Issue 135 (Terminal text overflow at startup)

## Phase 1: 問題の把握と詳細設計 (Discovery & Detailed Design)
- [x] Task: `autonomous-researcher` スキルによる詳細調査と `evidence_report.md` の作成
    - [x] Sub-task: ターミナル生成時（`WM_CREATE` や ConPTY 初期化周辺）での画面サイズ（クライアント領域）取得ロジックと文字幅（フォントメトリクス）取得の同期タイミングを調査する。
    - [x] Sub-task: 推奨案（起動時サイズの厳密化）の実装可能性を検証し、具体的な変更箇所（ファイル名と行数）を特定する。
    - [x] Sub-task: 代替案（サイズ確定まで遅延）の実現性についても、影響範囲（UIのちらつき等）を評価し特定しておく。
- [x] Task: 調査結果に基づいた `plan.md` の以降のタスクの具体化
    - [x] Sub-task: `evidence_report.md` の結果をもとに、修正するソースコードとテストケースを決定し、フェーズ2のタスクリストを書き換える。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 1' (調査結果と具体的プランの承認)

## Phase 2: 推奨案の試行 (Attempt Primary Approach - Superseded)
- [x] Task: `TerminalGuiDriver` のメトリクス先行更新メソッドの実装 a7f481c
- [x] Task: `open_custom_bar` におけるサイズ計算の厳密化 a7f481c
    - *結果: 下部のはみ出しは改善したが、右端のはみ出しが解消せず。EmEditor による配置確定を待つ方式へ移行。*

## Phase 3: 採用案の実装と検証 (Implement Adopted Approach - Option B)
- [x] Task: サイズ確定（WM_SIZE）までのターミナル初期化遅延ロジックの実装 420f8d6
    - [x] Sub-task: ウィンドウの `WM_SIZE` メッセージを受信してからターミナル内部グリッドを初期化し、描画を開始するよう処理フローを変更。
- [x] Task: 静的解析とフォーマットの実行 420f8d6
- [x] Task: 修正内容のコミット 420f8d6
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 3' (Issue 135 および起動直後のはみ出し修正の確認)

## Phase: Review Fixes
- [x] Task: Apply review suggestions 6c4055f
- [ ] Task: PR #142 レビュー指摘に基づく堅牢性と性能の改善
    - [ ] Sub-task: `update_metrics` において Win32 API の成功を確認し、不正なメトリクス（0等）によるゼロ除算を防止する。
    - [ ] Sub-task: `on_size` 内で `GetDC` の戻り値をチェックし、ConPTY 起動失敗時のウィンドウ破棄・状態リセットを実装する。
    - [ ] Sub-task: `render_internal` での `update_metrics` 呼び出し頻度を制限（未取得時または設定変更時のみ）し、ログレベルを調整する。
    - [ ] Sub-task: `spec.md` および `plan.md` を最終的な実装（Option B 採用）に合わせて更新する。
