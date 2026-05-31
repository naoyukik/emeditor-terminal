# Implementation Plan: Issue 202 - get_selected_text の選択判定効率化

## フェーズ 1: 問題の把握と詳細設計
- [x] Task: `autonomous-researcher` による詳細調査と `evidence_report.md` の作成
- [x] Task: 調査結果に基づいた `plan.md` の以降のタスク具体化
    - [x] `src/domain/model/terminal_buffer_entity.rs` の `get_selected_text` の変更方針を確定。
    - [x] 追加するテストケースを特定。
- [x] Task: Conductor - Clippy & fmt Check
- [x] Task: Conductor - User Manual Verification 'フェーズ 1'
- [x] Task: Conductor - 'フェーズ 1' の成果をコミット

## フェーズ 2: Domain テストによる既存挙動の固定
- [x] Task: `TerminalBufferEntity::get_selected_text()` の既存挙動を固定する単体テストを追加または更新する
    - [x] `src/domain/model/terminal_buffer_entity.rs` の末尾にテストを追加。
    - [x] テスト内容: 単一行選択、複数行選択、空の選択、ワイド文字（継続セルスキップ）の検証。
- [x] Task: 変更前テストを実行し、現行挙動を正しく表現していることを確認する
- [x] Task: Conductor - Clippy & fmt Check
- [x] Task: Conductor - User Manual Verification 'フェーズ 2'
- [x] Task: Conductor - 'フェーズ 2' の成果をコミット

## フェーズ 3: `get_selected_text()` の効率化実装
- [x] Task: `get_selected_text()` の抽出範囲算出を行単位へ変更する
    - [x] `src/domain/model/terminal_buffer_entity.rs` の `get_selected_text` を修正。
- [x] Task: Conductor - Clippy & fmt Check
- [x] Task: Conductor - User Manual Verification 'フェーズ 3'
- [x] Task: Conductor - 'フェーズ 3' の成果をコミット

## フェーズ 4: 総合検証と完了処理
- [x] Task: 全体テストを実行する (`cargo test`)
- [x] Task: `evidence_report.md` を最終更新する
- [x] Task: Conductor - Clippy & fmt Check
- [ ] Task: Conductor - User Manual Verification 'フェーズ 4'
- [ ] Task: Conductor - 'フェーズ 4' の成果をコミット
