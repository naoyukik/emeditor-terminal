# Evidence Report: Issue 202 - get_selected_text の選択判定効率化

## 調査結果

### 現行実装の課題 (`src/domain/model/terminal_buffer_entity.rs`)

`get_selected_text()` において、以下の二重ループと条件判定が行われている。

```rust
for logical_row in start.logical_row..=end.logical_row {
    if let Some(line) = self.get_line_at_logical_row(logical_row) {
        for x in 0..self.width {
            if selection_contains(self.selection_range, x, logical_row)
                && let Some(cell) = line.get(x)
                // ...
```

- `selection_contains()` は内部で `normalized_selection_range()` を呼び出している。
- `x` ごとに `selection_contains` が呼ばれるため、計算コストが無駄である。
- 特に大規模なバッファや長い選択範囲において、性能低下の要因となる。

### 改善方針

既に `normalized_selection_range()` によって正規化済みの `start` / `end` が得られているため、各 `logical_row` における `x` の範囲（`start_x` 〜 `end_x`）を直接算出できる。

- 開始行 (`logical_row == start.logical_row`): `start.x` から開始。
- 中間行 (`start.logical_row < logical_row < end.logical_row`): `0` から開始し、`self.width - 1` まで。
- 終了行 (`logical_row == end.logical_row`): `end.x` まで。
- 単一行選択 (`start.logical_row == end.logical_row`): `start.x` から `end.x` まで。

### EmEditor SDK 関連調査

`sdk/` 内を確認したが、本プラグインは独自の `TerminalBufferEntity` でテキストを管理しているため、EmEditor 本体の選択 API (`Editor_GetSelTextW` 等) を直接利用してバッファから抽出することはできない。本件は純粋なドメイン層のロジック改善である。

### Microsoft Learn / Windows Terminal 調査

Windows Terminal の選択挙動（マウス、キーボード）に関するドキュメントを確認。
今回の変更は抽出ロジックの最適化であり、選択範囲の定義や結果として得られる文字列の意味（ワイド文字の扱い、改行コード）を変更しないため、互換性の問題はない。

## 結論

- `selection_contains` のセル単位呼び出しを排除し、行ごとの範囲を直接算出する。
- 既存のユニットテストでデグレードがないことを確認する。
- 新たにエッジケース（単一行、複数行、ワイド文字）のテストを強化する。

## 検証結果

### ユニットテスト結果
`cargo test` により、以下の全 10 件の `terminal_buffer_entity` 関連テストを含む 55 件すべてのテストがパスすることを確認した。

- `test_get_selected_text_basic`: 単一行・複数行の抽出が正しい。
- `test_get_selected_text_wide_char`: ワイド文字（継続セルスキップ）の抽出が正しい。
- `test_get_selected_text_empty`: 空範囲で空文字を返す。
- `test_get_selected_text_reverse`: 逆向き選択でも正しく正規化して抽出される。

### 性能改善の確認
実装コードにおいて、セル走査ループ内の `selection_contains()` 呼び出しが完全に削除されていることを目視確認した。行ごとの `start_x`, `end_x` 算出は $O(1)$ であり、選択範囲外のセル（特に中間行の大部分）を走査する必要がなくなったため、大規模バッファでの性能が論理的に向上している。
