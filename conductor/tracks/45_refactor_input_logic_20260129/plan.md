# Track 45_refactor_input_logic Implementation Plan

## Phase 1: Domain Logic Extraction & Testing
**必ずPhaseの最後にユーザーの手動確認を依頼すること。依頼しなかった場合は切腹を言い渡す。**
このフェーズでは、外部APIに依存しない純粋なロジック部分を抽出・実装し、テストで固める。

- [ ] Task: 1.1 `src/domain/input.rs` モジュールの作成
    - [ ] `domain/mod.rs` に `input` モジュールを定義
    - [ ] `domain/input.rs` ファイルを作成
- [ ] Task: 1.2 `KeyTranslator` トレイトの定義
    - [ ] 変換ロジックのインターフェースを定義 (入力: キー情報, 出力: Option<String>)
- [ ] Task: 1.3 `vk_to_vt_sequence` ロジックの移動と適応
    - [ ] `custom_bar.rs` からロジックをコピーし、Win32 API依存（グローバル定数等）を引数で受け取るようにリファクタリング
    - [ ] `VtSequenceConverter` 構造体の実装
- [ ] Task: 1.4 ユニットテストの実装
    - [ ] 通常キーの変換テスト
    - [ ] 制御文字（Ctrl+Key）の変換テスト
    - [ ] 特殊キー（矢印キー、Home/End等）の変換テスト
    - [ ] 無視すべきキーのテスト
- [ ] **Task: Conductor - User Manual Verification 'Domain Logic' (Protocol in workflow.md)**
    - [ ] `cargo test` の実行と結果確認。

## Phase 2: Infra Layer Extraction (Keyboard Hook)
**必ずPhaseの最後にユーザーの手動確認を依頼すること。依頼しなかった場合は切腹を言い渡す。**
このフェーズでは、Win32 APIを直接扱うフック処理を `Infra` 層へ移動し、メッセージパッシング機構を整備する。

- [ ] Task: 2.1 `src/infra/input.rs` モジュールの作成
    - [ ] `infra/mod.rs` に `input` モジュールを定義
    - [ ] `infra/input.rs` ファイルを作成
- [ ] Task: 2.2 `KeyboardHook` 構造体の実装
    - [ ] `new(target_hwnd: HWND)`: ターゲットウィンドウを持つインスタンス生成
    - [ ] `install(&mut self)`: フックの登録処理
    - [ ] `uninstall(&mut self)`: フックの解除処理
    - [ ] `Drop` 実装: 自動的なアンフック
- [ ] Task: 2.3 フックプロシージャの移行とメッセージ送信実装
    - [ ] `custom_bar.rs` からプロシージャを移動
    - [ ] `PostMessage` を使用したターゲットウィンドウへの通知実装
    - [ ] 必要な Win32 定数/型のインポート整理
- [ ] Task: 2.4 ビルド検証 (Infra)
    - [ ] 単体でのビルドが通ることを確認
- [ ] **Task: Conductor - User Manual Verification 'Infra Layer' (Protocol in workflow.md)**

## Phase 3: Integration & Cleanup
**必ずPhaseの最後にユーザーの手動確認を依頼すること。依頼しなかった場合は切腹を言い渡す。**
GUI層 (`CustomBar`) を修正し、新しい Domain/Infra モジュールを利用するように変更する。

- [ ] Task: 3.1 `CustomBar` への `KeyboardHook` 組み込み
    - [ ] 構造体のフィールドに `KeyboardHook` を追加
    - [ ] 初期化時にフックをインストール
- [ ] Task: 3.2 メッセージ受信処理の実装 (`wnd_proc`)
    - [ ] `WM_USER` (または定義したメッセージ) のハンドリング追加
    - [ ] 受信したメッセージから `VtSequenceConverter` を呼び出し、ターミナルへ送信
- [ ] Task: 3.3 古いコードの削除
    - [ ] `custom_bar.rs` 内の古い `vk_to_vt_sequence` を削除
    - [ ] `custom_bar.rs` 内の古いフックプロシージャを削除
    - [ ] 不要になった import の削除
- [ ] Task: 3.4 動作確認 (Manual Test)
    - [ ] プラグインをビルド・インストール
    - [ ] EmEditor上でターミナルを起動し、キー入力が正常に反映されるか確認
- [ ] **Task: Conductor - User Manual Verification 'Integration' (Protocol in workflow.md)**
