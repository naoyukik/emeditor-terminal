# Specification: Issue 61 ターミナル二重起動時のクラッシュ修正

## 1. 概要 (Overview)
ターミナル起動中に再度起動ボタンを押すとアプリケーションがクラッシュする問題（Issue 61）を修正する。
本トラックは `conductor/archive/61_autonomous_research_20260409/evidence_report.md` の調査結果に基づき、安全なウィンドウハンドル操作と初期化プロセスのトランザクション化を実現する。

## 2. 目的 (Goals)
- ターミナルの二重起動によるクラッシュ（無効なHWNDの参照等）を防ぐ。
- `WindowGuiDriver` を新設し、Win32ウィンドウ操作を隔離する。
- ConPTY 起動失敗時などに不完全な状態（ゾンビハンドル）を残さない。

## 3. 要件 (Functional Requirements)

### 3.1. `WindowGuiDriver` の実装
- `src/gui/driver/window_gui_driver.rs` を新設する。
- 以下のWin32 API呼び出しをカプセル化した静的メソッドを持つ構造体として実装する。
  - 既存ウィンドウの生存確認とフォーカス (`IsWindow`, `SetFocus`)
  - ウィンドウの安全な破棄 (`DestroyWindow`)

### 3.2. トランザクション的初期化 (`open_custom_bar`)
- 既存の `window_handle` を確認し、有効であれば `WindowGuiDriver` を介してフォーカスを当て、新たな起動処理をスキップする。
- `window_handle` に無効なハンドルが残っている場合は、`WindowGuiDriver` でクリアする。
- `start_conpty_and_reader_thread` が失敗した場合、中途半端な `window_handle` を残さないよう `WindowGuiDriver::destroy_window` を確実に呼び出し、状態を `None` にリセットする。

## 4. スコープ外 (Out of Scope)
- `TERMINAL_DATA` を HashMap 化するマルチウィンドウ対応（将来のトラックに譲る）。

## 5. 受け入れ条件 (Acceptance Criteria)
- [ ] ターミナル起動済みの状態で、再度起動ボタンを押してもクラッシュしないこと。
- [ ] すでにターミナルが起動している場合は、既存のターミナルウィンドウにフォーカスが当たる（前面に出る）こと。
- [ ] 何らかのエラーで起動に失敗した後、再度起動ボタンを押しても問題なくリカバリ・再起動を試行できること。
- [ ] `WindowGuiDriver` が導入され、`src/gui/window/mod.rs` 等の Win32 ウィンドウ操作の責務が隔離されていること。
