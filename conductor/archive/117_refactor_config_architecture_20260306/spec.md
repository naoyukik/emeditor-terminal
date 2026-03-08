# Specification - Track 117: 設定ダイアログにおけるアーキテクチャ違反の修正

## 1. Overview
Issue #110 の実装で発生した、GUI Driver (`src/gui/driver/config_gui_driver.rs`) から Infrastructure 層 (`EmEditorConfigRepositoryImpl`) への直接依存を排除し、プロジェクトの「Strict Rigid レイヤードアーキテクチャ」を再確立する。

## 2. Functional Requirements
- **共通型の移動と依存整理**:
    - `SendHWND` や設定 DTO を `src/common/` または `src/domain/model/` に配置し、Presentation 層（`resolver`）からの逆行参照を解消する。
- **Application 層の新設**:
    - `ConfigWorkflow` を `src/application/` に新設。
    - 設定のロード・保存ロジック（ビジネスロジック）を `ConfigWorkflow` に集約する。
- **依存関係の注入 (DI)**:
    - `ConfigWorkflow` は `TerminalConfigRepository` トレイトを受け取るようにする。
    - `config_gui_driver.rs` は `ConfigWorkflow` を通じて設定を操作し、具象クラス（`EmEditorConfigRepositoryImpl`）への依存を完全に排除する。
- **Driver の責務分離**:
    - `config_gui_driver.rs` は純粋な Win32 UI 操作（コントロールの更新、値の取得）のみを担当する。
- **自動建築チェックの導入**:
    - 最後に `validate_dependency_architecture.py` フックを正式に導入し、依存関係の違反を自動検知する体制を構築する。

## 3. Non-Functional Requirements
- **Strict Layered Architecture**: 全ての依存方向は外側（GUI/Infra）から内側（Domain）に向かうこと。
- **Pure Domain**: Domain 層は `windows` クレートに依存しないこと。

## 4. Acceptance Criteria
- [ ] `config_gui_driver.rs` 内に `EmEditorConfigRepositoryImpl` のインポートや直接インスタンス化が存在しないこと。
- [ ] 設定ダイアログの挙動（フォント変更、保存、ロード）が従来通り正常に動作すること。
- [ ] `cargo build` および `cargo clippy` が警告なしで通ること。
- [ ] `validate_dependency_architecture.py` によるチェックをパスすること。

## 5. Out of Scope
- 設定ダイアログの UI デザイン（レイアウト、文言）の変更。
- 設定項目（フォント名、サイズ以外）の追加。
- `TerminalWorkflow` の既存ロジックの大規模なリファクタリング（設定関連以外）。
