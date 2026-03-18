# Specification: ThemeType 変換ロジックの集約 (Issue #129)

## 概要
現在、`ThemeType` とインデックス（整数値）や表示名との相互変換ロジックが、`ThemeType` 自身のメソッド、`EmEditorConfigRepositoryImpl`、および `config_gui_driver.rs` に分散・重複している。これを `ThemeType` に集約し、将来的なテーマ追加に対する保守性と拡張性を向上させる。

## 機能要件
- **変換ロジックの集約**:
    - `i32` (レジストリ/INI保存値) から `ThemeType` への変換。
    - `ThemeType` から `i32` への変換。
    - コンボボックス等のUI表示名へのマッピング（`One Half Dark` 等の文字列）。
- **分散箇所の排除**:
    - `EmEditorConfigRepositoryImpl` 内の `match` による手動変換を `ThemeType::from_index` / `to_index` 等に置換。
    - `config_gui_driver.rs` 内の ComboBox インデックス処理を共通メソッドに置換。
- **拡張性の向上**:
    - 将来的な新しいテーマ（例：Issue #128）の追加が、`ThemeType` の定義更新のみで完結するような構造（Traitパターンや静的配列のマッピング等）を検討する。

## 非機能要件
- **後方互換性**:
    - 既存の設定値（0=System Default, 1=Dark, 2=Light）との互換性を完全に維持すること。
- **パフォーマンス**:
    - 頻繁に呼び出される変換ロジックではないため、可読性と保守性を最優先とする。

## 受け入れ条件 (Acceptance Criteria)
- [ ] `ThemeType` 以外の場所にあるハードコードされた `match` や `if` による変換ロジックが排除されていること。
- [ ] 設定ダイアログでのテーマ切り替え、および永続化後の復元が正しく動作すること。
- [ ] ユニットテストによって、全ての `ThemeType` が正しくインデックスおよび表示名と相互変換できることが確認されていること。

## 範囲外 (Out of Scope)
- 新しいカラーテーマの追加自体（Issue #128）。
- JSON ファイルからの動的テーマロード機能。
