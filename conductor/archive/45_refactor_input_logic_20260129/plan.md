# Track 45_refactor_input_logic Implementation Plan

## Phase 1: Domain Logic Extraction & Testing
**必ずPhaseの最後にユーザーの手動確認を依頼すること。依頼しなかった場合は切腹を言い渡す。**
このフェーズでは、外部APIに依存しない純粋なロジック部分を抽出・実装し、テストで固める。

- [x] Task: 1.1 `src/domain/input.rs` モジュールの作成
    - [x] `domain/mod.rs` に `input` モジュールを定義
    - [x] `domain/input.rs` ファイルを作成
- [x] Task: 1.2 `KeyTranslator` トレイトの定義
    - [x] 変換ロジックのインターフェースを定義 (入力: キー情報, 出力: Option<String>)
- [x] Task: 1.3 `vk_to_vt_sequence` ロジックの移動と適応
    - [x] `custom_bar.rs` からロジックをコピーし、Win32 API依存（グローバル定数等）を引数で受け取るようにリファクタリング
    - [x] `VtSequenceConverter` 構造体の実装
- [x] Task: 1.4 ユニットテストの実装
    - [x] 通常キーの変換テスト
    - [x] 制御文字（Ctrl+Key）の変換テスト
    - [x] 特殊キー（矢印キー、Home/End等）の変換テスト
    - [x] 無視すべきキーのテスト
- [x] **Task: Conductor - User Manual Verification 'Domain Logic' (Protocol in workflow.md)**
    - [x] `cargo test` の実行と結果確認。

## Phase 2: Infra Layer Extraction (Keyboard Hook)
**必ずPhaseの最後にユーザーの手動確認を依頼すること。依頼しなかった場合は切腹を言い渡す。**
このフェーズでは、Win32 APIを直接扱うフック処理を `Infra` 層へ移動し、メッセージパッシング機構を整備する。

- [x] Task: 2.1 `src/infra/input.rs` モジュールの作成
    - [x] `infra/mod.rs` に `input` モジュールを定義
    - [x] `infra/input.rs` ファイルを作成
- [x] Task: 2.2 `KeyboardHook` 構造体の実装
    - [x] `new(target_hwnd: HWND)`: ターゲットウィンドウを持つインスタンス生成
    - [x] `install(&mut self)`: フックの登録処理
    - [x] `uninstall(&mut self)`: フックの解除処理
    - [x] `Drop` 実装: 自動的なアンフック
- [x] Task: 2.3 フックプロシージャの移行とメッセージ送信実装
    - [x] `custom_bar.rs` からプロシージャを移動
    - [x] `PostMessage` を使用したターゲットウィンドウへの通知実装
    - [x] 必要な Win32 定数/型のインポート整理
- [x] Task: 2.4 ビルド検証 (Infra)
    - [x] 単体でのビルドが通ることを確認
- [x] **Task: Conductor - User Manual Verification 'Infra Layer' (Protocol in workflow.md)**

## Phase 3: Domain Model Refinement & Integration
**必ずPhaseの最後にユーザーの手動確認を依頼すること。依頼しなかった場合は切腹を言い渡す。**
GUI層 (`CustomBar`) を修正し、新しい Domain/Infra モジュールを利用するように変更する。その前に、ドメインモデルを導入し、より堅牢な設計にする。

- [x] Task: 3.1 ドメインモデルの導入 (`InputKey` struct)
    - [x] `src/domain/input.rs` に `InputKey` 構造体と `Modifiers` 構造体を定義
    - [x] `KeyTranslator` トレイトのシグネチャ変更
    - [x] `VtSequenceTranslator` の実装修正
    - [x] ユニットテストの修正
- [x] Task: 3.2 `CustomBar` への `KeyboardHook` 組み込み
    - [x] 構造体のフィールドに `KeyboardHook` を追加
    - [x] 初期化時にフックをインストール
- [x] Task: 3.3 メッセージ受信処理の実装 (`wnd_proc`)
    - [x] `WM_USER` (または定義したメッセージ) のハンドリング追加
    - [x] 受信したメッセージから `VtSequenceConverter` を呼び出し、ターミナルへ送信
- [x] Task: 3.4 古いコードの削除
    - [x] `custom_bar.rs` 内の古い `vk_to_vt_sequence` を削除
    - [x] `custom_bar.rs` 内の古いフックプロシージャを削除
    - [x] 不要になった import の削除
- [x] Task: 3.5 動作確認 (Manual Test)
    - [x] プラグインをビルド・インストール
    - [x] EmEditor上でターミナルを起動し、キー入力が正常に反映されるか確認
- [x] **Task: Conductor - User Manual Verification 'Integration' (Protocol in workflow.md)**
