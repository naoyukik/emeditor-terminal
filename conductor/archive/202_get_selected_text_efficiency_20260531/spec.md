# Specification: Issue 202 - get_selected_text の選択判定効率化

## Overview

GitHub Issue #202 に基づき、`TerminalBufferEntity::get_selected_text()` の選択抽出ロジックを効率化する。

現在の実装は `normalized_selection_range()` で正規化済みの開始点・終了点を取得しているにもかかわらず、各論理行の全セルに対して `selection_contains()` を繰り返し呼び出している。これは機能不具合ではないが、広い選択範囲では不要な判定コストを生む。

本トラックでは Domain 層に限定し、正規化済み選択範囲から各行の抽出開始列・終了列を直接決定する実装へ変更する。

## Background

PR #201 のレビューで、`get_selected_text()` が選択範囲の幅分だけ `selection_contains()` を呼び出している点が指摘された。既に `normalized_selection_range()` により `start` / `end` は順方向に正規化されているため、抽出対象の論理行ごとに列範囲を算出すれば、セル単位で範囲包含判定を繰り返す必要はない。

Microsoft Learn の Windows Terminal 選択関連ドキュメントでは、ターミナル選択は複数行のテキストコピー、マウス選択、クリップボード連携を前提として扱われている。今回の変更は OS API 連携ではなく Domain 層の抽出ロジック改善であり、選択結果の意味を変更しない。

EmEditor SDK 側には `Editor_GetSelTextW`, `Editor_GetSelStart`, `Editor_GetSelEnd`, `Editor_GetSelType` などの選択取得 API が存在するが、本プラグインのターミナルバッファ選択は独自の `SelectionRange` と logical row に基づいて管理されるため、SDK API の直接変更は対象外とする。

## Functional Requirements

- `TerminalBufferEntity::get_selected_text()` は `normalized_selection_range()` で得た `start` / `end` を直接使用して選択テキストを抽出する。
- `get_selected_text()` 内で、選択範囲内かどうかを判定するための `selection_contains()` のセル単位反復呼び出しを削減する。
- 選択範囲が複数行にまたがる場合、開始行・中間行・終了行それぞれの抽出列範囲を正しく算出する。
- ワイド文字の継続セル、すなわち `cell.is_wide_continuation == true` のセルは、既存どおり抽出結果に含めない。
- スクロールバックを含む logical row ベースの抽出挙動を維持する。
- 同一行選択、複数行選択、スクロール後の選択抽出に対する既存挙動を維持する。

## Non-Functional Requirements

- 変更範囲は Domain 層の `TerminalBufferEntity::get_selected_text()` と関連単体テストに限定する。
- Windows API 型や EmEditor SDK 型を Domain 層へ持ち込まない。
- Strict Rigid レイヤードアーキテクチャの依存方向を維持する。
- 実装は可読性を損なわず、行ごとの抽出範囲算出が追跡しやすい構造にする。
- 性能確認は簡易確認とし、ベンチマーク追加は必須としない。

## Acceptance Criteria

- `get_selected_text()` が `normalized_selection_range()` の結果から行ごとの抽出範囲を直接決定している。
- `get_selected_text()` の抽出処理から、セルごとの `selection_contains()` 呼び出しが不要になっている。
- 既存の選択抽出テストが通る。
- 必要な単体テストが追加または更新され、同一行・複数行・スクロールバック・ワイド文字継続セルの既存挙動が維持されていることを確認できる。
- `cargo test` が通る。
- `cargo clippy` と `cargo fmt` が通る。
- `install.ps1` による実機確認タスクが `plan.md` に含まれている。
- Conductor workflow に従い、フェーズごとのユーザー手動検証とコミットタスクが `plan.md` に含まれている。

## Out of Scope

- GUI ハイライト側の `selection_contains()` 利用最適化。
- `SelectionRange` / `SelectionPoint` のデータ構造変更。
- コピー時の末尾空白トリムなど、抽出結果の意味変更。
- EmEditor SDK の選択 API 利用への置き換え。
- Windows Terminal 相当の新しい選択モード、矩形選択、単語選択、コピー設定の追加。
- 本格的なベンチマーク基盤の導入。
