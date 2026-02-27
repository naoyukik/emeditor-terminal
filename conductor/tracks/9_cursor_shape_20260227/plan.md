# 軌跡実施計画 (Track Implementation Plan): 9_cursor_shape_20260227

## Phase 0: 調査と仕様確認 (Research & Analysis)
**目的:** `DECSCUSR` (Set Cursor Style) シーケンスの正確な仕様（xterm, VT520 等の互換性）と、`vte` クレートでのパース動作を調査する。

- [x] Task: `DECSCUSR` のエスケープシーケンス仕様の調査
- [x] Task: `vte` クレートの `csi_dispatch` での受け取り方の確認
- [x] Task: 調査結果に基づく実装方針の最終決定
- [x] Task: Conductor - ユーザー手動検証 'Phase 0: Research' (Protocol in workflow.md)
- [x] Task: `git commit -m "chore(conductor): Phase 0 Research completed"`

## Phase 1: ドメイン層の拡張 (Domain Layer Extension)
**目的:** `TerminalBufferEntity` にカーソル形状（Style）の状態を追加し、`vte::Perform` トレイトで `DECSCUSR` シーケンスをパース・処理できるようにする。

- [x] Task: `Cursor` 構造体に `CursorStyle` 列挙型を追加
- [x] Task: `vte::Perform::csi_dispatch` の `q` アクション (DECSCUSR) の実装
- [x] Task: ユニットテストの追加
- [x] Task: Conductor - ユーザー手動検証 'Phase 1: Domain Layer' (Protocol in workflow.md)
- [x] Task: `git commit -m "feat: DECSCUSR (Cursor Style) support in Domain layer"`

## Phase 2: GUI 層の描画実装 (GUI Layer Implementation)
**目的:** `TerminalGuiDriver` において、`TerminalBufferEntity` から取得した `CursorStyle` に基づいてカーソルを適切に描画する。

- [x] Task: `TerminalBufferEntity` に `CursorStyle` の Getter を追加
- [x] Task: `TerminalGuiDriver` のカーソル描画ロジックの修正
- [x] Task: 描画テストと動作確認
- [x] Task: Conductor - ユーザー手動検証 'Phase 2: GUI Layer' (Protocol in workflow.md)
- [x] Task: `git commit -m "feat: Render cursor shape based on DECSCUSR style"`

## Phase 3: 反転属性の改善と Gemini CLI への対応 (Inverse Video & Gemini CLI Support)
**目的:** Gemini CLI 等で多用される「反転属性（Inverse）」によるブロックカーソルを正しく描画し、可視性の問題を解決する。

- [x] Task: `TerminalGuiDriver` の反転描画ロジックの修正
- [x] Task: `TerminalBufferEntity` の `h` / `l` (DECSET/DECRST) パラメータ処理の修正
- [x] Task: デフォルトカーソル形状の調整
- [x] Task: 動作確認
- [x] Task: Conductor - ユーザー手動検証 'Phase 3: Inverse & Visibility' (Protocol in workflow.md)
- [x] Task: `git commit -m "fix: Correct inverse video rendering and DECSET/DECRST multi-parameter handling"`

## Phase 4: 最終調整とドキュメント更新 (Final Adjustment & Docs)
**目的:** 全体的な動作確認を行い、ドキュメントを更新する。

- [x] Task: 全体テストとリファクタリング
    - [x] `cargo test`, `cargo clippy` の実行
- [x] Task: 完了報告とドキュメントの整理
- [x] Task: Conductor - ユーザー手動検証 'Phase 4: Final' (Protocol in workflow.md)
- [x] Task: `git commit -m "chore(conductor): Track 9_cursor_shape_20260227 completed"`

## Phase 5: レビューフィードバックへの対応 (Addressing Review Feedback)
**目的:** Opus および AcePilot によるコードレビューの指摘事項を修正し、品質を向上させる。

- [x] Task: `DECSCUSR` の複数パラメータ処理の修正
    - [x] `handle_decscusr` 内で `params.iter().last()` を使用するように修正
- [x] Task: カーソル幅の全角文字（Wide Character）対応
    - [x] `TerminalGuiDriver` で描画対象セルの文字幅を取得し、矩形幅に反映
- [x] Task: 二重反転によるカーソル消失の防止
    - [x] セルが既に `is_inverse` の場合、物理カーソルの描画をスキップするロジックを追加
- [x] Task: `_intermediates` 引数の命名修正
    - [x] `csi_dispatch` の `_intermediates` を `intermediates` にリネーム
- [x] Task: テストケースの拡充
    - [x] 無効なパラメータ値、パラメータなし（デフォルト挙動）のテストを追加
- [x] Task: Conductor - ユーザー手動検証 'Phase 5: Review Feedback' (Protocol in workflow.md)
- [ ] Task: `git commit -m "fix: Address review feedback (multi-param, wide char, inverse visibility, naming)"`
