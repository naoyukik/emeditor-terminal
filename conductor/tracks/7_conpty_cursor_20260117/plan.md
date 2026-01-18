# Implementation Plan: ConPty Cursor Rendering

## Phase 1: 基礎データ構造とパースロジックの実装
- [x] Task: カーソル状態を管理するための構造体定義と初期化
    - [x] `terminal.rs` (または関連ファイル) に `Cursor` 構造体を追加
    - [x] `Terminal` 構造体に `cursor` フィールドを統合
- [x] Task: Cursor Position (CUP) シーケンスのパース実装
    - [x] `ESC [ <y> ; <x> H` を解析し、論理座標を更新するロジックを実装
    - [x] 座標変換（1-based から 0-based 等）の整合性を確認
    - [x] ユニットテストの作成（正常系・異常系）
- [x] Task: Cursor Visibility (DECTCEM) シーケンスのパース実装
    - [x] `ESC [ ? 25 h` / `l` を解析し、表示フラグを更新するロジックを実装
    - [x] ユニットテストの作成
- [ ] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)

## Phase 2: 座標変換とレンダリング基盤の整備
- [ ] Task: 論理座標からピクセル座標への変換ロジックの実装
    - [ ] フォントサイズ・パディング・スクロールオフセットを考慮した計算式を実装
    - [ ] `Terminal` に座標取得用ヘルパーメソッドを追加
    - [ ] ユニットテストの作成
- [ ] Task: 描画コンポーネントへのカーソル描画処理の追加
    - [ ] "Bar" スタイルの矩形描画ロジックを実装（オーバーレイ方式）
    - [ ] カーソルの表示フラグ (`cursor_visible`) に応じた条件分岐を追加
- [ ] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)

## Phase 3: 結合テストと動作確認
- [ ] Task: 実際の ConPty 出力を用いた結合テスト
    - [ ] `ping` などのコマンド実行時にカーソル位置が正しく追従することを確認
    - [ ] ウィンドウサイズ変更時のカーソル位置追従の確認
- [ ] Task: パフォーマンス確認と最適化（必要に応じて）
    - [ ] カーソル移動時の再描画負荷の確認
- [ ] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)