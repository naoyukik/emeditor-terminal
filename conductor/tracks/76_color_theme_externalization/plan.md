# 実装計画: カラーテーマ設定の外部化とファイルからのロード機能 (Issue #76)

## Phase 1: ドメイン層・インフラ層の拡張 (ThemeTypeの永続化対応)
- [ ] Task: ドメインモデルの修正
    - [ ] `TerminalConfig` 構造体 (`src/domain/model/terminal_config.rs`) を確認し、`theme_type` 関連のフィールドがシリアライズ/デシリアライズ可能な形になっているか検証。
    - [ ] 必要に応じて `ThemeType` Enum に `SystemDefault` (Auto) の定義を追加。
- [ ] Task: インフラ層（永続化ロジック）の実装 (TDD)
    - [ ] `src/infra/repository/emeditor_config_repository_impl.rs` を開き、設定読み書き用のキー（例: `ColorTheme`）を定義。
    - [ ] `EE_REG_QUERY_VALUE` / `EE_REG_SET_VALUE` を用いて、`ThemeType` を文字列または整数値としてレジストリ/INIから読み書きするロジックを実装。
    - [ ] 既存キーが存在しない場合は `SystemDefault` をデフォルト値としてフォールバックするロジックを追加。
    - [ ] `EmEditorConfigRepositoryImpl` のテストコード（Mockを用いた永続化テスト）を追加・実行。
- [ ] Task: Conductor - Clippy & fmt Check (Clippyは自動フォーマットを使用すること)
- [ ] Task: Conductor - 'Phase 1: ドメイン層・インフラ層の拡張' の成果をコミット
- [ ] Task: Conductor - User Manual Verification 'Phase 1: ドメイン層・インフラ層の拡張' (Protocol in workflow.md)

## Phase 2: GUI設定ダイアログへの統合
- [ ] Task: リソース定義の更新 (`emeditor-terminal.rc`)
    - [ ] `IDD_CONFIG_DIALOG` ダイアログリソースに、カラーテーマ選択用のコンボボックス (`COMBOBOX`, 例: `IDC_COMBO_THEME`) およびラベルテキストを追加。
- [ ] Task: GUI Driver (ダイアログ制御) の実装
    - [ ] `src/gui/driver/config_dialog_driver.rs` (または関連するダイアログプロシージャ) を開き、初期化 (`WM_INITDIALOG`) 時にコンボボックスへアイテム (`System Default`, `One Half Dark`, `One Half Light`) を追加する処理を実装。
    - [ ] `TerminalConfig` から受け取った現在のテーマ設定を、コンボボックスの初期選択状態に反映する処理を実装。
    - [ ] OKボタン押下時 (`OnCommand` の IDOK) に、コンボボックスの選択値を読み取り、`TerminalConfig` にセットして保存を要求する処理を実装。
- [ ] Task: GUI層の単体/ロジックテスト
    - [ ] コンボボックスのインデックスと `ThemeType` を相互変換するマッピングロジックのテストを追加。
- [ ] Task: Conductor - Clippy & fmt Check (Clippyは自動フォーマットを使用すること)
- [ ] Task: Conductor - 'Phase 2: GUI設定ダイアログへの統合' の成果をコミット
- [ ] Task: Conductor - User Manual Verification 'Phase 2: GUI設定ダイアログへの統合' (Protocol in workflow.md)

## Phase 3: System Default (Auto) テーマの解決と反映
- [ ] Task: テーマ解決ロジックの実装
    - [ ] `TerminalWorkflow` または `ThemeResolver` 等の適切な場所で、`ThemeType::SystemDefault` が指定された場合に、EmEditor または Windows OS の現在のカラーモード（Dark/Light）を判定して、具体的な `TerminalColorTheme` (Dark or Light) に解決する処理を実装。
    ※ 判定方法の調査（EmEditorのプロパティ参照、またはWindowsのレジストリ `AppsUseLightTheme` 等の参照）を含む。
- [ ] Task: ターミナル初期化時の適用
    - [ ] `TerminalWorkflow::initialize_terminal` (またはプラグインロード時) において、ロードされた `TerminalConfig` の `ThemeType` を元に、実際に描画エンジン (`TerminalGuiDriver`) に設定するカラーパレットを決定する。
- [ ] Task: Conductor - Clippy & fmt Check (Clippyは自動フォーマットを使用すること)
- [ ] Task: Conductor - 'Phase 3: System Default (Auto) テーマの解決と反映' の成果をコミット
- [ ] Task: Conductor - User Manual Verification 'Phase 3: System Default (Auto) テーマの解決と反映' (Protocol in workflow.md)

## Phase 4: 総合テストと最終検証
- [x] Task: ビルドと手動機能テスト
    - [x] プロジェクト全体をビルド (`cargo build`) し、エラー・警告がないことを確認。
    - [x] EmEditor上で設定ダイアログを開き、テーマを切り替えてOKを押す。
    - [x] 再起動後、ターミナルの背景色・文字色が選択したテーマに沿って描画されることを目視確認。
    - [x] `System Default` 選択時、OSまたはEmEditorのダークモード/ライトモード設定に応じたテーマが適用されることを確認。
- [x] Task: Conductor - Clippy & fmt Check (Clippyは自動フォーマットを使用すること)
- [x] Task: Conductor - 'Phase 4: 総合テストと最終検証' の成果をコミット
- [x] Task: Conductor - User Manual Verification 'Phase 4: 総合テストと最終検証' (Protocol in workflow.md)

## Phase: Review Fixes
- [x] Task: Apply review suggestions (sha: 7a1411a)