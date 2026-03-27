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

## Phase 2: GUI 層での保存トリガーのフックと通知
- [x] Task: `ConfigGuiDriver` または `ConfigWorkflow` を通じて、設定ダイアログの「OK」ボタン押下時に保存処理を実行するように実装する
- [x] Task: `TerminalWindow` の `WM_DESTROY` ハンドラ（`on_destroy`）に `ConfigWorkflow::save_config` の呼び出しを追加する
- [x] Task: 保存失敗時に EmEditor の `MessageBox` を表示するエラー通知ロジックを `Infrastructure` 層または `GUI` 層に実装する
- [ ] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)
- [ ] Task: Commit progress: `feat(gui): Hook save triggers and implement error notification`

## Phase 3: 結合テストと実機検証
- [ ] Task: EmEditor 実機でのテスト：設定変更後の再起動で設定が復元されることを確認する
- [ ] Task: EmEditor 実機でのテスト：プラグイン終了後に設定が永続化されていることを確認する（INI/レジストリの直接確認）
- [ ] Task: 異常系テスト：保存失敗時のダイアログ表示をシミュレートし、正しく表示されることを確認する
- [ ] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)
- [ ] Task: Commit progress: `test: Verify configuration persistence on real machine`
