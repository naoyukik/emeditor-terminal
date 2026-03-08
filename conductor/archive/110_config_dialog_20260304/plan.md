# Implementation Plan - Issue #110: プラグイン設定ダイアログの実装

## Phase 1: Resource Setup & SDK Message Integration
EmEditor SDK の `EP_QUERY_PROPERTIES` および `EP_SET_PROPERTIES` メッセージを処理し、ダイアログ表示のトリガーを実装する。また、リソーススクリプト (`.rc`) をプロジェクトに導入する。

- [x] Task: プロジェクトへのリソーススクリプト (`emeditor-terminal.rc`) の導入
    - [x] `build.rs` を更新し、`embed-resource` 等を用いてリソースをコンパイル対象に含める。
    - [x] 最小限の設定ダイアログ（OK/Cancel/Static Text）を定義する。
- [x] Task: EmEditor SDK メッセージの受信処理の実装 (`lib.rs`)
    - [x] `EP_QUERY_PROPERTIES` に対して `TRUE` を返す。
    - [x] `EP_SET_PROPERTIES` 受信時に `MessageBoxW` 等で動作を確認する。
- [x] Task: Conductor - ユーザー手動検証 'Phase 1' (Protocol in workflow.md)

## Phase 2: Configuration Dialog Implementation
リソースダイアログを実際に表示し、Win32 ダイアログプロシージャを通じてコントロールを制御する。

- [x] Task: 設定ダイアログ表示ロジックの実装
    - [x] `DialogBoxParamW` を用いて、リソースベースのダイアログを表示する。
    - [x] ダイアログプロシージャを実装し、初期化時に現在の設定をロードする。
- [x] Task: `ChooseFontW` ダイアログの統合
    - [x] 「Change Font」ボタン押下時に標準フォント選択ダイアログを表示する。
    - [x] 選択された結果を親ダイアログ上のテキスト（例: "Current Font: MS Gothic, 12pt"）に反映させる。
- [x] Task: Conductor - ユーザー手動検証 'Phase 2' (Protocol in workflow.md)

## Phase 3: Persistence & Verification
ダイアログで設定された値を保存し、次回の起動時に正しく反映されることを確認する。

- [x] Task: `ConfigRepository` への保存処理の統合
    - [x] 「OK」ボタン押下時に、選択されたフォント設定を永続化する。
- [x] Task: 起動時の設定反映の確認
    - [x] 保存された設定が次回のプラグインロード時に正しく読み込まれ、ターミナルの描画に使用されることを確認する。
- [x] Task: Conductor - ユーザー手動検証 'Phase 3' (Protocol in workflow.md)

## Phase 4: Review Fixes & Stabilization
GitHub PR および追加レビュー指摘に基づき、コードの堅牢性、ドキュメントの整合性、および設定保存の挙動を最終改善する。

- [x] Task: 追加レビュー指摘事項の修正
    - [x] `config_resolver.rs`: 親 HWND 取得の堅牢化 (wParam/lParam 対応)
    - [x] `config_gui_driver.rs`: MutexGuard 利用箇所の修正
    - [x] `config_gui_driver.rs`: EndDialog の panic 回避とエラーログ追加
    - [x] `config_gui_driver.rs`: DialogBoxParamW の失敗ログ追加
    - [x] `config_gui_driver.rs`: UTF-16 バッファ変換処理の正規化
    - [x] `emeditor_config_repository_impl.rs`: query_string のリトライループ復元
    - [x] `emeditor_config_repository_impl.rs`: 保存時の空文字ガード撤廃（クリア/デフォルト戻しを許可）
    - [x] `build.rs`: rerun-if-changed 出力の追加
    - [x] `product.md`: フォント反映タイミングの記述修正（再起動後に反映）
    - [x] `spec.md`: 実装状況に合わせた要件記述の整理（Font Style の除外）
- [x] Task: Conductor - ユーザー手動検証 'Phase 4' (Protocol in workflow.md)
