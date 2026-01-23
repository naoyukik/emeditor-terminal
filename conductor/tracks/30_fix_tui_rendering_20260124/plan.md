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
- [ ] Task: SGR パーサーの堅牢化
    - [ ] 未知の属性コード（128等）を検知した際に `WARN` ログを出力し、画面描画からはスキップする処理の実装
- [ ] Task: カーソル/スクロール制御シーケンスの修正
    - [ ] DECSTBM (Set Top and Bottom Margins) 等、TUI のレイアウトに影響する未実装または不完全なシーケンスの修正
- [ ] Task: 構文チェック
    - [ ] `cargo clippy` を実行し、変更箇所のコード品質を確認
- [ ] Task: Conductor - User Manual Verification 'Parser Improvement' (Protocol in workflow.md)

## Phase 3: Layout & Resize Stability
- [ ] Task: バッファ初期化とリサイズ処理の検証
    - [ ] リサイズ時に ConPTY に送られる信号と、内部バッファの整合性を確認・修正
- [ ] Task: 構文チェック
    - [ ] `cargo clippy` を実行し、修正全体の整合性を確認
- [ ] Task: Conductor - User Manual Verification 'Layout & Resize Stability' (Protocol in workflow.md)

## Phase 4: Verification
- [ ] Task: 動作確認
    - [ ] Microsoft Edit でメニューバーが表示され、行ズレがないことを確認
    - [ ] ログを確認し、未知のコードが適切にトラップされているか確認
- [ ] Task: Conductor - User Manual Verification 'Final Quality Check' (Protocol in workflow.md)
- [ ] Task: 最終ビルドとクリーンアップ
    - [ ] `cargo build` および最終的な `cargo clippy` のパスを確認
