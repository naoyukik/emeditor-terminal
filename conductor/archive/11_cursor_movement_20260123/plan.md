# 実装計画: 追加のカーソル移動シーケンスの実装

## フェーズ 1: 開発環境の準備と既存コードの理解
- [x] Task: 既存の `handle_csi` および `display_col_to_char_index` の動作確認
    - [x] `src/domain/terminal.rs` の該当箇所を再読し、パースロジックを確認する
- [x] Task: テストスイートの準備
    - [x] `src/domain/terminal.rs` の `tests` モジュールに、現在のカーソル移動機能が壊れていないか確認する既存テストの実行
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 1' (Protocol in workflow.md)

## フェーズ 2: 拡張シーケンスの実装 (CHA, VPA)
- [x] Task: `CHA` (Cursor Horizontal Absolute) の実装
    - [x] `handle_csi` に 'G' コマンドのハンドラを追加
    - [x] 境界クランプ処理の実装
    - [x] ユニットテストの追加と実行 (feat: CHAの実装)
- [x] Task: `VPA` (Vertical Line Position Absolute) の実装
    - [x] `handle_csi` に 'd' コマンドのハンドラを追加
    - [x] 既存の表示カラムを維持するための再計算ロジックの実装
    - [x] ユニットテストの追加と実行 (feat: VPAの実装)
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 2' (Protocol in workflow.md)

## フェーズ 3: 行間移動シーケンスの実装 (CNL, CPL)
- [x] Task: `CNL` (Cursor Next Line) の実装
    - [x] `handle_csi` に 'E' コマンドのハンドラを追加
    - [x] 下方向への移動と X=0 へのリセット処理の実装
    - [x] ユニットテストの追加と実行 (feat: CNLの実装)
- [x] Task: `CPL` (Cursor Previous Line) の実装
    - [x] `handle_csi` に 'F' コマンドのハンドラを追加
    - [x] 上方向への移動と X=0 へのリセット処理の実装
    - [x] ユニットテストの追加と実行 (feat: CPLの実装)
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 3' (Protocol in workflow.md)

## フェーズ 4: 統合テストと最終調整
- [x] Task: 境界条件と全角文字混在の総合テスト
    - [x] 画面端での挙動、全角文字の泣き別れ防止などのテストケース追加
    - [x] 全テストのパス確認
- [x] Task: コードのリファクタリングとドキュメント更新
    - [x] 冗長なロジックの整理
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 4' (Protocol in workflow.md)
