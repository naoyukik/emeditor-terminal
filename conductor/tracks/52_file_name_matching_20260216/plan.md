# Implementation Plan - Track 52: ファイル名と構造体名の一致 (Strict Rigid版)

## Phase 1: 物理構造の監査とマッピング
- [ ] Task: プロジェクト全域の監査。構造体名とファイル名が一致していない箇所、および `mod.rs` 内にロジックが残っている箇所をリストアップする。
- [ ] Task: 構造体名の一致を優先する順序を確定させ、依存関係のループが発生しないよう変更順序を計画する。
- [ ] Task: 変更内容のコミット
- [ ] Task: Conductor - User Manual Verification 'Phase 1: マッピング承認' (Protocol in workflow.md)

## Phase 2: Domain層の物理的隔離 (Inside-Out)
- [ ] Task: `src/domain/` 配下の構造体を `_entity.rs`, `_value.rs`, `_repository.rs` 等へ分割・リネームする。
- [ ] Task: `windows` クレートへの依存が Domain 層に残っていないか確認し、あれば Pure Rust 型へ置換する。
- [ ] Task: 変更内容のコミット
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Domain層の統制' (Protocol in workflow.md)

## Phase 3: Application層の物理的隔離
- [ ] Task: `TerminalService` を `terminal_service_workflow.rs` へリネームし、DTO（Input/Result）を別ファイルへ抽出する。
- [ ] Task: 依存方向が Application -> Domain であることを `use` 宣言から確認する。
- [ ] Task: 変更内容のコミット
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Application層の統制' (Protocol in workflow.md)

## Phase 4: Infrastructure層の物理的隔離
- [ ] Task: `conpty.rs`, `editor.rs` 等を `_io_driver.rs` へ、リポジトリ実装を `_repository_impl.rs` へリネーム・再編する。
- [ ] Task: Win32 API の操作が `_io_driver.rs` 内に完全に封印されていることを確認する。
- [ ] Task: 変更内容のコミット
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Infrastructure層の統制' (Protocol in workflow.md)

## Phase 5: Presentation / GUI層の再編 (Final Frontier)
- [ ] Task: `wnd_proc` 周辺を `_resolver.rs` へ、描画・IME・スクロールを `_gui_driver.rs` へ、メッセージ型を `_request.rs` / `_response.rs` へ再編する。
- [ ] Task: `gui/window/mod.rs` を純粋化し、ウィンドウライフサイクルのみを管理する構成にする。
- [ ] Task: 変更内容のコミット
- [ ] Task: Conductor - User Manual Verification 'Phase 5: Presentation層の統制' (Protocol in workflow.md)

## Phase 6: 全体統合と掟の検証
- [ ] Task: プロジェクト全体の `mod.rs` および `lib.rs` を最終調整し、コンパイルエラーを解消する。
- [ ] Task: 全テストの実行と `cargo clippy` による最終監査。
- [ ] Task: 最終成果物のコミット
- [ ] Task: Conductor - User Manual Verification 'Phase 6: 最終確認' (Protocol in workflow.md)
