# Implementation Plan - Issue #136: 起動時の自動フォーカス設定

## Gate: 依存Issue #61の先行解消
Issue #136 は、Issue #61（ターミナル起動中の再実行クラッシュ）解消後にのみ着手する。

- [ ] Task: Issue #61 を先行修正し、受け入れ条件（再実行時クラッシュなし・既存ターミナルへフォーカス）を満たすことを確認
- [ ] Task: Issue #61 の修正が `main` に取り込まれたこと（Issue Close とコミット SHA）を確認
- [ ] Task: #136 作業ブランチへ #61 修正を取り込み、`OnCommand` 再実行時クラッシュが再発しないことを確認
- [ ] Task: Conductor - User Manual Verification 'Gate: Issue #61 完了確認' (Protocol in workflow.md)

## Phase 0: 自律型リサーチと既存コードの分析
`autonomous-researcher` スキルを活用し、Windows API (`SetFocus`) の最適な呼び出しタイミングと `GuiDriver` 層の現状を調査する。

- [ ] Task: `autonomous-researcher` による `SetFocus` と `OnCommand` 実行時フォーカス遷移の挙動調査
    - [ ] 表示成功直後に `SetFocus` を呼ぶ際の制約やベストプラクティスを調査。
    - [ ] EmEditor プラグイン（子ウィンドウ）におけるフォーカス管理の特異性を確認。
- [ ] Task: `window` / `resolver` レイヤーのハンドラ調査
    - [ ] `src/gui/window/mod.rs` の `wnd_proc` と `open_custom_bar` のフォーカス関連コードを調査。
    - [ ] `src/gui/resolver/window_message_resolver.rs` の `WM_SETFOCUS` / `WM_KILLFOCUS` 関連コードを調査。
- [ ] Task: Input ドメインモデルへの影響調査
    - [ ] `src/domain/model/input.rs` とキーボードフック処理の整合性を確認。
- [ ] Task: `cargo clippy` & `cargo fmt` の実行
- [ ] Task: 実機・手動検証 'Phase 0' (調査結果の妥当性確認)
- [ ] Task: `git commit` (chore: Phase 0 の調査結果と設計指針を反映)
- [ ] Task: Conductor - User Manual Verification 'Phase 0' (Protocol in workflow.md)

## Phase 1: フォーカス設定ロジックの実装
特定したメッセージハンドラに `SetFocus` の呼び出しを組み込み、基本的な動作を実現する。

- [ ] Task: `OnCommand` 表示成功時の `SetFocus` ロジック追加
    - [ ] プラグインバー表示処理が成功したタイミングでターミナルペインへフォーカスを移す。
    - [ ] EmEditor メニュー/モーダル表示中のフォーカス奪取抑止を実装する。
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
