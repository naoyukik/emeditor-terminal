# Implementation Plan - Track: 15-refactor-architecture

## Phase 1: Preparation & Directory Setup
ディレクトリを作成し、単純なファイル移動と `mod` 定義の修正を行う。まずは「コンパイルが通る」状態を目指す。

- [x] Task: ディレクトリ構造の作成
    - [x] `src/domain`, `src/infra`, `src/gui` ディレクトリを作成する。
    - [x] 各ディレクトリに `mod.rs` を作成する。
- [x] Task: ファイルの移動
    - [x] `src/terminal.rs` -> `src/domain/terminal.rs`
    - [x] `src/conpty.rs` -> `src/infra/conpty.rs`
    - [x] `src/editor.rs` -> `src/infra/editor.rs`
    - [x] `src/custom_bar.rs` -> `src/gui/custom_bar.rs`
- [x] Task: モジュール参照の修正
    - [x] `lib.rs` の `mod` 宣言を新しい構造に合わせて修正する。
    - [x] 各ファイルの `use crate::...` パスを修正する。
    - [x] 可視性 (`pub`, `pub(crate)`) のエラーを修正し、`cargo check` を通す。
- [ ] Task: Conductor - User Manual Verification 'Preparation & Directory Setup' (Protocol in workflow.md)

## Phase 2: Logic Extraction (Renderer)
`custom_bar.rs` に残った描画ロジックを `renderer.rs` へ分離する。

- [x] Task: `TerminalRenderer` の作成
    - [x] `src/gui/renderer.rs` を作成する。
    - [x] `custom_bar.rs` から `WM_PAINT` 内のロジック（`TerminalMetrics`, フォント生成, `ExtTextOutW` 等）を `renderer.rs` へコピー＆修正する。
    - [x] `TerminalRenderer` 構造体を定義し、初期化メソッド (`new`) と描画メソッド (`render`) を実装する。
- [x] Task: `custom_bar.rs` のリファクタリング
    - [x] `custom_bar.rs` 内で `TerminalRenderer` を保持・利用するように変更する。
    - [x] 古い描画コードを削除する。
- [x] Task: 動作確認
    - [x] `cargo build` を実行し、DLLを生成する。
    - [x] EmEditorで動作を確認し、表示崩れがないかチェックする。
- [ ] Task: Conductor - User Manual Verification 'Logic Extraction (Renderer)' (Protocol in workflow.md)

## Phase 3: Review Fixes & Stabilization
GitHub PR #28 のレビュー指摘事項のうち、リファクタリング起因の問題および安全性に関わる修正を行う。

- [x] Task: Critical/Safety Fixes
    - [x] `src/gui/renderer.rs`: `CreatePen` の戻り値 (NULL) チェックを追加し、安全性を確保する。
    - [x] `src/gui/renderer.rs`: `CreateFontW` の戻り値 (NULL) チェックを追加し、安全性を確保する。
    - [x] `conductor/tracks.md`: ファイル先頭の BOM (U+FEFF) を削除する。
- [x] Task: Documentation Recovery
    - [x] `src/gui/renderer.rs`: `SendHFONT` の `unsafe impl` に関する詳細な安全性ドキュメントを復元する。
