# Evidence Report - Issue 61: ターミナル二重起動時のクラッシュ調査

## 1. 調査の背景と目的
ターミナル起動中に再度起動ボタンを押すとアプリケーションがクラッシュする問題（Issue 61）の根本原因を特定し、修正のための技術的根拠を提示する。

## 2. 参照リソース (Primary Sources)
- **GitHub Issue 61**: [naoyukik/emeditor-terminal#61](https://github.com/naoyukik/emeditor-terminal/issues/61)
- **EmEditor SDK**: `EE_CUSTOM_BAR_OPEN` メッセージおよびカスタムバー管理仕様
- **Windows API**: `SetFocus`, `CreateWindowExW`, `IsWindow`, `DestroyWindow`
- **Codebase**: `src/gui/window/mod.rs` (`open_custom_bar`), `src/gui/resolver/window_message_resolver.rs` (`on_destroy`)

## 3. 調査結果 (Key Findings)

### 3.1 根本原因の特定（仮説）
コードの静的解析により、以下の 3 つの脆弱性を特定した。

1.  **無効なウィンドウハンドルへのアクセス (Invalid HWND Reference)**:
    `open_custom_bar` は冒頭で `window_handle` をチェックし、存在すれば `SetFocus` を実行する。しかし、ウィンドウが予期せず破棄された場合（または `on_destroy` が呼ばれる前の不完全な状態）に、無効なハンドルに対して `SetFocus` を試行し、OS レベルのエラーまたは例外を引き起こしている可能性がある。

2.  **不完全な初期化状態での排他制御の漏れ**:
    `open_custom_bar` 内で `window_handle` をセットした後、`start_conpty_and_reader_thread` を実行する。この関数が `false` を返した場合（ConPTY 起動失敗）、`open_custom_bar` は `false` を返すが、**`window_handle` に格納されたハンドルは破棄されず、`None` にも戻されない**。この「ゾンビハンドル」が存在する状態で 2 回目のボタンが押されると、無効なハンドルを操作しようとしてクラッシュする。

3.  **マルチウィンドウ環境下での競合 (Global State Conflict)**:
    `get_terminal_data()` は `OnceLock` を使用したグローバルな `Arc<Mutex>` を返すが、これはプロセス全体で 1 つの状態（1 つの HWND）しか管理できないことを意味する。EmEditor で複数のウィンドウ（またはタブ）が開いている場合、別のウィンドウでターミナルを起動しようとすると、既存の（別ウィンドウの）ターミナルハンドルを誤って参照し、不整合が発生する。

### 3.2 アーキテクチャ規約との整合性
- **現状**: `gui/window/mod.rs` が状態管理（`window_handle`）と Win32 操作を混在させている。
- **理想**: ウィンドウの生存確認やハンドルの有効性チェックは `Driver` 層（`ImeGuiDriver` や `KeyboardGuiDriver` と同様の `WindowGuiDriver` 等）に抽出し、`Resolver` はその安全な抽象化のみを扱うべきである。

## 4. 将来の修正で期待される挙動 (Expected Behavior)
本調査結果に基づき、将来的な修正において実現すべき挙動を以下に定義する。

- **二重起動の防止とフォーカス制御**:
    既にターミナルが起動している状態で再度ボタンが押された場合、新たなウィンドウ作成や ConPTY の初期化は行わず、既存のターミナルウィンドウに `SetFocus` を行うことで、ユーザー入力を即座に受け付けられる状態にする。
- **堅牢なクリーンアップと再試行**:
    ConPTY の起動やウィンドウ作成の過程でエラーが発生した場合は、作成途中のリソース（HWND 等）を完全に破棄し、グローバルな状態管理（`window_handle`）を `None` にリセットする。これにより、無効なハンドルを参照してクラッシュすることを防ぎ、ユーザーによる再起動操作を正常に受け付けられるようにする。

## 5. 修正に向けた推奨事項 (Recommendations)
- **安全性強化**: `SetFocus` 前に `IsWindow` 関数によるハンドルの生存確認を必須とする。
- **トランザクション的初期化**: `start_conpty_and_reader_thread` が失敗した場合は、直ちに `DestroyWindow` を呼び出し、`window_handle` を `None` にリセットするガードロジックを実装する。
- **ウィンドウ識別子の導入**: `window_handle` を管理する際に、親ウィンドウ（`hwnd_editor`）の ID と紐付け、別ウィンドウからの誤参照を防ぐ。

## 6. 証拠 (Evidence)
- `src/gui/window/mod.rs` L160-167: ロック取得後の早期リターンにおけるハンドル検証の欠如。
- `src/gui/window/mod.rs` L236: `start_conpty_and_reader_thread` 失敗時のクリーンアップ処理の欠落。
