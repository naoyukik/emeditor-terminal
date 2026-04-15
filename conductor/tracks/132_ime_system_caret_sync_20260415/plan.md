# Implementation Plan: Issue 132 - IME System Caret Sync (Clean-Slate Experimentation)

## Phase 1: 問題の把握と詳細設計 (Discovery & Detailed Design)
このフェーズでは、コードの変更を行わず、現在のエミュレータ実装と TUI アプリケーション間の座標系の乖離やイベントループの競合要因を調査する。
- [x] Task: `autonomous-researcher` スキルを使用し、`src/gui/driver/ime_gui_driver.rs` および `terminal_gui_driver.rs` 等の関連コードを詳細に調査し、`evidence_report.md` を作成する。
- [x] Task: 調査結果に基づき、`plan.md` の Phase 2 以降のタスク（具体的にどこにデバッグ機構を仕込むか、最初の仮説は何か）をより具体化する。
- [x] Task: Conductor - ユーザー手動検証 'Phase 1' (調査結果と具体的プランの承認 - Protocol in workflow.md)

## Phase 2: 実験基盤の構築 (Foundation & Debug Harness)
このフェーズでは、問題を観察しやすくするための最小限のデバッグ機構のみを導入する。
- [x] Task: `src/gui/driver/ime_gui_driver.rs` の `sync_system_caret` に、同期時の座標、HWND、`GetFocus()` の結果、およびビューポートオフセットを出力する詳細な `log::info!` を追加する。
- [x] Task: `src/gui/resolver/window_message_resolver.rs` の IME 関連ハンドラ (`on_ime_start_composition`, `on_ime_composition`) に、メッセージ受信を知らせるログを追加する。
- [ ] Task: この時点での変更を `chore: add debug harness for IME caret sync` としてコミットし、「クリーンな実験の起点」として保存する。
- [ ] Task: Conductor - ユーザー手動検証 'Phase 2' (Protocol in workflow.md)

## Phase 3: 反復検証サイクル (Iterative Experimentation)
このフェーズは、成功するまで「仮説立案 → 実装 → 検証 →（失敗時はリセットして）再仮説」を繰り返す。
- [ ] Task: **Hypothesis A (強制同期)**: `sync_system_caret` の呼び出しを、`PostMessage` (WM_APP_REPAINT) 経由ではなく、IME メッセージ受信時に直接かつ同期的に行うよう調整し、必要に応じて `UpdateWindow` で描画を確定させる。
- [ ] Task: Gemini CLI を実機で起動し、IME 候補ウィンドウが仮想カーソル位置に正しく追従するか手動テストを行う。
- [ ] Task: Conductor - ユーザー手動検証 'Phase 3' (Protocol in workflow.md)
  - **※重要**: この検証に失敗した場合、AIまたはユーザーは直ちに `git reset --hard HEAD` 等を用いて Phase 2 完了時のクリーンな状態に戻り、`evidence_report.md` に失敗理由を追記した上で、新たな仮説（Hypothesis B）を立ててこのフェーズを再試行すること。
- [ ] Task: **Hypothesis B (フォーカス判定緩和)**: `sync_system_caret` 内の `GetFocus()` 判定が原因で同期がスキップされている場合、判定条件を緩和（IME構成中なら許可等）し、再検証する。

## Phase 4: 最終統合とクリーンアップ (Final Integration & Cleanup)
Phase 3 で完全な成功が証明された場合のみ、このフェーズに進む。
- [ ] Task: Phase 2 で導入した不要なデバッグログやテスト機構（Debug Harness）を削除する。
- [ ] Task: `cargo clippy` および `cargo fmt` を実行し、コードの品質とフォーマットを保証する。
- [ ] Task: 成功した実装内容を `fix: システムキャレットの常時同期による IME 候補ウィンドウ表示位置の汎用的解決` としてコミットする。
- [ ] Task: Conductor - ユーザー手動検証 'Phase 4' (Protocol in workflow.md)
