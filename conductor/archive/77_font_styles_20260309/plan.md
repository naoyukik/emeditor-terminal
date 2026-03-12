### Issue #77 フォントスタイルの外部化と描画への反映 実装プラン

#### フェーズ 0: 技術調査と設計確認
- [x] Task: Microsoft Learn を用いた `LOGFONTW` および `CHOOSEFONTW` の Weight/Italic 属性の再調査
- [x] Task: EmEditor SDK における設定保存 (EP_SET_PROPERTIES) と反映タイミングの制約確認
- [x] Task: Conductor - User Manual Verification 'Phase 0' (Protocol in workflow.md)
- [x] Task: `style`, `fmt`, `chore`: フェーズ完了の整合性確認とコミット
    - [x] `cargo fmt` および `cargo clippy` の実行
    - [x] `chore(conductor): Phase 0 技術調査完了`

**調査メモ (Research Notes):**
- **LOGFONTW 属性**: `lfWeight` は `i32` 型（400: 標準, 700: 太字）。`lfItalic` は `u8` 型（1: 斜体, 0: 標準）。
- **CHOOSEFONTW 連携**: ユーザーがダイアログで選択したスタイルは、`lpLogFont` が指す `LOGFONTW` 構造体に格納されて返される。
- **EmEditor 永続化**: プラグイン独自の設定保存には、既存の `EE_REG_QUERY_VALUE` / `EE_REG_SET_VALUE` を用いたレジストリ/INIアクセスが最適。
- **設定キー**: `FontWeight` (DWORD), `FontItalic` (DWORD) を `[Terminal]` セクションに追加する。

#### フェーズ 1: ドメインとインフラストラクチャの拡張
- [x] Task: `TerminalConfig` モデルの拡張（font_weight, font_italic）
    - [x] `src/domain/model/terminal_config_value.rs` に `font_weight: i32` と `font_italic: bool` を追加
    - [x] `Default` 実装およびユニットテストを更新
- [x] Task: `ConfigurationRepository` 実装の更新
    - [x] `src/infra/repository/emeditor_config_repository_impl.rs` にて `FontWeight` と `FontItalic` の読み書きを実装
- [x] Task: Conductor - ユーザー手動検証 'Phase 1' (Protocol in workflow.md)
- [x] Task: `style`, `fmt`, `feat`: フェーズ完了の整合性確認とコミット
    - [x] `cargo fmt` および `cargo clippy` の実行
    - [x] `feat: TerminalConfig と永続化層の拡張 (Weight/Italic)`

#### フェーズ 2: GUI 設定ダイアログの強化
- [x] Task: `SettingsDialogDriver` におけるスタイル取得ロジックの強化
    - [x] `src/gui/driver/settings_dialog_driver.rs` の `on_font_button_click` で `lfWeight` / `lfItalic` を取得・保持
- [ ] Task: ダイアログ UI の表示幅調整 (長いフォント名への対応)
- [x] Task: Conductor - ユーザー手動検証 'Phase 2' (Protocol in workflow.md)
- [x] Task: `style`, `fmt`, `feat`: フェーズ完了の整合性確認とコミット
    - [x] `cargo fmt` および `cargo clippy` の実行
    - [x] `feat: 設定ダイアログでのフォントスタイル取得機能の実装`

#### フェーズ 3: コア描画処理の実装と反映
- [x] Task: `TerminalGuiDriver` のフォント生成ロジックの刷新
    - [x] 初期化時または描画時に `TerminalConfig` を受け取るようにインターフェースを変更
    - [x] `get_font_for_style` でハードコードされている `Consolas` と `lfHeight: 16` を `TerminalConfig` の値に置き換える
    - [x] `TerminalConfig` の `font_weight` と `font_italic` を `HFONT` 生成ロジックに適用する
- [x] Task: 描画ロジックの目視検証 (EmEditor 実機)
- [x] Task: Conductor - ユーザー手動検証 'Phase 3' (Protocol in workflow.md)
- [x] Task: `style`, `fmt`, `feat`: フェーズ完了の整合性確認とコミット
    - [x] `cargo fmt` および `cargo clippy` の実行
    - [x] `feat: ターミナル描画へのフォントスタイルおよび書体・サイズの完全反映`

#### フェーズ 4: 最終検証とドキュメント更新
- [x] Task: エンドツーエンド (E2E) テストによる永続化の確認
- [x] Task: `product.md` の現在のステータスを更新
- [x] Task: Conductor - ユーザー手動検証 'Phase 4' (Protocol in workflow.md)
- [x] Task: `style`, `fmt`, `docs`: 最終クリーンアップとコミット
    - [x] `cargo fmt` および `cargo clippy` の実行
    - [x] `docs: Issue #77 の完了とドキュメント更新`

#### フェーズ 5: PRレビュー指摘への対応 (Critical/Bug Fix)
- [x] Task: 安全性とバグ修正の実施
    - [x] `GetDC` の NULL チェックとエラーハンドリングの追加 (`window_message_resolver.rs`)
    - [x] `get_font_for_style` の無限再帰防止 (`terminal_gui_driver.rs`)
    - [x] 描画時の DPI 計算を `hdc` ベースに修正 (`points_to_pixels` のオーバーロードまたは変更)
    - [x] 太字ウェイトの合成ロジック改善 (`max(config.font_weight, 700)`)
- [x] Task: Conductor - ユーザー手動検証 'Phase 5'
- [x] Task: `style`, `fmt`, `fix`: 指摘対応の完了とコミット

**Future Tickets (PRレビューより):**
- **Issue #123**: `TerminalWorkflow` の冗長なフォントキャッシュフィールドを削除。
- **Issue #124**: 設定変更時の描画エンジンキャッシュクリア（Live Reload 準備）。
