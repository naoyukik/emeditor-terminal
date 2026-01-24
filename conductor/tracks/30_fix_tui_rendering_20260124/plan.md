# Implementation Plan: Fix TUI Rendering Issues (Microsoft Edit)

## Phase 1: Investigation & Diagnostic Logging
- [x] Task: 解析用の詳細ログ実装
    - [x] `src/infra/conpty.rs` または `src/domain/terminal.rs` のパーサー部分に、受信した全シーケンスをダンプする一時的なデバッグログを追加
- [x] Task: 再現確認とログ収集
    - [x] Microsoft Edit を起動し、表示崩れと `;128m` が発生する際のログを収集
    - [x] 収集したログから、未対応 of SGR コードや、行ズレの原因となるカーソル制御/スクロール領域設定シーケンスを特定
- [x] Task: 仕様確認 (Microsoft Learn)
    - [x] 特定されたシーケンス（特に SGR 128 や DECSTBM 等）について、Microsoft Learn の "Console Virtual Terminal Sequences" ドキュメントを参照し、正しい挙動を確認する
- [x] Task: Conductor - User Manual Verification 'Investigation & Diagnostic Logging' (Protocol in workflow.md)

## Phase 2: Parser Improvement
- [x] Task: SGR パーサーの堅牢化
    - [x] 未知の属性コード（128等）を検知した際に `WARN` ログを出力し、画面描画からはスキップする処理の実装
- [x] Task: カーソル/スクロール制御シーケンスの修正
    - [x] DECSTBM (Set Top and Bottom Margins) 等、TUI のレイアウトに影響する未実装または不完全なシーケンスの修正
- [x] Task: 構文チェック
    - [x] `cargo clippy` を実行し、変更箇所のコード品質を確認
- [x] Task: Conductor - User Manual Verification 'Parser Improvement' (Protocol in workflow.md)

## Phase 3: Layout & Resize Stability
- [x] Task: バッファ初期化とリサイズ処理の検証
    - [x] リサイズ時に ConPTY に送られる信号と、内部バッファの整合性を確認・修正
- [x] Task: 構文チェック
    - [x] `cargo clippy` を実行し、修正全体の整合性を確認
- [x] Task: Conductor - User Manual Verification 'Layout & Resize Stability' (Protocol in workflow.md)

## Phase 4: TUI Rendering Stabilization
- [x] Task: 問題1 & 2 & 3 の修正（メニューバー消失、1行目重複、全体ズレ）
    - [x] 初期サイズ同期の実装による改善効果を確認
    - [x] 必要であれば、バッファリセット処理やカーソル同期処理を追加 (Tab処理追加)
- [x] Task: 問題4 の修正（ゴミ文字・文字化け）
    - [x] 残存する文字化け（`隨`など）の原因特定と除去 (シーケンス分断対策実装)
- [x] Task: 構文チェック
    - [x] `cargo clippy` を実行
- [x] Task: Conductor - User Manual Verification 'TUI Stabilization' (Protocol in workflow.md)

## Phase 5: Verification
- [x] Task: 動作確認
    - [x] Microsoft Edit でメニューバーが表示され、行ズレがないことを確認
    - [x] ログを確認し、未知のコードが適切にトラップされているか確認
- [x] Task: Conductor - User Manual Verification 'Final Quality Check' (Protocol in workflow.md)
- [x] Task: 最終ビルドとクリーンアップ
    - [x] `cargo build` および最終的な `cargo clippy` のパスを確認
