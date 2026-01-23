# Implementation Plan - Track: 15-refactor-architecture

## Phase 1: Preparation & Directory Setup
ディレクトリを作成し、単純なファイル移動と `mod` 定義の修正を行う。まずは「コンパイルが通る」状態を目指す。

- [ ] Task: ディレクトリ構造の作成
    - [ ] `src/domain`, `src/infra`, `src/gui` ディレクトリを作成する。
    - [ ] 各ディレクトリに `mod.rs` を作成する。
- [ ] Task: ファイルの移動
    - [ ] `src/terminal.rs` -> `src/domain/terminal.rs`
    - [ ] `src/conpty.rs` -> `src/infra/conpty.rs`
    - [ ] `src/editor.rs` -> `src/infra/editor.rs`
    - [ ] `src/custom_bar.rs` -> `src/gui/custom_bar.rs`
- [ ] Task: モジュール参照の修正
    - [ ] `lib.rs` の `mod` 宣言を新しい構造に合わせて修正する。
    - [ ] 各ファイルの `use crate::...` パスを修正する。
    - [ ] 可視性 (`pub`, `pub(crate)`) のエラーを修正し、`cargo check` を通す。
- [ ] Task: Conductor - User Manual Verification 'Preparation & Directory Setup' (Protocol in workflow.md)

## Phase 2: Logic Extraction (Renderer)
`custom_bar.rs` に残った描画ロジックを `renderer.rs` へ分離する。

- [ ] Task: `TerminalRenderer` の作成
    - [ ] `src/gui/renderer.rs` を作成する。
    - [ ] `custom_bar.rs` から `WM_PAINT` 内のロジック（`TerminalMetrics`, フォント生成, `ExtTextOutW` 等）を `renderer.rs` へコピー＆修正する。
    - [ ] `TerminalRenderer` 構造体を定義し、初期化メソッド (`new`) と描画メソッド (`render`) を実装する。
- [ ] Task: `custom_bar.rs` のリファクタリング
    - [ ] `custom_bar.rs` 内で `TerminalRenderer` を保持・利用するように変更する。
    - [ ] 古い描画コードを削除する。
- [ ] Task: 動作確認
    - [ ] `cargo build` を実行し、DLLを生成する。
    - [ ] EmEditorで動作を確認し、表示崩れがないかチェックする。
- [ ] Task: Conductor - User Manual Verification 'Logic Extraction (Renderer)' (Protocol in workflow.md)
