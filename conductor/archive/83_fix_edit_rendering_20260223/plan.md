# Implementation Plan - Track 83: fix: edit等のアプリケーションにおける表示ずれの修正

## Phase 1: 精密なログ収集と現状解析 (Detailed Investigation)
- [x] Task: 全ての制御シーケンスとバッファ状態（座標、領域）を記録する詳細ロギングの実装
- [x] Task: `edit` で `LICENSE` および `README.md` を操作し、不具合発生時のログを採取
- [x] Task: chore(conductor): Phase 1: 調査用ロギングの導入をコミット
- [x] Task: ログを分析し、座標ズレの起点となるシーケンスと、文字幅判定の齟齬を特定
- [x] Task: chore(conductor): Phase 1: 調査完了（真因特定）のコミット
- [x] Task: Conductor - User Manual Verification 'Phase 1: 真因特定完了' (Protocol in workflow.md)

## Phase 2: バッファ管理とシーケンス処理の修正 (Core Logic Fix)
- [x] Task: 特定された不足シーケンス（`SU`, `SD`, `RI`, `DECOM` 等）の実装（物理グリッドバッファベース）
- [x] Task: 罫線文字の表示幅を 1 カラムに修正し、TUI レイアウトの整合性を確保
- [x] Task: BCE (Background Color Erase) の実装（現在の背景色で塗りつぶし）
- [x] Task: ターミナルサイズ（行数）の同期不備の解消
- [x] Task: UTF-8 バイトバッファによる文字泣き別れの根本解決
- [x] Task: 既存のユニットテストの修正と、原点モード等のテスト追加
- [x] Task: fix: edit等のアプリケーションにおける表示ずれの修正（ロジック修正）
- [x] Task: chore(conductor): Phase 2: 修正完了のコミット
- [x] Task: Conductor - User Manual Verification 'Phase 2: ロジック修正完了' (Protocol in workflow.md)

## Phase 3: 最終検証と品質向上 (Final Verification & Polishing)
- [x] Task: `edit` における正常動作（スクロール、メニュー、罫線）の確認
- [x] Task: 他の TUI アプリでのデグレード確認
- [x] Task: 不要なログ出力の削除とコードクリーンアップ
- [x] Task: style: 表示ずれ修正に伴うコードのクリーンアップ
- [x] Task: chore(conductor): Phase 3: 全工程完了のコミット
- [x] Task: Conductor - User Manual Verification 'Phase 3: 全工程完了' (Protocol in workflow.md)
