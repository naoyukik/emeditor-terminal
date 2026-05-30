# Implementation Plan: Issue 202 - get_selected_text の選択判定効率化

## フェーズ 1: 問題の把握と詳細設計

- [ ] Task: `autonomous-researcher` による詳細調査と `evidence_report.md` の作成
    - [ ] GitHub Issue #202 の背景、目的、成功条件を整理する。
    - [ ] `src/domain/model/terminal_buffer_entity.rs` の `get_selected_text()` を確認し、現在の `selection_contains()` 呼び出し箇所と反復構造を記録する。
    - [ ] `src/domain/model/terminal_buffer_view_entity.rs` の `normalized_selection_range()` / `selection_contains()` の契約を確認する。
    - [ ] `sdk/` 配下の EmEditor SDK 選択 API 情報を確認し、本件がプラグイン独自バッファ選択ロジックの改善であることを記録する。
    - [ ] Microsoft Learn の Windows Terminal 選択・コピー関連情報を確認し、選択結果の意味を変えない方針を記録する。
- [ ] Task: 調査結果に基づいた `plan.md` の以降のタスク具体化
    - [ ] 対象ファイル、変更対象関数、追加または更新するテストケースを行単位で具体化する。
    - [ ] `selection_contains()` をセル単位で呼ばず、論理行ごとに `start_x` / `end_x` を算出する方針を明文化する。
- [ ] Task: Conductor - Clippy & fmt Check。Clippyはfixを使用すること。&&は使えないので個別に実行すること。
    - [ ] `cargo clippy --fix --allow-dirty` を実行し、必要な修正を確認する。
    - [ ] `cargo fmt` を実行する。
- [ ] Task: Conductor - `.\install.ps1 -Path "$env:PLUGINS_DIR"` を実行
- [ ] Task: Conductor - User Manual Verification 'フェーズ 1' (Protocol in workflow.md)
- [ ] Task: Conductor - 'フェーズ 1' の成果をコミット

## フェーズ 2: Domain テストによる既存挙動の固定

- [ ] Task: `TerminalBufferEntity::get_selected_text()` の既存挙動を固定する単体テストを追加または更新する
    - [ ] 同一行選択で選択範囲の文字列が維持されることを確認する。
    - [ ] 複数行選択で行間に `\n` が挿入されることを確認する。
    - [ ] スクロールバックを含む logical row 選択で、viewport 移動後も同じ選択文字列を返すことを確認する。
    - [ ] ワイド文字継続セル `is_wide_continuation == true` が抽出結果に含まれないことを確認する。
- [ ] Task: 変更前テストを実行し、対象テストが現行挙動を正しく表現していることを確認する
    - [ ] `cargo test terminal_buffer` もしくは該当モジュールの単体テストを実行する。
- [ ] Task: Conductor - Clippy & fmt Check。Clippyはfixを使用すること。&&は使えないので個別に実行すること。
    - [ ] `cargo clippy --fix --allow-dirty` を実行し、必要な修正を確認する。
    - [ ] `cargo fmt` を実行する。
- [ ] Task: Conductor - `.\install.ps1 -Path "$env:PLUGINS_DIR"` を実行
- [ ] Task: Conductor - User Manual Verification 'フェーズ 2' (Protocol in workflow.md)
- [ ] Task: Conductor - 'フェーズ 2' の成果をコミット

## フェーズ 3: `get_selected_text()` の効率化実装

- [ ] Task: `get_selected_text()` の抽出範囲算出を行単位へ変更する
    - [ ] `normalized_selection_range(self.selection_range)` で得た `start` / `end` を使用する。
    - [ ] 各 `logical_row` について、開始行なら `start.x`、それ以外なら `0` を抽出開始列とする。
    - [ ] 終了行なら `end.x`、それ以外なら `self.width.saturating_sub(1)` を抽出終了列とする。
    - [ ] 算出した列範囲のみを走査し、`cell.is_wide_continuation == false` のセルだけを `selected_text` に追加する。
    - [ ] `get_selected_text()` 内のセル単位 `selection_contains()` 呼び出しを削除する。
- [ ] Task: 既存 API 境界を維持する
    - [ ] `SelectionRange` / `SelectionPoint` の型定義は変更しない。
    - [ ] Windows API 型、EmEditor SDK 型、GUI 層の型を Domain 層へ持ち込まない。
    - [ ] GUI ハイライト側の `selection_contains()` 利用は本トラックでは変更しない。
- [ ] Task: Conductor - Clippy & fmt Check。Clippyはfixを使用すること。&&は使えないので個別に実行すること。
    - [ ] `cargo clippy --fix --allow-dirty` を実行し、必要な修正を確認する。
    - [ ] `cargo fmt` を実行する。
- [ ] Task: Conductor - `.\install.ps1 -Path "$env:PLUGINS_DIR"` を実行
- [ ] Task: Conductor - User Manual Verification 'フェーズ 3' (Protocol in workflow.md)
- [ ] Task: Conductor - 'フェーズ 3' の成果をコミット

## フェーズ 4: 総合検証と完了整理

- [ ] Task: 全体テストを実行する
    - [ ] `cargo test` を実行する。
    - [ ] 選択抽出関連テストがすべて通ることを確認する。
- [ ] Task: 簡易性能確認として差分レビューを行う
    - [ ] `get_selected_text()` のセル走査中に `selection_contains()` が呼ばれていないことを確認する。
    - [ ] 行ごとの範囲算出により、選択範囲外セルの不要な包含判定がなくなっていることを確認する。
- [ ] Task: `evidence_report.md` を最終更新する
    - [ ] 実装内容、検証結果、未実施の本格ベンチマークが Out of Scope であることを記録する。
- [ ] Task: Conductor - Clippy & fmt Check。Clippyはfixを使用すること。&&は使えないので個別に実行すること。
    - [ ] `cargo clippy --fix --allow-dirty` を実行し、必要な修正を確認する。
    - [ ] `cargo fmt` を実行する。
- [ ] Task: Conductor - `.\install.ps1 -Path "$env:PLUGINS_DIR"` を実行
- [ ] Task: Conductor - User Manual Verification 'フェーズ 4' (Protocol in workflow.md)
- [ ] Task: Conductor - 'フェーズ 4' の成果をコミット
