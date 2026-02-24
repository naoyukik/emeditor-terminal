# Implementation Plan - Track: PR #89 Review Fixes

## Phase 1: 安全性とドキュメントの復元 (Safety & Documentation Restoration)
- [ ] Task: `SendHFONT` 構造体における `unsafe` 実装の正当性と制約ドキュメントの復元
- [ ] Task: `TerminalBufferEntity` および描画ループにおける配列アクセスの境界チェックの修正
    - [ ] `TerminalBufferEntity::put_char` 等のインデックス参照（`line[x]`, `line[x+1]`）への境界チェック追加
    - [ ] 描画ループ（`terminal_gui_driver.rs`）におけるインデックス参照への境界チェック追加
- [ ] Task: フォント作成失敗時のフォールバックロジック（`get_font_for_style`）の復元
- [ ] Task: chore(conductor): Phase 1: 安全性とドキュメントの復元完了をコミット
- [ ] Task: Conductor - User Manual Verification 'Phase 1: 安全性・ドキュメント復元完了' (Protocol in workflow.md)

## Phase 2: 表示ロジックとアクセシビリティの修正 (Logic & Accessibility Fix)
- [ ] Task: `char_display_width` における Unicode レンジ（ハングル、CJK互換文字等）の復元
- [ ] Task: タブ文字処理における `cursor.x` 二重加算の修正
- [ ] Task: 画面下部および行の右側の未使用領域クリア処理の復元（`FillRect`）
- [ ] Task: コントラスト自動補正ロジックの復元・改善
- [ ] Task: fix: PR #89 レビュー指摘事項の表示ロジック修正をコミット
- [ ] Task: Conductor - User Manual Verification 'Phase 2: 表示ロジック修正完了' (Protocol in workflow.md)

## Phase 3: テストの復元と最終検証 (Testing & Final Verification)
- [ ] Task: 削除された主要なユニットテスト（`test_sgr_colors`, `test_terminal_resize` 等）の復元と再実装
- [ ] Task: 復元・追加された全テストの実行とパスの確認 (`cargo test`)
- [ ] Task: `edit` を用いた表示の乱れや残像、クラッシュの有無の最終確認
- [ ] Task: test: PR #89 レビュー指摘事項に伴うテストの復元をコミット
- [ ] Task: chore(conductor): Phase 3: 全修正完了のコミット
- [ ] Task: Conductor - User Manual Verification 'Phase 3: 全修正完了' (Protocol in workflow.md)
