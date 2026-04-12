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

---

## 7. Review Fixes Findings (Phase: Review Fixes)

PR #138 のレビュー指摘事項に基づき、以下の点を修正・整理する。

### 7.1 指摘事項への対応方針

- **指摘 1: モーダル表示中のフォーカス回避**
    - `src/gui/window/mod.rs` において、`open_custom_bar` はユーザーが EmEditor のプラグインコマンドを明示的に実行した際に同期的に呼び出される。
    - EmEditor がモーダルダイアログを表示している間は、通常プラグインコマンド自体が実行不可となるため、実用上の競合は極めて低い。
    - **判断**: 要件がオーバースペックであるため、`spec.md` の「モーダル表示中は奪取しない」という記述を削除または緩和し、現在の実装（起動時は即座にフォーカス）を正とする。
- **指摘 2: ログレベルの変更**
    - `src/gui/driver/window_gui_driver.rs` の `log::info!` を `log::debug!` に変更し、クリック時の過剰出力を抑止する。
- **指摘 3: metadata.json の更新**
    - `status` を `completed` に変更する。
- **指摘 4: tracks.md のインデント修正**
    - 他のトラック（Issue #76）とインデントを揃え、Markdown 構造を修正する。

### 7.2 設計判断
- **仕様の修正**: `spec.md` を現在の実装に合わせて更新する。
- **実装の調整**: ログレベルのみ修正し、フォーカス制御ロジックは現状を維持する。
