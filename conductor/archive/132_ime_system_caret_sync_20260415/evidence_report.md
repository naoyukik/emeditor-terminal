# Evidence Report: Issue 132 - IME System Caret Sync (Clean-Slate Experimentation)

## 1. Discovery Summary (Phase 1)

- **[Problem Statement]**: 
  - Gemini CLI 等の TUI アプリケーション動作時、IME 候補ウィンドウが仮想カーソル位置に追従せず、(0,0) 等の誤った位置に表示される。
  - 通常のシェル入力（pwsh 等）では概ね正しく動作しているが、複雑な画面更新を伴う TUI 環境下で座標同期が壊れる。
- **[Scope]**: 
  - `src/gui/driver/ime_gui_driver.rs` における座標同期ロジックの修正。
  - `src/gui/resolver/window_message_resolver.rs` におけるイベント発生タイミングの最適化。
- **[Non-Goals]**: 
  - IME 入力メソッド自体の実装や変更。
  - ターミナル描画エンジン全体の刷新。
- **[Constraints]**: 
  - 厳格なレイヤードアーキテクチャ（Domain 層に GUI の知識を持ち込まない）を維持する。
  - Win32 API (`SetCaretPos`, `ImmSetCompositionWindow`) を適切に使用する。
- **[Success Criteria]**: 
  - Gemini CLI 上で日本語入力を行い、候補ウィンドウが常に現在の入力位置（仮想カーソル）の直下に表示されること。

## 2. Codebase Findings (Phase 2)

- **[Similar Implementations]**:
  - `src/gui/driver/ime_gui_driver.rs:88`: `sync_system_caret` 関数。システムキャレットと IME ウィンドウの位置を同期する核心ロジック。
  - `src/gui/resolver/window_message_resolver.rs:70`: `on_paint` 内での同期。
  - `src/gui/resolver/window_message_resolver.rs:478`: `on_app_repaint` (ConPTY 出力受信時) 内での同期。
- **[Architecture and Dependency Notes]**:
  - `TerminalWindowResolver` が `CaretHandle` (RAII) を保持し、UI スレッドでのみ操作を許可している。
  - 出力受信スレッドから `PostMessage` を介して UI スレッドに通知を送る仕組みになっている。
- **[Reusable Components]**:
  - `TerminalGuiDriver::cell_to_pixel`: セル座標からピクセル座標への変換ロジック。
- **[Estimated Impact Area]**:
  - `sync_system_caret` の呼び出しタイミング（イベントハンドラ）。
  - `SetCaretPos` を呼ぶ際のフォーカス判定ロジック。

## 3. Clarifying Questions (Phase 3)

- **[Open Questions]**:
  1. Gemini CLI 動作時、`GetFocus()` は常にターミナルウィンドウの `HWND` を返しているか？（IME ウィンドウにフォーカスが奪われていないか）
  2. `PostMessage` による非同期更新の遅延が、IME の候補ウィンドウ表示に影響を与えている可能性はあるか？
  3. `ImmSetCompositionWindow` に渡す `rcArea` (Exclusion Area) の計算は、TUI のカーソル位置に対して適切か？
- **[User Answers / Delegations]**:
  - (未回答)
- **[Unresolved Items]**:
  - 実機ログによる座標値の推移確認。

## 4. 将来の修正で期待される挙動 (Expected Behavior)
- VTE パースにより仮想カーソルが移動した際、即座に（あるいは IME が位置を参照する前に）システムキャレットと IME 構成ウィンドウの位置が同期されること。
- スクロールバックバッファ閲覧中や、フォーカスが外れている際には IME 候補ウィンドウが不正な位置に表示されないこと。

## 5. Architecture Options (Phase 4)

### Option A: Minimal Changes (Sync on Output)
- **[Change Targets]**: `sync_system_caret` の呼び出しを、より確実なタイミング（`WM_IME_COMPOSITION` の直前など）に集中させる。
- **[Pros]**: 既存ロジックへの影響が少ない。
- **[Cons/Risks]**: 非同期による遅延を根本的に解決できない可能性がある。

### Option B: Synchronous Caret Update (Direct Sync)
- **[Change Targets]**: `on_app_repaint` で `InvalidateRect` を呼ぶ前に、`sync_system_caret` を呼び出し、必要に応じて `UpdateWindow` を使用して同期性を高める。
- **[Pros]**: TUI 更新とキャレット位置の不一致を最小化できる。
- **[Cons/Risks]**: UI スレッドの負荷増大。

### Option C: Pragmatic Balance (Caret focus-agnostic sync)
- **[Change Targets]**: `GetFocus()` 判定を緩和し、IME 構成中であれば強制的に同期する、あるいは `ImmSetCompositionWindow` のパラメータを最適化する。
- **[Pros]**: フォーカス状態に依存しない安定した表示。
- **[Cons/Risks]**: 意図しないウィンドウでのキャレット干渉（ただしターミナルウィンドウ専用ならリスク低）。

- **[Recommended Option]**: Option B + C
- **[Reason]**: TUI アプリ特有の高速更新に対応するためには、同期性を高める (B) とともに、IME 入力中の安定性を確保する (C) の組み合わせが最適であると考えられる。

## 6. 推奨される実装方針 (Implementation Strategy)
- **[Architecture Alignment]**: `ime_gui_driver.rs` の責務を維持しつつ、`window_message_resolver.rs` でのイベント制御を強化する。
- **[Logic Changes]**:
  1. `sync_system_caret` のログ出力を強化し、実座標を確認可能にする。
  2. `WM_IME_STARTCOMPOSITION` 時に、最新の仮想カーソル位置を確実に取得・反映する。
- **[Validation Plan]**:
  - デバッグログによる座標追跡。
  - Gemini CLI 上での反復テスト。

## 7. Evidence and Alignment
- **[Source URLs]**:
  - https://learn.microsoft.com/en-us/windows/win32/api/imm/nf-imm-immsetcompositionwindow
  - https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setcaretpos
- **[Research Date]**: 2026-04-15
- **[Key Findings]**:
  - `SetCaretPos` はキャレットを所有しているスレッドのキューにあるときのみ動作する。
  - IME はシステムキャレットの位置を候補ウィンドウのデフォルト位置として使用する。
- **[Local Constraint Alignment]**: 規約に準拠。
- **[Potential Regressions]**: 通常のシェル入力時。
- **[Residual Risks / Unknowns]**: 特定の IME (Google 日本語入力 vs MS-IME) による挙動の差。
