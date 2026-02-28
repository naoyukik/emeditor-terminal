# 軌跡仕様書 (Track Specification): 9_cursor_shape_20260227

## 1. 概要 (Overview)
現在、EmEditor-Terminal のカーソル形状は垂直線（Bar）に固定されている。
標準的なターミナルアプリケーションが要求する `DECSCUSR` (Set Cursor Style) シーケンスをサポートすることで、エディタの状態（挿入/上書きモード）やアプリケーションの意図に応じたカーソル形状の動的な変更を実現する。

## 2. 背景・目的 (Background / Purpose)
- `DECSCUSR` シーケンスを解釈し、ドメインモデルにカーソルスタイルを保持する。
- 保持されたスタイルに基づき、GUI レンダラーが適切な形状でカーソルを描画する。
- ユーザー体験を向上させ、他のターミナルエミュレータとの互換性を高める。

## 3. 機能要件 (Functional Requirements)
- **CSI シーケンスのパース**: `ESC [ n q` (DECSCUSR) を `vte::Perform` トレイト経由で正しく解釈する。
- **カーソルスタイルの保持**: `TerminalBufferEntity` 内の `Cursor` 構造体に、以下のスタイル情報を追加・保持する。
    - `0`, `1`: Blinking Block (デフォルト)
    - `2`: Steady Block
    - `3`: Blinking Underline
    - `4`: Steady Underline
    - `5`: Blinking Bar
    - `6`: Steady Bar
- **GUI レンダリングの更新**: `TerminalGuiDriver` において、`InvertRect` または `FillRect` を使用して指定された形状を描画する。
    - Block: セル全体を反転。
    - Underline: セル下部（1-2px）を反転。
    - Bar: セル左端（1-2px）を反転。
- **点滅（Blinking）属性の扱い**: 現時点では実際の点滅アニメーションは実装しないが、属性値としては保持し、将来の拡張に備える（現在は `Blinking` も `Steady` も静的な描画とする）。

## 4. 非機能要件 (Non-Functional Requirements)
- **パフォーマンス**: 描画ループ内での条件分岐を最小限に抑え、レンダリング速度に影響を与えないようにする。
- **カプセル化**: カーソルの内部状態（スタイル）は適切な Getter/Setter 経由で操作する（Issue #99 の方針に準拠）。

## 5. 承諾基準 (Acceptance Criteria)
- `echo -e "\e[1q"` (Block), `\e[3q"` (Underline), `\e[5q"` (Bar) 等のコマンドを発行した際に、カーソル形状が期待通りに変化すること。
- ウィンドウのリサイズやスクロールを行っても、カーソル形状が正しく維持・描画されること。
- `vte` クレートによるパースが正しく機能し、他のシーケンスに悪影響を与えないこと。

## 6. スコープ外 (Out of Scope)
- 実際の点滅アニメーションの実装（点灯状態のままとする）。
- カラー設定によるカーソル色の変更（反転による描画を維持）。
