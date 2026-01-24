# Implementation Plan - Alt Key Support for TUI

TUI アプリケーションでの Alt キー操作をサポートするため、ウィンドウプロシージャでのメッセージハンドリングを実装し、EmEditor への伝播を抑制しつつ ConPTY へ入力を転送します。

## Phase 1: 調査とメッセージハンドリングのプロトタイプ
ターミナルウィンドウの `WndProc` で `WM_SYSKEYDOWN` / `WM_SYSKEYUP` を捕捉し、ログ出力による動作確認を行います。

- [x] Task: 現行の `WndProc` 実装の確認 (`src/gui/renderer.rs` または関連するメッセージループ)
- [x] Task: `WM_SYSKEYDOWN` / `WM_SYSKEYUP` メッセージのハンドリング追加とデバッグログの実装
- [x] Task: Alt キー単独押下時の `DefWindowProc` 呼び出し抑制による EmEditor メニューアクティブ化の阻止の検証
- [x] Task: `clippy` による静的解析と修正
- [x] Task: Conductor - User Manual Verification 'Phase 1: メッセージ捕捉の確認' (Protocol in workflow.md)

## Phase 2: ConPTY への Alt キー入力転送の実装
捕捉した Alt キーイベントを ConPTY が理解できる入力シーケンスに変換し、送信します。

- [x] Task: `VK_MENU` および `Alt + Key` の組み合わせを ConPTY の入力形式へ変換するロジックの実装
- [x] Task: 除外対象ショートカット (`Alt+Tab`, `Alt+F4` 等) の判定とスルー処理の実装
- [x] Task: `src/infra/conpty.rs` への入力送信処理の統合
- [x] Task: `clippy` による静的解析と修正
- [x] Task: Conductor - User Manual Verification 'Phase 2: TUIアプリでの操作確認' (Protocol in workflow.md)

## Phase 3: エッジケースの対応とリファクタリング
リピート入力や IME 競合などのエッジケースを修正し、コードの品質を整えます。

- [x] Task: Alt キーの長押し（リピート入力）への対応確認と修正
- [x] Task: ターミナルからフォーカスが外れた際の挙動（EmEditor メニューの復帰）の確認
- [x] Task: `clippy` による静的解析とコードフォーマットの適用
- [x] Task: Conductor - User Manual Verification 'Phase 3: 最終動作確認とエッジケース検証' (Protocol in workflow.md)
