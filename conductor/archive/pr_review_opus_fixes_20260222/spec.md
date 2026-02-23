# Specification: Opus 4.6 レビュー指摘修正

## Overview
セカンドオピニオン（Opus 4.6）による指摘事項のうち、リソースリークの危険性がある GDI オブジェクトの Drop 順序問題、およびコードの品質向上に関する修正を行う。

## Functional Requirements
1.  **SelectObject 復元の RAII 化**: `SelectedObjectGuard` を導入し、スコープを抜ける際に必ず元の GDI オブジェクトが再選択（現在のオブジェクトが選択解除）されることを保証する。
2.  **ブラシ管理の RAII 化**: `render_internal` 内のブラシ作成を `GdiObjectGuard` を用いた管理に移行する。
3.  **マジックナンバーの定数化**: `HGDI_ERROR` 相当の値を定数として定義し、可読性を高める。

## Acceptance Criteria
- [ ] パニック時や早期リターン時も GDI オブジェクトが確実に解放される構造になっていること。
- [ ] すべての GDI オブジェクト管理が RAII パターンに統一されていること。
- [ ] ビルドが正常に通ること。
