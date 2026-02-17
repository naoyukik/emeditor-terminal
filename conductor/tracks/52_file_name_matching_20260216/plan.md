# Implementation Plan - Track 52: ファイル名と構造体名の一致 (Strict Rigid版)

## Phase 1: 物理構造の監査とマッピング
- [x] Task: プロジェクト全域の監査。構造体名とファイル名が一致していない箇所、および `mod.rs` 内にロジックが残っている箇所をリストアップする。
- [x] Task: 構造体名の一致を優先する順序を確定させ、依存関係のループが発生しないよう変更順序を計画する。

### 監査・マッピング結果 (Phase 1)

※`mod.rs` の解体およびロジック抽出は Issue #68 へ切り出されたため、本トラックでは既存の独立したファイルのリネームのみを行う。

| 現在のファイル | 含まれる構造体・型 | 新ファイル名 (サフィックスルール) | レイヤー |
| :--- | :--- | :--- | :--- |
| `src/domain/parser.rs` | `AnsiParser` | `src/domain/service/ansi_parser_domain_service.rs` | Domain Service |
| `src/application/service.rs` | `TerminalWorkflow` | `src/application/terminal_workflow.rs` | Application (Workflow) |
| `src/infra/conpty.rs` | `ConptyIoDriver` | `src/infra/driver/conpty_io_driver.rs` | Infrastructure (IO Driver) |
| `src/infra/editor.rs` | `CUSTOM_BAR_INFO` | `src/infra/driver/emeditor_io_driver.rs` | Infrastructure (IO Driver) |
| `src/infra/input.rs` | `KeyboardIoDriver` | `src/infra/driver/keyboard_io_driver.rs` | Infrastructure (IO Driver) |
| `src/gui/renderer.rs` | `TerminalGuiDriver` | `src/gui/driver/terminal_gui_driver.rs` | GUI (GUI Driver) |
| `src/gui/scroll.rs` | `ScrollGuiDriver` | `src/gui/driver/scroll_gui_driver.rs` | GUI (GUI Driver) |
| `src/gui/terminal_data.rs` | `TerminalWindowResolver` | `src/gui/resolver/terminal_window_resolver.rs` | GUI (Resolver) |
| `src/gui/window/handlers.rs`| (wnd_proc logic) | `src/gui/resolver/window_message_resolver.rs` | GUI (Resolver) |

※ `TerminalColor`, `Cell`, `TerminalBufferEntity` 等の細かい Value Object / Entity の抽出も Issue #68 に含める。

- [x] Task: 変更内容のコミット
- [ ] Task: Conductor - User Manual Verification 'Phase 1: マッピング承認' (Protocol in workflow.md)

## Phase 2: Domain層の物理的隔離 (Inside-Out)
- [x] Task: `src/domain/` 配下の構造体を `_entity.rs`, `_value.rs`, `_repository.rs` 等へ分割・リネームする。
- [x] Task: `windows` クレートへの依存が Domain 層に残っていないか確認し、あれば Pure Rust 型へ置換する。
- [x] Task: 変更内容のコミット
- [x] Task: Conductor - User Manual Verification 'Phase 2: Domain層の統制' (Protocol in workflow.md)

## Phase 3: Application層の物理的隔離
- [x] Task: `TerminalWorkflow` を `terminal_workflow.rs` へリネームし、DTO（Input/Result）を別ファイルへ抽出する。
- [x] Task: 依存方向が Application -> Domain であることを `use` 宣言から確認する。
- [x] Task: 変更内容のコミット
- [x] Task: Conductor - User Manual Verification 'Phase 3: Application層の統制' (Protocol in workflow.md)

## Phase 4: Infrastructure層の物理的隔離
- [x] Task: `conpty.rs`, `editor.rs` 等を `_io_driver.rs` へ、リポジトリ実装を `_repository_impl.rs` へリネーム・再編する。
- [x] Task: Win32 API の操作が `_io_driver.rs` 内に完全に封印されていることを確認する。
- [x] Task: 変更内容のコミット
- [x] Task: Conductor - User Manual Verification 'Phase 4: Infrastructure層の統制' (Protocol in workflow.md)

## Phase 5: Presentation / GUI層の再編 (Final Frontier)
- [x] Task: `wnd_proc` 周辺を `_resolver.rs` へ、描画・IME・スクロールを `_gui_driver.rs` へ、メッセージ型を `_request.rs` / `_response.rs` へ再編する。
- [x] Task: `gui/window/mod.rs` を純粋化し、ウィンドウライフサイクルのみを管理する構成にする。
- [x] Task: 変更内容のコミット
- [x] Task: Conductor - User Manual Verification 'Phase 5: Presentation層の統制' (Protocol in workflow.md)

## Phase 6: 全体統合と掟の検証
- [x] Task: プロジェクト全体の `mod.rs` および `lib.rs` を最終調整し、コンパイルエラーを解消する。
- [x] Task: 全テストの実行と `cargo clippy` による最終監査。
- [x] Task: 最終成果物のコミット
- [ ] Task: Conductor - User Manual Verification 'Phase 6: 最終確認' (Protocol in workflow.md)
