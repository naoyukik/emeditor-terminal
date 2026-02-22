# Specification: PR #80 レビュー指摘修正

## Overview
GDI ダブルバッファリング導入の PR #80 に対して行われた GitHub Copilot レビューの指摘事項を修正し、描画パイプラインの堅牢性を高める。

## Functional Requirements
1.  **SelectObject 失敗時の早期リターン**: `SelectObject` の戻り値を検証し、失敗時はログを出力して描画を中断する。
2.  **背景クリアの堅牢化**: `CreateSolidBrush` 失敗時に `ExtTextOutW` (ETO_OPAQUE) を用いた代替クリアパスを実装する。
3.  **エラーハンドリング強化**: `BitBlt` 等の戻り値をチェックし、失敗時にエラーログを出力する。
4.  **InvalidateRect 最適化**: `WM_SIZE` 時の `InvalidateRect` の `bErase` 引数を `FALSE` に変更し、不要な `WM_ERASEBKGND` を抑制する。

## Acceptance Criteria
- [ ] すべての指摘箇所について、コード修正が適用されていること。
- [ ] ビルドが正常に通ること。
- [ ] 実機で描画ノイズやリソースリークが発生しないことを再確認すること。
