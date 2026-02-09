# Implementation Plan: Repositoryパターンと依存性注入(DI)の導入

## Phase 1: Domain層の定義
Domain層において、Infrastructure層の実装に依存しないRepositoryインターフェースを定義する。

- [x] Task: Repository定義用のモジュール構成の作成
    - [x] `src/domain/repository/mod.rs` の作成
- [x] Task: `TerminalOutputRepository` トレイトの定義
    - [x] `src/domain/repository/terminal_output_repository.rs` を作成
    - [x] `send_input`, `resize` メソッドを定義
- [x] Task: `ConfigurationRepository` トレイトの定義
    - [x] `src/domain/repository/configuration_repository.rs` を作成
    - [x] 必要な設定取得メソッドを定義
- [x] Task: Conductor - User Manual Verification 'Phase 1: Domain層の定義' (Protocol in workflow.md)

## Phase 2: Infrastructure層の実装
Domain層で定義したトレイトを、Windows APIや既存の `ConPTY` を用いて実装する。

- [x] Task: `ConptyRepositoryImpl` の実装
    - [x] `src/infra/repository/conpty_repository_impl.rs` を作成
    - [x] `ConPTY` 構造体をラップして `TerminalOutputRepository` を実装
- [x] Task: `EmEditorConfigRepositoryImpl` の実装
    - [x] `src/infra/repository/emeditor_config_repository_impl.rs` を作成
    - [x] EmEditor SDKを介して設定を取得する `ConfigurationRepository` を実装
- [x] Task: Infrastructure層のテスト
    - [x] 実装が正しいシグネチャを持っていることを確認
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Infrastructure層の実装' (Protocol in workflow.md)

## Phase 3: Application層のリファクタリング (DI導入)
`TerminalService` を具体的な実装から切り離し、DIを受け入れるように変更する。

- [x] Task: `TerminalService` の構造体変更
    - [x] `output_repo`, `config_repo` をメンバに追加
- [x] Task: `TerminalService::new` のシグネチャ変更
    - [x] リポジトリを引数で受け取るように変更（Constructor Injection）
- [x] Task: 既存メソッドの修正
    - [x] `send_input`, `resize` 等をリポジトリ経由の呼び出しに変更
- [x] Task: キャッシュロジックの実装
    - [x] 生成時に `config_repo` から設定を取得して保持する
- [x] Task: Conductor - User Manual Verification 'Phase 3: Application層のリファクタリング (DI導入)' (Protocol in workflow.md)

## Phase 4: 統合と動作検証
GUI層での初期化処理を修正し、全体の動作を確認する。

- [x] Task: `gui/custom_bar.rs` の修正
    - [x] `TerminalService` 生成時に具象リポジトリを注入するように変更
- [x] Task: ビルドと基本的な動作確認
    - [x] 文字入力、リサイズが正しく動作することを確認
- [x] Task: ユニットテストの追加
    - [x] Mockリポジトリを用いた `TerminalService` のテストを追加
- [x] Task: Conductor - User Manual Verification 'Phase 4: 統合と動作検証' (Protocol in workflow.md)
