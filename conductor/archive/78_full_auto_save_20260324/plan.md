# Implementation Plan: Issue 78 - 完全な自動保存の実装

## Phase 0: 外部・内部調査 (Research Phase)
- [x] Task: EmEditor SDK 調査：`on_destroy` (WM_DESTROY) 時の `reg_set_value` 呼び出しの有効性、プラグイン終了プロトコルの確認
- [x] Task: Microsoft Learn 調査：Win32 API `MessageBox` および EmEditor 独自のメッセージハンドリング手法のベストプラクティスを調査
- [x] Task: 内部調査：既存 spinning `EmEditorConfigRepositoryImpl` の `save` 実装に潜在的なバグや制約（書き込み権限等）がないか再確認
- [x] Task: 成果物：調査結果に基づき `evidence_report.md` を作成する
- [x] Task: フィードバック：調査結果を本 `plan.md` または `spec.md` に反映する
- [ ] Task: Conductor - User Manual Verification 'Phase 0' (Protocol in workflow.md)
- [ ] Task: Commit progress: `chore(conductor): Complete Research Phase 0`

## Phase 1: Application 層の保存ロジック強化とテスト
- [x] Task: `ConfigWorkflow` の保存メソッドの戻り値を `Result<(), ConfigError>` に変更し、エラー情報を定義する
- [x] Task: `TerminalWorkflow` に `persist_config` を追加し、現在のメモリ上の状態を保存できるようにする
- [x] Task: ユニットテストを追加し、`ConfigWorkflow` が `ConfigurationRepository` の `save` を正しく呼び出すことを検証する
- [ ] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)
- [ ] Task: Commit progress: `feat(config): Implement robust save logic in Application layer`

## Phase 2: GUI 層での保存トリガーの最適化と通知
- [x] Task: 設定ダイアログの「OK」ボタン押下時に保存処理 (`save_config`) を実行するように実装する
- [x] Task: 終了時の「先祖返り」バグを防ぐため、`on_destroy` での冗長な保存処理を排除（または実装しない）することを決定・実施する
- [x] Task: 保存失敗時に EmEditor の `MessageBox` を表示するエラー通知ロジックを実装する
- [ ] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)
- [ ] Task: Commit progress: `feat(gui): Optimize save trigger and fix revert bug`

## Phase 3: 結合テストと実機検証
- [x] Task: EmEditor 実機でのテスト：設定変更後の再起動で設定が復元されることを確認する
- [x] Task: EmEditor 実機でのテスト：プラグイン終了後に設定が永続化されていることを確認する（INI/レジストリの直接確認）
- [x] Task: 異常系テスト：保存失敗時のダイアログ表示をシミュレートし、正しく表示されることを確認する
- [x] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)
- [x] Task: Commit progress: `test: Verify configuration persistence on real machine`

## Phase 4: PR レビュー対応
- [x] Task: 仕様書 (`spec.md`) およびメタデータ (`metadata.json`) の説明文を「即時保存」の方針に統一する
- [x] Task: `config_resolver.rs` の MessageBox の親ウィンドウを `parent_hwnd` に修正する
- [x] Task: 未使用の `persist_config` および `ConfigError::LoadFailed` に `#[allow(dead_code)]` を付与し、警告を抑制する
- [x] Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)
- [x] Task: Commit progress: `fix(review): Address PR review comments and documentation consistency`
