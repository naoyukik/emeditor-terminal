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

## フェーズ 3: テストと統合検証
- [x] Task: 回帰テストを追加する（選択後スクロール、逆方向スクロール、履歴跨ぎ、逆方向ドラッグ）。
- [x] Task: 既存テストを含めて実行し、選択・スクロール・コピーの一貫性を確認する。
- [ ] Task: Conductor - User Manual Verification 'フェーズ 3' (最終動作確認)
- [x] Task: Conductor - Clippy & fmt Check。Clippyはfixを使用すること。&&は使えないので個別に実行すること。
- [ ] Task: Conductor - `.\install.ps1 -Path "$env:PLUGINS_DIR"` を実行
- [ ] Task: Conductor - 'フェーズ 3' の成果をコミット
