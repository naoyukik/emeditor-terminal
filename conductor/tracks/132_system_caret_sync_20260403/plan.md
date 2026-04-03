# Implementation Plan: システムキャレットの常時同期による IME 候補ウィンドウ位置の解決

## Phase 0: 調査と証拠収集 (Research & Evidence Gathering)
- [x] Task: `autonomous-researcher` スキルを使用し、Win32 API における `SetCaretPos` と `ImmSetCompositionWindow` の座標系要件の証拠（Evidence）を収集
    - `SetCaretPos` はクライアント座標、IME 制御はスクリーン座標であることの確証を Evidence Report として作成。
- [ ] Task: Conductor - User Manual Verification 'Phase 0: Research' (Protocol in workflow.md)

## Phase 1: 基礎インフラの整備と検証 (Infrastructure & Resource Management)
- [ ] Task: `src/gui/driver/ime_gui_driver.rs` における `CaretHandle` のライフサイクル検証
    - UI スレッド拘束の厳格化と、不適切なスレッドからの破棄防止の確認。
- [ ] Task: 座標変換ロジック (`cell_to_pixel`) の正確性向上とユニットテスト追加
- [ ] Task: 各タスク完了時のコミット、`cargo clippy` & `cargo fmt` の実行
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Infrastructure' (Protocol in workflow.md)

## Phase 2: 座標同期ロジックの「スクリーン座標」対応 (Coordination & Screen Sync)
- [ ] Task: `ClientToScreen` を使用した IME ウィンドウ位置（スクリーン座標）の計算実装
- [ ] Task: `GetFocus` ガードの導入による EmEditor 本体（親ウィンドウ）への干渉防止
- [ ] Task: `TerminalWorkflow` におけるカーソル移動（文字入力・エスケープシーケンス）時の同期フック実装
- [ ] Task: 各タスク完了時のコミット、`cargo clippy` & `cargo fmt` の実行
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Coordination' (Protocol in workflow.md)

## Phase 3: 全体的統合と古い実装の整理 (Global Integration & Cleanup)
- [ ] Task: 描画ループ (`on_paint`) および出力受信タイミングへの `sync_system_caret` 統合
- [ ] Task: `WM_IME_COMPOSITION` ハンドラ等に残存する古い座標計算の完全削除
- [ ] Task: スクロール発生時のキャレット位置補正の最終調整
- [ ] Task: 各タスク完了時のコミット、`cargo clippy` & `cargo fmt` の実行
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Integration' (Protocol in workflow.md)

## Phase 4: 実機検証と回帰テスト (Final Validation & Regression Test)
- [ ] Task: ユーザー環境 (ATOK) および Gemini CLI での候補ウィンドウ位置の最終確認
- [ ] Task: Vim / PSReadLine 等の標準 TUI アプリにおける追従性の網羅的検証
- [ ] Task: 最終的なコード品質チェックとクリーンアップ
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Final Validation' (Protocol in workflow.md)
