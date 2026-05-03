# 実装計画: シェル上でのマウス操作のサポート (Issue #178)

## フェーズ 1: 問題の把握と詳細設計 (Discovery & Detailed Design) [Done]
- [x] Task: `autonomous-researcher` による詳細調査と `evidence_report.md` の作成
- [x] Task: 調査結果に基づいた `plan.md` の以降のタスクの具体化。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 1' (Protocol in workflow.md)

## フェーズ 2: クリップボード・リポジトリの導入と貼り付けの実装 [Done]
- [x] Task: ドメイン層の定義
    - [x] Action: `src/domain/repository/clipboard_repository.rs` を作成し、`ClipboardRepository` トレイトを定義。
- [x] Task: インフラ層の実装
    - [x] Action: `src/infra/repository/windows_clipboard_repository_impl.rs` を作成し、Win32 API を使用してクリップボード文字列を取得するロジックを実装。
- [x] Task: アプリケーション層への注入
    - [x] Action: `TerminalWorkflow` に `ClipboardRepository` を保持させ、`new` メソッドで注入するように変更。
    - [x] Action: `src/gui/window/mod.rs` での `TerminalWorkflow` 生成処理を更新。
- [x] Task: 右クリック貼り付けの実装
    - [x] Action: `src/gui/resolver/window_message_resolver.rs` の `dispatch_mouse_event` から Shift バイパス制限を削除。
    - [x] Action: `TerminalWorkflow::handle_mouse_event` にて、`mode == None` 且つ右クリック時にクリップボード内容を `send_input` する処理を追加。
- [x] Task: 動作確認とテスト
    - [x] Action: クリップボード取得および貼り付けロジックのユニットテスト (Mock使用) を追加。
    - [x] Action: `cargo clippy`, `cargo fmt` を実行。
- [x] Task: コミット
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 2' (Protocol in workflow.md)

## フェーズ 3: 左クリックによるカーソル移動機能の実装 [Done]
- [x] Task: カーソル移動ロジックの実装
    - [x] Action: `TerminalWorkflow::handle_mouse_event` にて、`mode == None` 且つ左クリック時に現在カーソルと同一行であれば、水平距離分の矢印キーシーケンスを `send_input` する処理を追加。
- [x] Task: 動作確認とテスト
    - [x] Action: 距離計算とシーケンス生成のテストを追加。
    - [x] Action: `cargo clippy`, `cargo fmt` を実行。
- [x] Task: コミット
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 3' (Protocol in workflow.md)

## フェーズ 4: テキスト選択機能の調査 (Discovery - Selection) [Done]
- [x] Task: `autonomous-researcher` による詳細調査と `evidence_report.md` の追記
- [x] Task: 調査結果に基づいた `plan.md` の以降のタスクの具体化。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 4' (Protocol in workflow.md)

## フェーズ 5: テキスト選択機能の実装 (Implementation - Selection) [Done]
- [x] Task: 選択状態の管理ロジック実装
    - [x] Action: `TerminalBufferEntity` もしくは `TerminalWorkflow` に選択範囲（開始座標・終了座標）を保持する変数を追加。
- [x] Task: ドラッグイベントの処理
    - [x] Action: `dispatch_mouse_event` でドラッグ開始・継続・終了を正しく判定し、選択範囲を更新する。
- [x] Task: 描画反映
    - [x] Action: `TerminalGuiDriver::render` にて、選択範囲に含まれるセルの背景色を変更する（反転表示等）。
- [x] Task: コード品質の確保とテスト
    - [x] Action: 選択座標計算ロジックのテスト追加。
    - [x] Action: `cargo clippy`, `cargo fmt` を実行。
- [x] Task: コミット
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 5' (Protocol in workflow.md)

## フェーズ 6: 選択範囲のコピー機能の調査 (Discovery - Copy) [Done]
- [x] Task: `autonomous-researcher` による詳細調査と `evidence_report.md` の追記
- [x] Task: 調査結果に基づいた `plan.md` の以降のタスクの具体化。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 6' (Protocol in workflow.md)

## フェーズ 7: 選択範囲のコピー機能の実装 (Implementation - Copy) [Done]
- [x] Task: クリップボード・リポジトリの拡張
    - [x] Action: `ClipboardRepository` トレイトに `set_text` メソッドを追加し、`WindowsClipboardRepositoryImpl` で実装。
- [x] Task: テキスト抽出ロジックの実装
    - [x] Action: `TerminalBufferEntity` または `TerminalWorkflow` に、選択範囲の座標から文字列を生成するメソッドを実装。
- [x] Task: 右クリックイベントの拡張
    - [x] Action: `TerminalWorkflow::handle_mouse_event` にて、選択範囲が存在する場合の右クリックでコピー処理を実行し、選択を解除するロジックを追加。
- [x] Task: コード品質の確保とテスト
    - [x] Action: テキスト抽出およびコピー動作のテスト追加。
    - [x] Action: `cargo clippy`, `cargo fmt` を実行。
- [ ] Task: コミット
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 7' (Protocol in workflow.md)

## フェーズ 8: Ctrl+C コピー機能の調査と実装 [Done]
- [x] Task: キー入力処理経路の特定
    - [x] Action: `KeyboardGuiDriver` (IO経由) から `TerminalWorkflow::handle_key_event` を呼び出すように変更。
- [x] Task: コピーロジックの統合
    - [x] Action: `TerminalWorkflow` にて、選択範囲あり + Ctrl+C 押下時にクリップボードへコピーし、イベントを消費する処理を追加。
- [x] Task: コード品質の確保とテスト
    - [x] Action: Ctrl+C 押下時の挙動（コピー vs 信号送信）のユニットテストを追加。
    - [x] Action: `cargo clippy`, `cargo fmt` を実行。
- [ ] Task: コミット
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 8' (Protocol in workflow.md)

## フェーズ 9: Ctrl+V 高速貼り付けの実装 [Done]
- [x] Task: Ctrl+V のインターセプト
    - [x] Action: `TerminalWorkflow::handle_key_event` にて Ctrl+V (0x56) を検知し、直接クリップボードから貼り付けるロジックを追加。
- [x] Task: コード品質の確保とテスト
    - [x] Action: Ctrl+V による貼り付け動作のユニットテストを追加。
    - [x] Action: `cargo clippy`, `cargo fmt` を実行。
- [ ] Task: コミット
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 9' (Protocol in workflow.md)
