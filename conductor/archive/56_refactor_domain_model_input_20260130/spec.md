# Track Specification: refactor/domain-model-separation-input

## Overview
AIエージェントによる開発効率向上と保守性の改善のため、`src/domain/input.rs` に混在しているドメインモデル（データ構造）とロジックを物理的に分離する。今回は「Inputモジュール」を対象とし、データ構造を `src/domain/model/` ディレクトリへ抽出する。

## Functional Requirements
- `src/domain/model/` ディレクトリを新規作成する。
- 以下のデータ構造を `src/domain/model/input.rs` に移動する。
    - `struct Modifiers`
    - `struct InputKey`
- `src/domain/input.rs` には以下のロジックを維持する。
    - `trait KeyTranslator`
    - `struct VtSequenceTranslator` とその実装
    - 関連する単体テスト (`mod tests`)
- プロジェクト全体（`src/` 配下）で、移動したモデルを参照している箇所（`use` 宣言など）を適切に修正する。

## Non-Functional Requirements
- **後方互換性**: 外部（Application層やInfra層）からの利用において、機能的な変更を伴わないこと。
- **テスタビリティ**: 既存のテストがすべてパスすること。
- **可読性**: ファイル分割により、各ファイルの責務が明確になり、AIが一度に読み込むコンテキスト量が削減されること。

## Acceptance Criteria
- [ ] `src/domain/model/input.rs` が存在し、`InputKey`, `Modifiers` が定義されている。
- [ ] `src/domain/input.rs` に変換ロジックとテストが残っている。
- [ ] `cargo build` が正常に完了する。
- [ ] `cargo test` を実行し、既存の入力変換テストがすべてパスする。

## Out of Scope
- `terminal.rs` や `parser.rs` など、Input以外のドメインモジュールの分離。
- `VtSequenceTranslator` の `src/domain/service/` への移動（今回は実施しない）。
