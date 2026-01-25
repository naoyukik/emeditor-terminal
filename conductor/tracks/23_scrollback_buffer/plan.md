# Track: 23_scrollback_buffer 実装計画

## フェーズ 0: リファクタリング (Parser Separation)
肥大化した `TerminalBuffer` から ANSI 解析ロジックを分離し、責務を明確化する。

- [x] Task: `src/domain/parser.rs` の作成と `AnsiParser` 構造体の定義
- [x] Task: `TerminalBuffer` から解析ロジック（`write_string`, `handle_csi` 等）を `AnsiParser` へ移動
- [x] Task: `TerminalBuffer` と `AnsiParser` の連携実装（`TerminalBuffer` を操作するトレイトまたは直接呼び出し）
- [x] Task: 既存テストの移行と動作確認
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 0: リファクタリング' (Protocol in workflow.md)

## フェーズ 0.5: アーキテクチャ強化 (Application Layer)
GUI層とドメイン層の結合度を下げるため、アプリケーション層を導入する。

- [x] Task: `src/application/mod.rs` の作成と `TerminalService` の定義（`service.rs` へ分離）
- [x] Task: `TerminalService` への `TerminalBuffer`, `AnsiParser`, `ConPTY` の集約
- [x] Task: `src/gui/custom_bar.rs` のリファクタリング（`TerminalData` を `TerminalService` のラッパーに変更）
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 0.5: アーキテクチャ強化' (Protocol in workflow.md)

## フェーズ 1: ドメイン層の拡張 (TerminalBuffer)
ヒストリー保持とビューポート制御のロジックを `TerminalBuffer` に実装する。

- [ ] Task: `TerminalBuffer` 構造体へのフィールド追加 (`history`, `viewport_offset`, `scrollback_limit`)
- [ ] Task: `scroll_up` メソッドの修正（押し出された行を `history` へ移動）
- [ ] Task: ビューポート制御メソッドの実装 (`scroll_to`, `get_visible_lines`, `reset_viewport`)
- [ ] Task: 入力時オートスクロール・出力時追従ロジックの実装
- [ ] Task: ユニットテストの追加と既存テストの修正
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 1: ドメイン層の拡張' (Protocol in workflow.md)

## フェーズ 2: GUI層の拡張 (Scrollbar & Message Handling)
Win32 スクロールバーの制御とメッセージ処理を `custom_bar.rs` に実装する。

- [ ] Task: `SCROLLINFO` の更新ロジック実装（バッファサイズに基づくリアルタイム同期）
- [ ] Task: `WM_VSCROLL` メッセージハンドラの追加（スクロールバー操作の処理）
- [ ] Task: `WM_MOUSEWHEEL` メッセージハンドラの追加（マウスホイール操作の処理）
- [ ] Task: `WM_KEYDOWN` へのオートスクロールトリガー追加
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 2: GUI層の拡張' (Protocol in workflow.md)

## フェーズ 3: レンダリングの調整 (Renderer)
ビューポートに基づいて描画内容を切り替えるよう `TerminalRenderer` を修正する。

- [ ] Task: `TerminalRenderer::render` の引数拡張（`viewport_offset` への対応）
- [ ] Task: ヒストリー領域の描画ロジックの実装
- [ ] Task: 全体的な動作確認とパフォーマンス調整
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 3: レンダリングの調整' (Protocol in workflow.md)
