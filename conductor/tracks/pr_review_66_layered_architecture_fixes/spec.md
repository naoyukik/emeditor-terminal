# Specification: PR #66 Review Fixes (Layered Architecture)

## Overview
本トラックは、[PR #66](https://github.com/naoyukik/emeditor-terminal/pull/66) (Issue #49) に対して実施されたコードレビュー（セカンドオピニオン）のうち、今回の変更に直接関連する指摘事項に対応する。
具体的には、`parser` モジュールの可視性不整合の解消と、警告抑制箇所への意図明記を行う。

## Functional Requirements
1.  **Domain Layer Visibility (`src/domain/`)**:
    -   `mod.rs`: `parser` モジュールの公開範囲を `pub` から `pub(crate)` に変更し、内部の `AnsiParser` の可視性（`pub(crate)`）と整合させる。

2.  **Code Documentation (`src/application/`)**:
    -   `service.rs`: 今回 `#[allow(dead_code)]` を付与した箇所に対し、将来的な使用予定（フォント設定UI等）を明記した `TODO` コメントを追加する。

## Non-Functional Requirements
-   既存のテストが全て通過すること。
-   `cargo check` が警告なしで通過すること。

## Acceptance Criteria
-   `src/domain/mod.rs` で `pub(crate) mod parser;` となっていること。
-   `src/application/service.rs` の `#[allow(dead_code)]` 箇所にコメントが付与されていること。

## Out of Scope
-   `src/application/mod.rs` や `src/domain/input.rs` の可視性変更（今回のPRの変更範囲外のため）。
