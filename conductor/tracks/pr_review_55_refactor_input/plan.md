# Track pr_review_55_refactor_input Implementation Plan

## Phase 1: Infra & GUI Layer Improvements
**必ずPhaseの最後にユーザーの手動確認を依頼すること。依頼しなかった場合は切腹を言い渡す。**
Infra層の安全性向上とGUI層のコード重複排除を行う。

- [ ] Task: 1.1 Infra層の修正
    - [ ] `src/infra/input.rs`: `install()` の順序修正（`TARGET_HWND` 設定をフック登録後に）。
    - [ ] `src/infra/input.rs`: `PostMessageW` のエラーハンドリング追加。
- [ ] Task: 1.2 GUI層の修正
    - [ ] `src/gui/custom_bar.rs`: `lparam` コメントの修正。
    - [ ] `src/gui/custom_bar.rs`: システムショートカット判定ロジックの共通化（ヘルパー関数導入）。
    - [ ] `src/gui/custom_bar.rs`: `WM_APP_KEYINPUT` と `WM_SYSKEYDOWN` の修正。
- [ ] Task: 1.3 ビルド検証
    - [ ] `cargo check`
- [ ] **Task: Conductor - User Manual Verification 'Infra & GUI' (Protocol in workflow.md)**

## Phase 2: Domain Layer Improvements
**必ずPhaseの最後にユーザーの手動確認を依頼すること。依頼しなかった場合は切腹を言い渡す。**
Domain層のテスト拡充とロジックの明示化を行う。

- [ ] Task: 2.1 ロジックの明示化
    - [ ] `src/domain/input.rs`: Alt+数字キーの変換ロジックを明示的な計算式（`b'0' + offset`）に変更。
- [ ] Task: 2.2 テストケースの追加
    - [ ] `src/domain/input.rs`: Ctrl+Alt 組み合わせのテスト。
    - [ ] `src/domain/input.rs`: ファンクションキー (F1-F12) のテスト。
    - [ ] `src/domain/input.rs`: PageUp/PageDown, Enter, Tab, Escape のテスト。
    - [ ] `src/domain/input.rs`: Alt+特殊キーのテスト。
- [ ] Task: 2.3 テスト実行
    - [ ] `cargo test domain::input`
- [ ] **Task: Conductor - User Manual Verification 'Domain' (Protocol in workflow.md)**

## Phase 3: Final Verification & Commit
**必ずPhaseの最後にユーザーの手動確認を依頼すること。依頼しなかった場合は切腹を言い渡す。**

- [ ] Task: 3.1 動作確認 (Manual Test)
    - [ ] EmEditor上での動作確認（特にシステムショートカット、Alt+数字、特殊キー）。
- [ ] Task: 3.2 コミットとプッシュ
    - [ ] 変更をコミットし、PR #55 にプッシュする。
- [ ] **Task: Conductor - User Manual Verification 'Final' (Protocol in workflow.md)**
