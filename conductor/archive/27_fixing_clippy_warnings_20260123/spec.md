# Specification: fix: clippyによる警告（Lints）の修正

## Overview
Issue #27 で報告された計28件の `cargo clippy` による警告（Lints）を修正し、コード品質を向上させる。
機能変更は行わず、リファクタリングの過程で発生した冗長な記述や、潜在的なバグの温床となる記述を修正する。

## Functional Requirements
- **警告の修正**: 以下のカテゴリに該当する警告を解消する。
  - `Unnecessary unsafe block`: 不要な `unsafe` ブロックの削除。
  - `Use of ptr != null_mut()`: `ptr.is_null()` への置き換え。
  - `Implicit saturating subtraction`: `saturating_sub` の使用。
  - `Needless return`: 不要な `return` の削除。
  - `Needless range loop`: イテレータへの変更。
  - `Single match`: `if let` への変更。
  - `Manual slice fill`: `slice.fill()` への変更。
  - `Same item push`: `extend` や `vec![]` 初期化への変更。

- **未使用コード (`Unused code`) の取り扱い**:
  - 文脈を分析し、将来的に使用される可能性が高いコード（例: `send_input`, `editor.rs` 内の定数など）は `#[allow(dead_code)]` を付与して維持する。
  - 明らかに不要、または古い実装の残骸と判断できるものは削除する。

## Non-Functional Requirements
- **安全性**: `unsafe` ブロックやポインタ操作の変更において、既存の動作（クラッシュしないこと、メモリ安全性の維持）を破壊しないこと。
- **可読性**: Rust のイディオムに従い、コードを簡潔化する。

## Acceptance Criteria
1. **Clippy 通過**: `cargo clippy` を実行し、警告 (Warnings) が0件になること。
2. **テスト通過**: 既存のユニットテスト (`cargo test`) がすべてパスすること。
3. **実機動作**: EmEditor 上でプラグインをロードし、以下の動作に異常がないこと（リグレッションテスト）。
   - プラグインの起動・終了。
   - ターミナルへの文字入力・出力。

## Out of Scope
- Issue #27 で指摘されていない新たな機能追加や、大規模なアーキテクチャ変更。
