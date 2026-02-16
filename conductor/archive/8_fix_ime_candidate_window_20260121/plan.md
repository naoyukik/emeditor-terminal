# Track Plan: 8_fix_ime_candidate_window_20260121

## Phase 1: Preparation & Analysis
- [x] Task: Conductor - ユーザー手動検証 'Phase 1: Preparation & Analysis' (Protocol in workflow.md)

## Phase 2: Window Handle & Message Loop Refactoring
- [x] Task: リファクタリング - `src/custom_bar.rs` の `wnd_proc` を拡張し、IME関連メッセージ (`WM_IME_STARTCOMPOSITION`, `WM_IME_COMPOSITION`, `WM_IME_ENDCOMPOSITION`, `WM_IME_SETCONTEXT`) をハンドルする準備を行う。**各メッセージ受信時に詳細なデバッグログ（WPARAM, LPARAMの内容）を出力する処理を含めること。**
- [ ] Task: テスト - 擬似的なメッセージ送信によるハンドラの単体テスト実装。
- [x] Task: 実装 - `WM_IME_SETCONTEXT` を処理し、アプリケーション側でIME描画を制御することを通知する。
- [x] Task: Conductor - ユーザー手動検証 'Phase 2: Window Handle & Message Loop Refactoring' (Protocol in workflow.md)

## Phase 3: Inline Composition Implementation
- [x] Task: 実装 - `WM_IME_COMPOSITION` ハンドラ内で `ImmGetCompositionStringW` を使用し、未確定文字列を取得するロジックを実装。**取得した文字列の内容と長さをログ出力すること。**
- [x] Task: 実装 - 未確定文字列を保持する構造体 (`CompositionInfo`) を `TerminalData` に追加。
- [x] Task: 実装 - `WM_PAINT` 処理を修正し、`TerminalData` に未確定文字列が存在する場合、カーソル位置にオーバーレイ描画するロジックを追加。**描画座標と対象文字列をログ出力すること。**
- [ ] Task: テスト - 未確定文字列を含むバッファの描画テスト。
- [ ] Task: Conductor - ユーザー手動検証 'Phase 3: Inline Composition Implementation' (Protocol in workflow.md)

## Phase 4: Candidate Window Positioning
- [x] Task: 実装 - フォントメトリクスとカーソル位置から、画面上のピクセル座標を計算するヘルパー関数を実装。**計算されたピクセル座標をログ出力すること。**
- [x] Task: 実装 - `WM_IME_COMPOSITION` またはカーソル移動時に `SetCaretPos` を呼び出し、システムキャレットを更新。**更新前後のキャレット位置をログ出力すること。**
- [x] Task: 実装 - `ImmSetCompositionWindow` を使用して、候補ウィンドウの位置（`COMPOSITIONFORM`）を更新するロジックを追加。**設定しようとするCOMPOSITIONFORMの内容をログ出力すること。**
- [x] Task: Conductor - ユーザー手動検証 'Phase 4: Candidate Window Positioning' (Protocol in workflow.md)

## Phase 5: Finalization & Cleanup
- [x] Task: 実装 - `WM_IME_ENDCOMPOSITION` 処理を追加し、インライン表示のクリアと確定文字列の ConPTY 送信を実装。**確定文字列と送信バイト数をログ出力すること。**
- [ ] Task: リファクタリング - コード整理。**検証に必要なログは残しつつ、過剰なデバッグログについてはログレベルを調整（debug/trace化）する。**
- [ ] Task: Conductor - ユーザー手動検証 'Phase 5: Finalization & Cleanup' (Protocol in workflow.md)
