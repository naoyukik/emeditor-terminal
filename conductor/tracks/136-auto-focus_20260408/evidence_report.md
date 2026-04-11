# Evidence Report - Issue #136: 起動時の自動フォーカス設定

## 1. Discovery Summary
EmEditor プラグイン（カスタムバー）が起動された際、ユーザーが即座にキー入力を開始できるよう、ターミナルウィンドウに自動的にフォーカスを当てる必要がある。

## 2. Codebase Findings
### 2.1 フォーカス制御 (Focus Control)
- `src/gui/driver/window_gui_driver.rs`:
    - `WindowGuiDriver::focus_existing_window(hwnd: HWND)` が実装済み。
    - `IsWindow` チェックを行ってから `SetFocus` を呼び出すため安全。
- `src/gui/resolver/window_message_resolver.rs`:
    - `on_set_focus` において `KeyboardGuiDriver::install(hwnd)` を呼び出し、キーボードフックをインストールしている。
    - したがって、ウィンドウにフォーカスを当てれば、自動的にターミナルへの入力が有効になる。

### 2.2 起動プロセス (Startup Process)
- `src/gui/window/mod.rs`:
    - `open_custom_bar(hwnd_editor: HWND)` がエントリーポイント。
    - ウィンドウ作成 (`CreateWindowExW`) -> ハンドル保持 -> EmEditor への通知 (`EE_CUSTOM_BAR_OPEN`) -> ConPTY 起動 (`start_conpty_and_reader_thread`) の順で実行。
    - 最終的に `start_conpty_and_reader_thread` が `true` を返したタイミングが、フォーカスを当てるべき最適なポイント。

## 3. Clarifying Questions & Answers
- **Q**: EmEditor のメニュー表示中にフォーカスを奪ってはいけないという制約があるが、どう判定するか？
- **A**: `open_custom_bar` はユーザーがコマンド（ボタンクリック等）を明示的に実行した際に呼ばれるため、その時点ではユーザーの意図としてフォーカスを求めていると解釈できる。ただし、将来的に自動起動などが実装される場合は、`GetForegroundWindow` 等で EmEditor 本体の状態を確認する必要があるかもしれない。現時点では `open_custom_bar` の成功直後への挿入で十分。

## 4. Architecture Options
- **Option 1: `open_custom_bar` の末尾で実行 (推奨)**
    - 実装がシンプル。
    - 初期化が完全に完了（ConPTY 起動成功）してからフォーカスを当てるため、安全。
- **Option 2: `WM_CREATE` や `WM_SHOWWINDOW` ハンドラで実行**
    - メッセージリゾルバ側にロジックが分散する。
    - EmEditor 側のカスタムバー表示処理とのタイミング競合のリスクがある。

## 5. Recommendation
**Option 1** を採用する。
`src/gui/window/mod.rs` の `open_custom_bar` 関数において、`start_conpty_and_reader_thread` が `true` を返した直後に `WindowGuiDriver::focus_existing_window(hwnd_client)` を呼び出す。

## 6. Expected Behavior (将来の実装で達成すべき挙動)
1. ユーザーがプラグインボタンをクリックする。
2. ターミナルウィンドウが作成され、ConPTY が起動する。
3. ターミナルウィンドウにフォーカスが当たる。
4. ユーザーが即座に `dir` 等のコマンドを入力できる。
5. 入力がターミナルに反映される（キーボードフックが効いている）。
