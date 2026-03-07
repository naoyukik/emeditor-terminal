# Implementation Plan - Track 117: 設定ダイアログにおけるアーキテクチャ違反の修正

## Phase 1: Preparation and Common Refactoring
- [x] Task: 型の整理と依存関係の解消
    - [x] `SendHWND` や設定値 DTO を `src/common/` または `src/domain/model/` に配置する。
    - [x] Presentation 層 (`resolver`) からの逆行参照を解消する。
- [x] Task: Code Quality Checks
    - [x] `cargo fmt` を実行する。
    - [x] `cargo clippy --fix --allow-dirty` を実行する。
- [x] Task: Conductor - User Manual Verification 'Preparation' (Protocol in workflow.md)
- [x] Task: **Commit Phase 1 Changes**
    - [x] `chore(conductor): Complete Phase 1 - Preparation and Common Refactoring` でコミット。

## Phase 2: Application Layer (ConfigWorkflow) Implementation
- [x] Task: ConfigWorkflow の新設と TDD 開発
    - [x] `src/application/config_workflow.rs` を作成し、テストケースを記述する。
    - [x] 設定のロード・保存ロジック（ビジネスロジック）を実装する。
    - [x] `TerminalConfigRepository` トレイトとの DI 構成を確立する。
- [x] Task: Code Quality Checks
    - [x] `cargo fmt` を実行する。
    - [x] `cargo clippy --fix --allow-dirty` を実行する。
- [x] Task: Conductor - User Manual Verification 'Application Layer' (Protocol in workflow.md)
- [x] Task: **Commit Phase 2 Changes**
    - [x] `feat(application): Implement ConfigWorkflow and DI logic` でコミット。

## Phase 3: GUI Driver Refactoring
- [ ] Task: ConfigGuiDriver のリファクタリング
    - [ ] `config_gui_driver.rs` から具象クラスへの依存を完全に排除する。
    - [ ] `ConfigWorkflow` をコンストラクタで受け取るように変更する。
    - [ ] Driver は純粋な Win32 UI 操作（値のセット、コントロールからの値取得）のみを行う。
- [ ] Task: Code Quality Checks
    - [ ] `cargo fmt` を実行する。
    - [ ] `cargo clippy --fix --allow-dirty` を実行する。
- [ ] Task: Conductor - User Manual Verification 'GUI Driver' (Protocol in workflow.md)
- [ ] Task: **Commit Phase 3 Changes**
    - [ ] `refactor(gui): Decouple ConfigGuiDriver from Infrastructure layer` でコミット。

## Phase 4: Integration and Final Polish
- [ ] Task: 全体の統合テスト
    - [ ] 設定ダイアログを開き、値を変更して保存・ロードが正しく動作することを確認する。
- [ ] Task: 自動建築フックの正式導入
    - [ ] `validate_dependency_architecture.py` フックを Gemini CLI hooks として導入する。
- [ ] Task: Code Quality Checks
    - [ ] `cargo fmt` を実行する。
    - [ ] `cargo clippy --fix --allow-dirty` を実行する。
- [ ] Task: Conductor - User Manual Verification 'Integration' (Protocol in workflow.md)
- [ ] Task: **Commit Phase 4 Changes**
    - [ ] `chore(conductor): Add validate_dependency_architecture hook and final cleanup` でコミット。
