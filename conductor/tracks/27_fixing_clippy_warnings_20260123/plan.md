# Implementation Plan - fix: clippyによる警告（Lints）の修正

Issue #27 に基づき、`cargo clippy` の警告を修正し、コードの健全性を確保する。

## Phase 1: 警告内容の精査と環境準備
- [ ] Task: 現状の警告の再確認
    - [ ] `cargo clippy --all-targets -- -D warnings` を実行し、修正対象のファイルと場所をリストアップする
- [ ] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)

## Phase 2: 自動修正可能な項目の対応
- [ ] Task: 基本的な Lints の修正
    - [ ] `Needless return`, `Needless range loop`, `Single match` などの定型的な修正を行う
- [ ] Task: 配列・ベクタ操作の改善
    - [ ] `Manual slice fill`, `Same item push` を `fill()` や `extend` に置き換える
- [ ] Task: 算術演算とポインタチェックの修正
    - [ ] `saturating_sub` の導入および `is_null()` への置き換えを行う
- [ ] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)

## Phase 3: Unsafe および 未使用コードの対応
- [ ] Task: Unsafe ブロックの整理
    - [ ] 冗長な `unsafe` を削除し、安全性が保たれているか再確認する
- [ ] Task: 未使用コードの文脈分析と処置
    - [ ] `send_input` など将来利用するものは `#[allow(dead_code)]` を付与し、不要なものは削除する
- [ ] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)

## Phase 4: 最終検証と品質確認
- [ ] Task: 全テストの実行
    - [ ] `cargo test` を実行し、リグレッションがないことを確認する
- [ ] Task: 最終的な Clippy チェック
    - [ ] 警告が 0 件であることを確認する
- [ ] Task: EmEditor 実機動作確認
    - [ ] ビルドした DLL を EmEditor にロードし、基本機能（起動・入出力）をテストする
- [ ] Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)
