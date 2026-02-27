# 軌跡実施計画 (Track Implementation Plan): 9_cursor_shape_20260227

## Phase 0: 調査と仕様確認 (Research & Analysis)
**目的:** `DECSCUSR` (Set Cursor Style) シーケンスの正確な仕様（xterm, VT520 等の互換性）と、`vte` クレートでのパース動作を調査する。

- [ ] Task: `DECSCUSR` のエスケープシーケンス仕様の調査
    - [ ] Learn Microsoft および VT100.net 等のリファレンスで `DECSCUSR` (`ESC [ n q`) のパラメータと各形状の関係を確認。
    - [ ] デフォルト値（n=0）の挙動（多くのターミナルでは 1 と同等）を確認。
- [ ] Task: `vte` クレートの `csi_dispatch` での受け取り方の確認
    - [ ] `vte` クレートにおいて、中間バイトなしの `action == 'q'` が正しく `csi_dispatch` へ送られるか確認。
- [ ] Task: 調査結果に基づく実装方針の最終決定
- [ ] Task: Conductor - ユーザー手動検証 'Phase 0: Research' (Protocol in workflow.md)
- [ ] Task: `git commit -m "chore(conductor): Phase 0 Research completed"`

## Phase 1: ドメイン層の拡張 (Domain Layer Extension)
**目的:** `TerminalBufferEntity` にカーソル形状（Style）の状態を追加し、`vte::Perform` トレイトで `DECSCUSR` シーケンスをパース・処理できるようにする。

- [ ] Task: `Cursor` 構造体に `CursorStyle` 列挙型を追加
    - [ ] `src/domain/model/terminal_buffer_entity.rs` に `CursorStyle` を定義 (Block, Underline, Bar, 各 Blinking/Steady)
    - [ ] `Cursor` 構造体に `style: CursorStyle` フィールドを追加
- [ ] Task: `vte::Perform::csi_dispatch` の `q` アクション (DECSCUSR) の実装
    - [ ] `csi_dispatch` 内で `action == 'q'` の場合に `handle_decscusr` を呼び出すロジックを追加
    - [ ] `handle_decscusr` メソッドで `CursorStyle` を更新する
- [ ] Task: ユニットテストの追加
    - [ ] `TerminalBufferEntity` のテストで `DECSCUSR` シーケンスによるスタイル変更を検証する
- [ ] Task: Conductor - ユーザー手動検証 'Phase 1: Domain Layer' (Protocol in workflow.md)
- [ ] Task: `git commit -m "feat: DECSCUSR (Cursor Style) support in Domain layer"`

## Phase 2: GUI 層の描画実装 (GUI Layer Implementation)
**目的:** `TerminalGuiDriver` において、`TerminalBufferEntity` から取得した `CursorStyle` に基づいてカーソルを適切に描画する。

- [ ] Task: `TerminalBufferEntity` に `CursorStyle` の Getter を追加
    - [ ] `src/domain/model/terminal_buffer_entity.rs` に `get_cursor_style()` を実装
- [ ] Task: `TerminalGuiDriver` のカーソル描画ロジックの修正
    - [ ] `src/gui/driver/terminal_gui_driver.rs` の `render_internal` 内のカーソル描画部分を修正
    - [ ] `CursorStyle` に応じた `RECT` 計算（Block/Underline/Bar）を実装
- [ ] Task: 描画テストと動作確認
    - [ ] 各カーソル形状が正しく描画されることを目視で確認
- [ ] Task: Conductor - ユーザー手動検証 'Phase 2: GUI Layer' (Protocol in workflow.md)
- [ ] Task: `git commit -m "feat: Render cursor shape based on DECSCUSR style"`

## Phase 3: 最終調整とドキュメント更新 (Final Adjustment & Docs)
**目的:** 全体的な動作確認を行い、ドキュメントを更新する。

- [ ] Task: 全体テストとリファクタリング
    - [ ] `cargo test`, `cargo clippy` の実行
- [ ] Task: 完了報告とドキュメントの整理
- [ ] Task: Conductor - ユーザー手動検証 'Phase 3: Final' (Protocol in workflow.md)
- [ ] Task: `git commit -m "chore(conductor): Track 9_cursor_shape_20260227 completed"`
