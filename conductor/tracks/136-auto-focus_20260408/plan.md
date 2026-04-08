# Implementation Plan - Issue #136: 起動時の自動フォーカス設定

## Phase 0: 自律型リサーチと既存コードの分析
`autonomous-researcher` スキルを活用し、Windows API (`SetFocus`) の最適な呼び出しタイミングと `GuiDriver` 層の現状を調査する。

- [ ] Task: `autonomous-researcher` による `SetFocus` と `WM_SHOWWINDOW` の挙動調査
    - [ ] `WM_SHOWWINDOW` 時に `SetFocus` を呼ぶ際の制約やベストプラクティスを調査。
    - [ ] EmEditor プラグイン（子ウィンドウ）におけるフォーカス管理の特異性を確認。
- [ ] Task: `GuiDriver` レイヤーのメッセージハンドラ調査
    - [ ] `src/gui/driver/` 配下の `window_proc` 関連コードを調査。
- [ ] Task: `cargo clippy` & `cargo fmt` の実行
- [ ] Task: 実機・手動検証 'Phase 0' (調査結果の妥当性確認)
- [ ] Task: `git commit` (chore: Phase 0 の調査結果と設計指針を反映)
- [ ] Task: Conductor - User Manual Verification 'Phase 0' (Protocol in workflow.md)

## Phase 1: フォーカス設定ロジックの実装
特定したメッセージハンドラに `SetFocus` の呼び出しを組み込み、基本的な動作を実現する。

- [ ] Task: `WM_SHOWWINDOW` への `SetFocus` ロジック追加
    - [ ] ウィンドウが表示（`wParam == TRUE`）される際にターミナルペインへフォーカスを移す。
- [ ] Task: `cargo clippy` & `cargo fmt` の実行
- [ ] Task: `git commit` (feat: 起動時の自動フォーカス設定を実装)
- [ ] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)

## Phase 2: 最終検証と堅牢化
実機での詳細な動作確認と、副作用（他の UI 要素との競合）の排除。

- [ ] Task: 影響調査と副作用の確認
    - [ ] 起動時に他の UI 要素（メニューやダイアログ）と競合しないか、実機で徹底確認。
- [ ] Task: `cargo clippy` & `cargo fmt` の実行
- [ ] Task: `git commit` (fix/refactor: フォーカス管理の堅牢化とクリーンアップ)
- [ ] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)
