# Implementation Plan - Issue #136: 起動時の自動フォーカス設定

## Gate: 依存Issue #61の先行解消
Issue #136 は、Issue #61（ターミナル起動中の再実行クラッシュ）解消後にのみ着手する。

- [x] Task: Issue #61 を先行修正し、受け入れ条件（再実行時クラッシュなし・既存ターミナルへフォーカス）を満たすことを確認
- [x] Task: Issue #61 の修正が `main` に取り込まれたこと（Issue Close とコミット SHA）を確認
- [x] Task: #136 作業ブランチへ #61 修正を取り込み、`OnCommand` 再実行時クラッシュが再発しないことを確認
- [ ] Task: Conductor - User Manual Verification 'Gate: Issue #61 完了確認' (Protocol in workflow.md)

## Phase 0: 自律型リサーチと既存コードの分析
`autonomous-researcher` スキルを活用し、Windows API (`SetFocus`) の最適な呼び出しタイミングと `GuiDriver` 層の現状を調査する。

- [x] Task: `autonomous-researcher` による `SetFocus` と `OnCommand` 実行時フォーカス遷移の挙動調査
    - [x] 表示成功直後に `SetFocus` を呼ぶ際の制約やベストプラクティスを調査。
    - [x] EmEditor プラグイン（子ウィンドウ）におけるフォーカス管理の特異性を確認。
- [x] Task: `window` / `resolver` レイヤーのハンドラ調査
    - [x] `src/gui/window/mod.rs` の `wnd_proc` と `open_custom_bar` のフォーカス関連コードを調査。
    - [x] `src/gui/resolver/window_message_resolver.rs` の `WM_SETFOCUS` / `WM_KILLFOCUS` 関連コードを調査。
- [x] Task: Input ドメインモデルへの影響調査
    - [x] `src/domain/model/input.rs` とキーボードフック処理の整合性を確認。
- [x] Task: `cargo clippy` & `cargo fmt` の実行
- [x] Task: 実機・手動検証 'Phase 0' (調査結果の妥当性確認)
- [x] Task: `git commit` (chore: Phase 0 の調査結果と設計指針を反映)
- [x] Task: Conductor - User Manual Verification 'Phase 0' (Protocol in workflow.md)

## Phase 1: フォーカス設定ロジックの実装
調査結果に基づき、`open_custom_bar` 内で一貫したフォーカス制御を実現する。

- [ ] Task: `src/gui/window/mod.rs` の `open_custom_bar` への `SetFocus` 呼び出し追加
    - [ ] `start_conpty_and_reader_thread` 成功直後、`WindowGuiDriver::focus_existing_window(hwnd_client)` を実行して新規作成されたウィンドウにフォーカスを当てる。
    - [ ] 既にウィンドウが存在する場合 (`existing_hwnd` が有効な場合)、既に `focus_existing_window` が呼ばれているため、その挙動が期待通りであることをログ等で確認する。
- [ ] Task: `WindowGuiDriver` のフォーカス制御の堅牢化
    - [ ] `src/gui/driver/window_gui_driver.rs` の `focus_existing_window` において、`SetFocus` の戻り値をログ出力するように修正。
- [ ] Task: `cargo clippy` & `cargo fmt` の実行
- [ ] Task: `git commit` (feat: 起動時の自動フォーカス設定を実装)
- [ ] Task: Conductor - ユーザー手動検証 'Phase 1' (Protocol in workflow.md)

## Phase 2: 最終検証と堅牢化
実機での詳細な動作確認と、副作用（他の UI 要素との競合）の排除。

- [ ] Task: 影響調査と副作用の確認
    - [ ] 起動時に他の UI 要素（メニューやダイアログ）と競合しないか、実機で徹底確認。
    - [ ] `IsWindow` チェックが適切に機能し、クラッシュしないことを確認。
- [ ] Task: `cargo clippy` & `cargo fmt` の実行
- [ ] Task: `git commit` (fix/refactor: フォーカス管理の堅牢化とクリーンアップ)
- [ ] Task: Conductor - ユーザー手動検証 'Phase 2' (Protocol in workflow.md)
