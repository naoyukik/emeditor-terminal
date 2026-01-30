# Implementation Plan - refactor/domain-model-separation-input

## Phase 1: モデルの分離と定義
ディレクトリ構造を作成し、ドメインモデルを新しい場所に定義する。この段階ではまだ移行は行わず、新しい構造体が利用可能な状態にすることを目的とする。

- [x] Task: ディレクトリ作成
    - [x] `src/domain/model` ディレクトリを作成する。
    - [x] `src/domain/model.rs` (または `mod.rs`) を作成し、公開設定を行う。
- [x] Task: モデルの移動（コピー）
    - [x] `src/domain/input.rs` から `Modifiers`, `InputKey` の定義を `src/domain/model/input.rs` にコピーする。
    - [x] 新しいファイル (`model/input.rs`) がプロジェクトから参照できるよう `mod.rs` を調整する。
- [ ] Task: Conductor - User Manual Verification 'Phase 1: モデルの分離と定義' (Protocol in workflow.md)

## Phase 2: 参照の切り替えとクリーンアップ
既存のコードが新しいモデルを参照するように変更し、古い定義を削除する。コンパイルエラーを修正しながら進める。

- [ ] Task: 内部参照の修正 (`src/domain/input.rs`)
    - [ ] `src/domain/input.rs` 内での `InputKey`, `Modifiers` の使用箇所を、`crate::domain::model::input` (または同等のパス) を指すように `use` 宣言を変更する。
    - [ ] `src/domain/input.rs` 内の古い `InputKey`, `Modifiers` 定義を削除する。
- [ ] Task: 外部参照の修正 (プロジェクト全体)
    - [ ] `grep` 等で `domain::input::InputKey` や `domain::input::Modifiers` を参照している箇所を特定する。
    - [ ] 参照先を `domain::model::input::...` へ一括置換または修正する。
    - [ ] `cargo check` を実行し、コンパイルエラーを解消する。
- [ ] Task: テスト実行による検証
    - [ ] `cargo test` を実行し、リファクタリングによってロジックが壊れていないことを確認する。
- [ ] Task: Conductor - User Manual Verification 'Phase 2: 参照の切り替えとクリーンアップ' (Protocol in workflow.md)
