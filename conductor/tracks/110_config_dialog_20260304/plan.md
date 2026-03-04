# Implementation Plan - Issue #110: プラグイン設定ダイアログの実装

## Phase 1: Resource Setup & SDK Message Integration
EmEditor SDK の `EP_QUERY_PROPERTIES` および `EP_SET_PROPERTIES` メッセージを処理し、ダイアログ表示のトリガーを実装する。また、リソーススクリプト (`.rc`) をプロジェクトに導入する。

- [ ] Task: プロジェクトへのリソーススクリプト (`emeditor-terminal.rc`) の導入
    - [ ] `build.rs` を更新し、`embed-resource` 等を用いてリソースをコンパイル対象に含める。
    - [ ] 最小限の設定ダイアログ（OK/Cancel/Static Text）を定義する。
- [ ] Task: EmEditor SDK メッセージの受信処理の実装 (`lib.rs`)
    - [ ] `EP_QUERY_PROPERTIES` に対して `TRUE` を返す。
    - [ ] `EP_SET_PROPERTIES` 受信時に `MessageBoxW` 等で動作を確認する。
- [ ] Task: Conductor - ユーザー手動検証 'Phase 1' (Protocol in workflow.md)

## Phase 2: Configuration Dialog Implementation
リソースダイアログを実際に表示し、Win32 ダイアログプロシージャを通じてコントロールを制御する。

- [ ] Task: 設定ダイアログ表示ロジックの実装
    - [ ] `DialogBoxParamW` を用いて、リソースベースのダイアログを表示する。
    - [ ] ダイアログプロシージャを実装し、初期化時に現在の設定をロードする。
- [ ] Task: `ChooseFontW` ダイアログの統合
    - [ ] 「Change Font」ボタン押下時に標準フォント選択ダイアログを表示する。
    - [ ] 選択された結果を親ダイアログ上のテキスト（例: "Current Font: MS Gothic, 12pt"）に反映させる。
- [ ] Task: Conductor - ユーザー手動検証 'Phase 2' (Protocol in workflow.md)

## Phase 3: Persistence & Verification
ダイアログで設定された値を保存し、次回の起動時に正しく反映されることを確認する。

- [ ] Task: `ConfigRepository` への保存処理の統合
    - [ ] 「OK」ボタン押下時に、選択されたフォント設定を永続化する。
- [ ] Task: 起動時の設定反映の確認
    - [ ] 保存された設定が次回のプラグインロード時に正しく読み込まれ、ターミナルの描画に使用されることを確認する。
- [ ] Task: Conductor - ユーザー手動検証 'Phase 3' (Protocol in workflow.md)
