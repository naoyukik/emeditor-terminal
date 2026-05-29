# Evidence Report: `selection_range` の選択開始点がスクロールで移動する不具合 (Issue 198)

## Discovery Summary
- 問題は、選択範囲の保持単位が `visual row` になっているため、viewport 変更時に同じ論理位置が別の画面位置へ見えてしまう点にある。
- 成功条件は、選択中にスクロールしても開始点・終了点の論理位置が不変であること、コピー結果が表示と一致すること、逆方向スクロールや逆方向ドラッグでも破綻しないことである。
- スコープは選択座標モデルの修正に限定し、Win32 メッセージ経路やバッファ責務の広範な再分割は対象外とする。

## Codebase Findings
- [src/domain/model/terminal_buffer_view_entity.rs](/Z:/projects/emeditor-terminal/src/domain/model/terminal_buffer_view_entity.rs:1) で `SelectionRange` が旧式の 2 点タプルとして定義されていた。
- [src/application/terminal_workflow.rs](/Z:/projects/emeditor-terminal/src/application/terminal_workflow.rs:230) で左クリック Down/Drag の座標を `event.y` のまま保存していた。
- [src/domain/model/terminal_buffer_entity.rs](/Z:/projects/emeditor-terminal/src/domain/model/terminal_buffer_entity.rs:508) で `get_selected_text` が `visual row` 前提で抽出していた。
- [src/gui/driver/terminal_gui_driver.rs](/Z:/projects/emeditor-terminal/src/gui/driver/terminal_gui_driver.rs:430) で選択ハイライトが `visual row` 直接比較だった。
- `TerminalBufferEntity` には既に `history` と `scrollback` があり、`visual_row_to_logical_row` と `get_line_at_logical_row` を追加すれば、選択座標を論理行基準へ統一できる。

## Clarifying Questions
- 未解決のユーザー質問はなし。Issue 198 の記述で必要十分な前提が得られている。

## Architecture Options
- Minimal Changes: `SelectionRange` だけを論理座標へ変え、既存 API を最小限変更する。差分は小さいが、座標意味の曖昧さが残りやすい。
- Clean Architecture: 論理座標の value object を導入し、Domain/Application/GUI の責務を明示する。保守性は高いが、変更範囲が広い。
- Pragmatic Balance: `SelectionPoint { x, logical_row }` を導入し、保持・抽出・描画の 3 箇所だけを論理座標化する。今回の不具合修正としては最も妥当で、回帰も抑えやすい。
- 推奨案は Pragmatic Balance である。#198 は bugfix であり、過剰設計は避けるべきだが、座標意味の混在は明確に排除する必要がある。

## Sequential Thinking Checkpoints Summary
- Phase 1 完了時点で、問題は「座標の保持単位が visual row であること」に収束した。
- 未確定だったのは保持責務の置き場所と描画判定の責務境界だが、既存構造では Domain に論理変換を寄せるのが最も自然である。
- 次に掘るべき論点は、`SelectionRange` の型変更と `TerminalWorkflow` の入力変換、`TerminalGuiDriver` の判定基準の3点である。

## Expected Behavior
- 選択後に上下スクロールしても、開始点・終了点は論理位置として不変である。
- history のみ、screen のみ、history-screen 跨ぎのいずれでもコピー結果が表示と一致する。
- 逆方向スクロールや逆方向ドラッグでも、選択範囲の正規化結果は安定している。

## Evidence
- Learn Microsoft: `WM_MOUSEWHEEL` はフォーカスウィンドウへ送られ、`lParam` はスクリーン座標であり、`GET_X_LPARAM` / `GET_Y_LPARAM` 相当で扱うのが正道である。選択座標モデルの変更はこの経路に影響しない。
- Learn Microsoft: `WM_MOUSEHWHEEL` も同様にアクティブウィンドウへ送られ、`lParam` の座標意味は縦スクロールと同じである。
- EmEditor SDK: custom bar は `EE_CUSTOM_BAR_OPEN` / `EE_CUSTOM_BAR_CLOSE` で管理されるが、本件は UI コンテナのライフサイクル変更を伴わない。
- 調査日: 2026-05-26

## Implementation Guidance
- `SelectionRange` を `SelectionPoint { x, logical_row }` に置換する。
- `TerminalWorkflow` で mouse Down/Drag 時に `visual_row_to_logical_row` を通して保存する。
- `TerminalBufferEntity::get_selected_text` を `logical_row` ベースへ変更する。
- `TerminalGuiDriver` の選択判定を logical row 基準へ切り替える。
