# Implementation Plan: システムキャレットの常時同期による IME 候補ウィンドウ表示位置の解決

## Phase 0: Research & Evidence Gathering
ConPTY と Node.js (Gemini CLI) の IME 同期問題の背景と、他ターミナルの実装を調査する。

- [ ] Task: Microsoft Learn / GitHub での ConPTY と Node.js (Gemini CLI) の IME 同期問題の調査
    - なぜ画面端や画面下に候補ウィンドウが飛ぶのか、特定のシーケンスが影響していないかを確認
- [ ] Task: 他のモダンターミナル（Windows Terminal / Alacritty 等）のシステムキャレット同期実装の調査
- [ ] Task: Conductor - User Manual Verification 'Phase 0: Research' (Protocol in workflow.md)

## Phase 1: Infrastructure & Resource Management
システムキャレットを安全に管理するための基盤を Infrastructure 層に構築する。

- [ ] Task: `ImmGuiDriver` における `CaretHandle` (RAII) の実装
    - `Drop` トレイトによる `DestroyCaret` の確実な実行
- [ ] Task: `ImmGuiDriver` への `set_system_caret_pos(x, y)` メソッドの追加
- [ ] Task: ユニットテスト: キャレットリソースの生成・破棄・座標設定の正常系確認
- [ ] Task: 各タスク完了時のコミット、`cargo clippy` & `cargo fmt` の実行
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Infrastructure' (Protocol in workflow.md)

## Phase 2: Domain & Application Coordination
カーソル移動イベントを検知し、物理座標へ変換して Driver へ伝えるロジックを実装する。

- [ ] Task: `TerminalWorkflow` (Domain) におけるカーソル変更通知の強化
- [ ] Task: `TerminalService` (Application) への `sync_system_caret` 命令の実装
- [ ] Task: 仮想座標 (cell) からウィンドウ物理座標 (pixel) への変換ロジックの共通化
- [ ] Task: ユニットテスト: 異なるフォントサイズや拡大率における座標変換の正確性検証
- [ ] Task: 各タスク完了時のコミット、`cargo clippy` & `cargo fmt` の実行
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Coordination' (Protocol in workflow.md)

## Phase 3: Global Integration & Cleanup
すべてのカーソル移動パスに同期処理を組み込み、古い実装を削除する。

- [ ] Task: `TerminalGuiDriver` での描画ループ終了時および入力処理後の `sync_system_caret` 統合
- [ ] Task: エスケープシーケンス（CUP, VPA 等）およびスクロール処理への同期フック追加
- [ ] Task: `WM_IME_COMPOSITION` ハンドラ内の古い場当たり的な座標計算の削除
- [ ] Task: 各タスク完了時のコミット、`cargo clippy` & `cargo fmt` の実行
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Integration' (Protocol in workflow.md)

## Phase 4: Final Validation & Regression Test
実機環境での網羅的なテストを行い、安定性を確認する。

- [ ] Task: Gemini CLI での日本語入力候補ウィンドウ位置の正確性確認
- [ ] Task: Vim / Neovim / PSReadLine における追従性と退行確認
- [ ] Task: 最終的なコード品質チェックとクリーンアップ
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Final Validation' (Protocol in workflow.md)
