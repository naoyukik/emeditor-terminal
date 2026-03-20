# Implementation Plan: ThemeType 変換ロジックの集約 (Issue #129)

`ThemeType` とインデックス・表示名の相互変換ロジックをドメイン層に集約し、各層での重複を排除する。

## Phase 1: ドメイン層の強化 (ThemeType の刷新)
`ThemeType` に全ての変換ロジックを集約し、将来の拡張に耐えうる構造にする。

- [x] Task: `ThemeType` に表示名（UI文字列）を返すメソッドを追加
- [x] Task: `ThemeType::from_index` / `to_index` の実装を網羅的かつ堅牢にする
- [x] Task: `ThemeType` の変換ロジックをテストするユニットテストの作成・拡充
- [x] Task: `cargo fmt` および `cargo clippy` によるコード品質確認
- [x] Task: Phase 1 完了のコミット
- [x] Task: Conductor - User Manual Verification 'Phase 1: ドメイン層の強化' (Protocol in workflow.md)

## Phase 2: インフラ層・GUI層の置換
既存の重複した `match` ロジックを、Phase 1 で作成したメソッドに置き換える。

- [x] Task: `src/infra/repository/emeditor_config_repository_impl.rs` の変換ロジックを `ThemeType` のメソッドに置換
- [x] Task: `src/gui/driver/config_gui_driver.rs` の ComboBox 処理を `ThemeType` のメソッドに置換
- [x] Task: `src/application/config_workflow.rs` 等、他の参照箇所を確認・修正
- [x] Task: `cargo fmt` および `cargo clippy` によるコード品質確認
- [x] Task: Phase 2 完了のコミット
- [x] Task: Conductor - User Manual Verification 'Phase 2: インフラ層・GUI層の置換' (Protocol in workflow.md)

## Phase 3: 最終検証
全体的な動作確認と、コードのクリーンアップを行う。

- [x] Task: `cargo test` による全テストの通過確認
- [x] Task: `cargo clippy` による警告の確認・修正
- [x] Task: 設定ダイアログでのテーマ切り替えと保存・復元の手動動作確認
- [x] Task: `cargo fmt` による最終フォーマット確認
- [x] Task: Phase 3 完了のコミット
- [x] Task: Conductor - User Manual Verification 'Phase 3: 最終検証' (Protocol in workflow.md)
