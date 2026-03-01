# Implementation Plan: Grapheme Clusters 方式による文字処理エンジンの刷新 (Issue #106)

## Phase 1: 依存関係の追加とデータ構造の定義
- [x] Task: `Cargo.toml` に `unicode-segmentation` と `unicode-width` クレートを追加する。
- [x] Task: `src/domain/model/cell_entity.rs` (仮) の `Cell` 構造体を拡張し、`char` ではなく `String` を保持するように変更する。
- [x] Task: 既存の `TerminalBufferEntity` の関連するメソッド（`get_cell`, `set_cell` 等）のシグネチャを新しい `Cell` に合わせて調整する。
- [x] Task: コードをコミットする。
- [x] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)

## Phase 2: 書記素クラスター判定とパースロジックの刷新
- [x] Task: `TerminalBufferEntity` 内に、不完全な書記素クラスターを保持するためのテンポラリバッファ（確定待ちバッファ）を実装する。
- [x] Task: `print(char)` 時の入力をバッファリングし、`unicode-segmentation` を用いてクラスターが確定したタイミングでグリッドへ書き込むロジックを実装する。
- [x] Task: `unicode-width` を用いて、クラスター全体の物理カラム数（1 or 2）を算出し、グリッド上のセル占有を制御する。
- [x] Task: コードをコミットする。
- [x] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)

## Phase 3: 操作ロジック（カーソル・削除・境界保護）の刷新
- [x] Task: カーソル移動（左右）において、クラスターの境界を跨ぐようにロジックを修正する。
- [x] Task: バックスペースおよび `DEL` キー処理を、クラスター単位での削除に対応させる。
- [x] Task: ワイド文字（全角）の「泣き別れ」を防ぐための境界保護ロジックを、クラスター方式に合わせて再構築する。
- [x] Task: コードをコミットする。
- [x] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)

## Phase 4: テストによる検証とクリーンアップ
- [x] Task: 複雑な Unicode シーケンス（Emoji ZWJ, 結合文字）を用いた単体テストを作成・実行し、期待通りに動作することを確認する。
- [x] Task: Issue #104 の再現ケースを用い、ワイド文字間での挿入・削除によって座標がズレないことを確認する。
- [x] Task: `Clippy` / `cargo fmt` を実行し、既存のマルチバイト再構築ロジックのクリーンアップ（不要コードの削除）を行う。
- [x] Task: コードをコミットする。
- [x] Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)

## Phase 5: レビュー指摘修正と標準準拠
- [x] Task: `unicode-segmentation` を実際に使用し、`pending_cluster` バッファによる確実なクラスター判定を実装する。
- [x] Task: 文字幅計算を 1-2 カラムに制限し、カーソル移動 (CUF/CUB) を 1カラム単位の標準挙動に戻す。
- [x] Task: ログ出力から生テキストを除去し、セキュリティ・プライバシーリスクを解消する。
- [x] Task: 削除された `handle_decscusr` のテストを復元し、絵文字・国旗・NFD 等の検証ケースを拡充する。
- [x] Task: 描画ループ内の属性参照化など、レビューで指摘された最適化を適用する。
- [x] Task: コードをコミットする。
- [x] Task: Conductor - User Manual Verification 'Phase 5' (Protocol in workflow.md)
