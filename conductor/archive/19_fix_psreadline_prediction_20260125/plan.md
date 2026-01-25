# 実装計画: fix: PSReadLineの予測入力（Prediction）表示の改善 (Issue #19)

## フェーズ 1: ドメイン層の拡張と SGR パースの土台構築
- [x] Task: `Cell` 構造体に前景色属性を追加
    - [x] `src/domain/terminal.rs` の `Cell` 構造体に `fg_color` フィールドを追加
    - [x] デフォルト色の定義と初期化ロジックの実装
- [x] Task: SGR (Select Graphic Rendition) パースロジックの実装
    - [x] `ESC [ <n> m` を解析する関数を実装または拡張
    - [x] `30-37`, `38`, `39` の各モードを識別し、バッファの状態を更新するロジックを実装
- [x] Task: ユニットテストの作成
    - [x] SGR シーケンス入力後に `Cell` の属性が正しく更新されることを確認するテストを記述
- [x] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)

## フェーズ 2: インフラ層・GUI 層への反映
- [x] Task: `TerminalRenderer` の描画ロジック修正
    - [x] `src/gui/renderer.rs` において、描画前にセルの `fg_color` を取得
    - [x] `SetTextColor` を用いて、指定された色で文字を描画するよう変更
- [x] Task: カラーマップの実装
    - [x] ANSI 8色/16色を GDI の `COLORREF` (RGB) に変換するテーブルの実装
- [ ] Task: 動作確認用の簡易テスト
    - [ ] 模擬的な SGR シーケンスを流し込み、描画色が変化することを確認
- [x] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)

## フェーズ 3: 最終検証とクリーンアップ
- [x] Task: PSReadLine 予測入力の実機検証
- [x] Task: Clippy および Rustfmt によるコード品質確認
- [x] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)