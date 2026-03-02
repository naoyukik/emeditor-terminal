# Implementation Plan: Issue #74 - 設定基盤の構築 (EmEditor Config API)

## Phase 1: 設定エンティティとインターフェースの定義 (Entity & Repository Interface)
**目的:** 設定データ構造と、それを取得するための Repository トレイトを定義する。

- [x] Task: `TerminalConfig` 構造体の定義。`font_face`, `font_size`, `shell_path` を含める。
- [x] Task: `ConfigRepository` トレイトの定義。`load()` と `save()` メソッドを備える。
- [x] Task: Commit Phase 1 - `feat: 設定エンティティとインターフェースの定義`
- [ ] Task: Conductor - User Manual Verification 'Phase 1 - 設定構造の定義' (Protocol in workflow.md)

## Phase 2: インフラ層の実装 (Infrastructure Implementation)
**目的:** EmEditor SDK のメッセージを使用して設定を読み書きする Repository の実装。

- [x] Task: `EmEditorConfigRepositoryImpl` の実装。`EE_REG_QUERY_VALUE` / `EE_REG_SET_VALUE` メッセージを使用する。
- [x] Task: デフォルト値（フォールバック）の解決ロジックの実装。
- [ ] Task: テストコード（Mock を用いた Config 取得テスト）の作成。
- [x] Task: Commit Phase 2 - `feat: EmEditor標準APIによる設定読み書きの実装`
- [ ] Task: Conductor - User Manual Verification 'Phase 2 - インフラ層の読み書き' (Protocol in workflow.md)

## Phase 3: アプリケーション層への統合 (Application Integration)
**目的:** 起動時に設定をロードし、サービスへ注入する。

- [ ] Task: `TerminalWorkflow` および `TerminalService` のコンストラクタで `ConfigRepository` を受け取るように修正。
- [ ] Task: `TerminalWorkflow` の初期化プロセスで `config_repo.load()` を実行し、設定を適用する。
- [ ] Task: 起動時の設定ロードが正しく行われていることをログで確認。
- [ ] Task: Commit Phase 3 - `feat: 設定基盤のアプリケーション層への統合`
- [ ] Task: Conductor - User Manual Verification 'Phase 3 - 設定の適用と最終確認' (Protocol in workflow.md)
