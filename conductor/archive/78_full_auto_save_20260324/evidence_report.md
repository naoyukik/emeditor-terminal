# Evidence Report: Issue 78 - 完全な自動保存の実装

## 1. 調査対象: EmEditor SDK における終了時の保存処理

### 調査結果
- `WM_DESTROY` ハンドラ (`on_destroy`) で `reg_set_value` を呼び出し、設定を保存することは技術的に可能。
- `reg_set_value` (EE_REG_SET_VALUE: 2133) は `SendMessageW` を使用して EmEditor 本体にリクエストを送るため、`HWND` (hwnd_editor) が有効な間であれば動作する。
- `WM_DESTROY` はウィンドウが破棄される直前に送られるため、この時点での保存は妥当。

### 制約・注意点
- `on_destroy` の実行順序（他のリソース解放との兼ね合い）に注意する。
- ユーザーの意図した「設定変更時」にも保存を行うことで、万が一のクラッシュや予期せぬ終了時にも設定が失われないようにする。

## 2. 調査対象: Win32 MessageBox によるエラー通知

### 調査結果
- `MessageBoxW` (winuser.h) を使用して、モーダルなエラーダイアログを表示する。
- パラメータ:
    - `uType`: `MB_ICONERROR | MB_OK` (0x10 | 0x00) を使用し、エラーアイコンとOKボタンを表示する。
- EmEditor SDK の `EE_STATUS_BAR` (2138) も存在するが、仕様の「ダイアログを表示して通知」という要求に基づき、`MessageBoxW` を採用する。

## 3. 調査対象: 内部リポジトリ実装の確認

### 調査結果
- `EmEditorConfigRepositoryImpl` は既に `ConfigurationRepository::save` を実装済み。
- `src/infra/driver/emeditor_io_driver.rs` の `reg_set_value` をラップしている。
- 既存の実装は `hwnd_editor` が `is_null()` でないことを確認しており、安全性が考慮されている。

### 実装方針へのフィードバック
- `TerminalWorkflow` に `persist_config()` メソッドを追加し、`config_repo.save(&self.config)` を呼び出す。
- `ConfigWorkflow` の `save_config` の戻り値を `Result` に変更し、保存失敗を上位（GUI層）に伝播させる。
