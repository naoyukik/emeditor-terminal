# Specification: Refactor Layered Architecture Boundaries

## Overview
本トラックは、[Issue #49](https://github.com/naoyukik/emeditor-terminal/issues/49) に基づき、レイヤードアーキテクチャの境界を強化するためのリファクタリングを実施する。
特に、**Domain層**および**Application層**において、`windows` クレート（Win32 API）への直接的な依存や型の漏洩を排除・制限することを目的とする。

## Functional Requirements
1.  **Application Layer (`src/application/service.rs`)**:
    -   `TerminalService` のパブリックインターフェース（`pub fn`）が、`windows` クレートの型（例: `HWND`, `RECT` 等）を引数や戻り値として公開していないことを確認し、維持する。
    -   内部実装で必要な場合でも、その依存を最小限に留め、可能な限り隠蔽する。

2.  **Domain Layer (`src/domain/`)**:
    -   `windows` クレートへの依存が一切ない状態（Pure Rust）を維持する。
    -   `src/domain/input.rs` 等で Win32 API の定数（VKコード等）が必要な場合は、`windows` クレートをインポートせず、独自に定数定義を行う方針を継続する。
    -   その意図（依存排除）をコードコメントとして明記する。

3.  **Visibility Control**:
    -   各モジュールの `pub` 宣言を見直し、モジュール外に公開する必要のない構造体や関数は `pub(crate)` に変更してカプセル化を強化する。

## Non-Functional Requirements
-   **No Logic Change**: 既存の機能や振る舞いを変更しないこと。純粋な構造上のリファクタリングである。
-   **Test Passing**: `cargo test` が全て通過すること。
-   **Build Success**: `cargo build` が警告なし（または既存の警告のみ）で成功すること。

## Acceptance Criteria
-   `src/domain/` ディレクトリ内で `use windows::...` が存在しないこと。
-   `src/application/service.rs` のパブリックメソッドが `windows` クレートの型を使用していないこと。
-   `src/domain/input.rs` の定数定義に、`windows` クレート非依存の意図を示すコメントが追加されていること。
-   既存のユニットテストが全て成功すること。

## Out of Scope
-   `src/infra/` や `src/gui/` のリファクタリング（これらは `windows` クレートへの依存が許容される）。
-   入力ロジック自体の変更（Issueコメントにある「Altキー問題」を引き起こすようなロジック変更は行わない）。
