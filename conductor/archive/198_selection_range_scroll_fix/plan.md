# 実装計画 (Implementation Plan): selection_range の選択開始点がスクロールで移動する不具合修正 (Issue 198)

## フェーズ 1: 調査と設計確定
- [x] Task: `autonomous-researcher` により `selection_range`・`viewport_offset`・選択描画経路を調査し、`evidence_report.md` を作成する。
- [x] Task: 調査結果に基づき、`selection_range` の論理座標モデル（history を含む行座標）と変換責務を `plan.md` に具体化する。
- [x] Task: Conductor - User Manual Verification 'フェーズ 1' (調査結果と具体的プランの承認)
- [x] Task: Conductor - Clippy & fmt Check。Clippyはfixを使用すること。&&は使えないので個別に実行すること。
- [x] Task: Conductor - `.\install.ps1 -Path "$env:PLUGINS_DIR"` を実行
- [x] Task: Conductor - 'フェーズ 1' の成果をコミット

## フェーズ 2: 実装（論理座標モデル置換）
- [x] Task: `SelectionRange` を論理座標ベースへ変更し、`TerminalBufferEntity` の保持/取得 API を更新する。
- [x] Task: `TerminalWorkflow` のマウス選択更新（Down/Drag/Up）を論理座標で保存する実装へ変更する。
- [x] Task: `get_selected_text` を新座標モデルで動作させ、history-screen 跨ぎの選択抽出を保証する。
- [x] Task: `terminal_gui_driver` の選択判定を visual row 直接比較から logical row 解決ベースへ変更する。
- [ ] Task: Conductor - User Manual Verification 'フェーズ 2' (実装結果の確認)
- [x] Task: Conductor - Clippy & fmt Check。Clippyはfixを使用すること。&&は使えないので個別に実行すること。
- [ ] Task: Conductor - `.\install.ps1 -Path "$env:PLUGINS_DIR"` を実行
- [ ] Task: Conductor - 'フェーズ 2' の成果をコミット

## フェーズ 2R: 手動検証不合格後の再調査と修正
- [x] Task: 現在実装の座標変換を再検証する.
  - 対象: `src/domain/model/terminal_history_view_entity.rs`
  - 対象: `src/domain/model/terminal_buffer_entity.rs`
  - 作業: `TerminalHistoryViewEntity::resolve_visual_row` と `TerminalBufferEntity::visual_row_to_logical_row` が同じ viewport 解釈を共有していることをテストで固定する。
  - 作業: `viewport_offset=0`、`viewport_offset>0`、history-screen 跨ぎの visual row について、`get_line_at_visual_row(visual_row)` と `get_line_at_logical_row(visual_row_to_logical_row(visual_row))` が同じ行を返すことを検証する。
- [x] Task: 選択開始点の不変条件を domain テストで追加する.
  - 対象: `src/domain/model/terminal_buffer_entity.rs`
  - 作業: `SelectionPoint { x, logical_row }` を設定後に `scroll_lines` / `scroll_to` を実行しても `get_selection_range()` の start/end が変化しないことを検証する。
  - 作業: 選択済み論理行が viewport 外へ出た場合でも `get_selected_text()` が同じ文字列を返すことを検証する。
  - 作業: viewport を戻した時に同じ logical row のみが選択対象になることを検証する。
- [x] Task: マウス入力経路のスクロール中選択テストを追加する.
  - 対象: `src/application/terminal_workflow.rs`
  - 作業: 左 Down で start を保存後、`scroll_lines` してから Drag しても start が再計算されないことを検証する。
  - 作業: `viewport_offset>0` の状態で左 Down した場合、start が history 側の正しい logical row になることを検証する。
  - 作業: 逆方向 Drag とスクロールを組み合わせても正規化後の選択文字列が期待値になることを検証する。
- [x] Task: 描画判定の責務を domain に寄せ、GUI 側の座標解釈重複を減らす。
  - 対象: `src/domain/model/terminal_buffer_view_entity.rs`
  - 作業: `SelectionRange` を単なる型 alias から、正規化と `contains(logical_row, x)` を持つ値オブジェクトへ変更するか、同等の domain 関数を追加する。
  - 対象: `src/gui/driver/terminal_gui_driver.rs`
  - 作業: `is_in_selection` の private ロジックを domain の選択判定へ置き換え、描画側では `visual_row -> logical_row` の解決と domain 判定の呼び出しだけにする。
  - 作業: GUI の run 分割時に、同一 visual row 内で logical row を再計算せず、行単位で確定した logical row を使い続ける。
- [x] Task: 再現シナリオを実機ログ付きで確認する。
  - 対象: `$env:TEMP\emeditor_terminal.log`
  - 作業: 選択 Down、Drag、`WM_MOUSEWHEEL` / `WM_VSCROLL`、再描画時の `viewport_offset`、visual row、logical row、selection start/end を一時ログとして出力し、開始点ずれが「保存値の変化」か「描画判定の誤り」かを切り分ける。
  - 作業: 切り分け後、一時ログは削除する。恒久ログを残す場合は debug レベルに限定する。
- [x] Task: 修正実装は不要と判断し、見送りとする。
  - 対象: `src/domain/model/terminal_buffer_entity.rs`
  - 作業: 実機確認の結果、スクロール追従に問題は再現しなかったため、追加修正は行わない。
  - 対象: `src/application/terminal_workflow.rs`
  - 作業: 一時ログの確認後、選択開始点の保持挙動は既存実装のままで問題ないと確認した。
  - 対象: `src/gui/driver/terminal_gui_driver.rs`
  - 作業: 描画経路もスクロール追従に追随しており、修正は不要と判断した。
- [ ] Task: Conductor - Clippy & fmt Check。Clippyはfixを使用すること。&&は使えないので個別に実行すること。
- [x] Task: Conductor - `.\\install.ps1 -Path \"$env:PLUGINS_DIR\"` を実行
- [x] Task: Conductor - User Manual Verification 'フェーズ 2R' (開始点ずれ再現ケースが解消したことの確認)
- [ ] Task: Conductor - 'フェーズ 2R' の成果をコミット

## フェーズ 3: テストと統合検証
- [ ] Task: 回帰テストを追加する（選択後スクロール、逆方向スクロール、履歴跨ぎ、逆方向ドラッグ）。
  - 現在のブランチ差分では、`SelectionPoint` 型への追従テストはあるが、選択後スクロールで start/end が不変であることを直接検証するテストが不足している。
- [ ] Task: 既存テストを含めて実行し、選択・スクロール・コピーの一貫性を確認する。
  - `cargo test` だけでなく、フェーズ 2R で追加する viewport 変換・選択不変条件テストの失敗再現と成功を確認する。
- [ ] Task: Conductor - User Manual Verification 'フェーズ 3' (最終動作確認)
- [ ] Task: Conductor - Clippy & fmt Check。Clippyはfixを使用すること。&&は使えないので個別に実行すること。
- [ ] Task: Conductor - `.\install.ps1 -Path "$env:PLUGINS_DIR"` を実行
- [ ] Task: Conductor - 'フェーズ 3' の成果をコミット

## Phase: Review Fixes
- [x] Task: Apply review suggestions 9e97315
- [x] Task: PR 201 のレビューコメントを Future Ticket として記録する。
  - 対象: `src/domain/model/terminal_buffer_entity.rs`
  - 内容: `get_selected_text()` 内の `selection_contains()` 反復呼び出しを削減し、正規化済みの選択範囲を直接走査する最適化は今回の PR では実装しない。
  - Issue: #202
