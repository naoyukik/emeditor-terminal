# Implementation Plan - PR #40 Review Fixes

## Phase 1: 安全性とバグ修正 (Critical/Bug)
無限ループとクラッシュの危険性がある修正を優先して行う。

- [x] Task: SGRパースの無限ループ対策 (`src/domain/terminal.rs`)
    - [x] `handle_sgr` 内の `38`, `48` の分岐で、不正なパラメータの場合も確実に `i` をインクリメントするように修正。
    - [x] テストケースに不正なシーケンスを追加して検証。
- [x] Task: フォント生成失敗時のフォールバック (`src/gui/renderer.rs`)
    - [x] `get_font_for_style` で `CreateFontIndirectW` が失敗した場合、スタイルなし（マスク0）のフォント取得を試みるロジックを追加。
    - [x] それでも失敗する場合のログ出力を強化。
- [x] Task: Dim属性のロジック改善 (`src/gui/renderer.rs`)
    - [x] RGB分岐内だけでなく、すべての `TerminalColor` を `COLORREF` に変換した後で Dim 計算を行うようにリファクタリング。
- [x] Task: Conductor - User Manual Verification 'Phase 1: Fixes'

## Phase 2: ドキュメントとクリーンアップ (Docs/Refactor)
コードの品質と可読性を向上させる。

- [x] Task: ドキュメントの復元と修正
    - [x] `src/gui/renderer.rs` の \`SendHFONT\` に Safety ドキュメントを追記。
    - [x] `src/domain/terminal.rs` にコロン区切りパースのコメントを追加。
    - [x] `README.md` の文言修正。
- [x] Task: Conductor - User Manual Verification 'Phase 2: Docs'
