### Issue #77 フォントスタイルの外部化と描画への反映 実装プラン

#### フェーズ 0: 技術調査と設計確認
- [ ] Task: Microsoft Learn を用いた `LOGFONTW` および `CHOOSEFONTW` の Weight/Italic 属性の再調査
- [ ] Task: EmEditor SDK における設定保存 (EP_SET_PROPERTIES) と反映タイミングの制約確認
- [ ] Task: Conductor - User Manual Verification 'Phase 0' (Protocol in workflow.md)
- [ ] Task: `style`, `fmt`, `chore`: フェーズ完了の整合性確認とコミット
    - [ ] `cargo fmt` および `cargo clippy` の実行
    - [ ] `chore(conductor): Phase 0 技術調査完了`

#### フェーズ 1: ドメインとインフラストラクチャの拡張
- [ ] Task: `TerminalConfig` モデルの拡張（font_weight, font_italic）
    - [ ] `src/domain/model/terminal_config_value.rs` に `font_weight: i32` と `font_italic: bool` を追加
    - [ ] `Default` 実装およびユニットテストを更新
- [ ] Task: `ConfigurationRepository` 実装の更新
    - [ ] `src/infra/repository/emeditor_config_repository_impl.rs` にて `FontWeight` と `FontItalic` の読み書きを実装
- [ ] Task: Conductor - ユーザー手動検証 'Phase 1' (Protocol in workflow.md)
- [ ] Task: `style`, `fmt`, `feat`: フェーズ完了の整合性確認とコミット
    - [ ] `cargo fmt` および `cargo clippy` の実行
    - [ ] `feat: TerminalConfig と永続化層の拡張 (Weight/Italic)`

#### フェーズ 2: GUI 設定ダイアログの強化
- [ ] Task: `SettingsDialogDriver` におけるスタイル取得ロジックの強化
    - [ ] `src/gui/driver/settings_dialog_driver.rs` の `on_font_button_click` で `lfWeight` / `lfItalic` を取得・保持
- [ ] Task: ダイアログ UI の表示幅調整 (長いフォント名への対応)
- [ ] Task: Conductor - ユーザー手動検証 'Phase 2' (Protocol in workflow.md)
- [ ] Task: `style`, `fmt`, `feat`: フェーズ完了の整合性確認とコミット
    - [ ] `cargo fmt` および `cargo clippy` の実行
    - [ ] `feat: 設定ダイアログでのフォントスタイル取得機能の実装`

#### フェーズ 3: コア描画処理の実装と反映
- [ ] Task: `TerminalGuiDriver` のフォント生成ロジックの刷新
    - [ ] 初期化時または描画時に `TerminalConfig` を受け取るようにインターフェースを変更
    - [ ] `get_font_for_style` でハードコードされている `Consolas` と `lfHeight: 16` を `TerminalConfig` の値に置き換える
    - [ ] `TerminalConfig` の `font_weight` と `font_italic` を `HFONT` 生成ロジックに適用する
- [ ] Task: 描画ロジックの目視検証 (EmEditor 実機)
- [ ] Task: Conductor - ユーザー手動検証 'Phase 3' (Protocol in workflow.md)
- [ ] Task: `style`, `fmt`, `feat`: フェーズ完了の整合性確認とコミット
    - [ ] `cargo fmt` および `cargo clippy` の実行
    - [ ] `feat: ターミナル描画へのフォントスタイルおよび書体・サイズの完全反映`

#### フェーズ 4: 最終検証とドキュメント更新
- [ ] Task: エンドツーエンド (E2E) テストによる永続化の確認
- [ ] Task: `product.md` の現在のステータスを更新
- [ ] Task: Conductor - ユーザー手動検証 'Phase 4' (Protocol in workflow.md)
- [ ] Task: `style`, `fmt`, `docs`: 最終クリーンアップとコミット
    - [ ] `cargo fmt` および `cargo clippy` の実行
    - [ ] `docs: Issue #77 の完了とドキュメント更新`
