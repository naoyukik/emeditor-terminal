# トラック仕様書: fix: PSReadLineの予測入力（Prediction）表示の改善 (Issue #19)

## 1. 概要
ターミナル内で PowerShell (PSReadLine) を使用する際、予測入力（Inline View）で表示されるテキストが通常の入力文字と区別できない問題を修正する。具体的には、ConPTY から送られてくる SGR (Select Graphic Rendition) シーケンスを解析し、文字色（Foreground Color）を描画に反映する土台を実装する。

## 2. 機能要件（Functional Requirements）
- **SGRシーケンスの解析（VT100/ANSI）**:
    - `ESC [ <n> m` 形式のシーケンスをパースする機能を `TerminalBuffer` または関連モジュールに追加する。
    - 特に `30-37` (基本8色)、`38;5;<n>` (256色)、`38;2;<r>;<g>;<b>` (TrueColor)、および `39` (デフォルト色復帰) のパースに対応する。
- **セル属性の拡張**:
    - `Cell` 構造体に、前景色（Foreground Color）を保持するフィールドを追加する。
- **レンダリングへの反映**:
    - `TerminalRenderer` (GDI) において、各セルの前景色属性を読み取り、`SetTextColor` 等を用いて適切な色で文字を描画する。
    - PSReadLine が予測入力に使用する色（一般的に ANSI 8色またはグレー系）が正しく表示されることを確認する。

## 3. 非機能要件（Non-Functional Requirements）
- **拡張性**: 後続のタスクで背景色やその他のスタイル（太字等）を追加しやすいように、属性の保持方式を設計する。
- **パフォーマンス**: 描画ループ内での色切り替えがパフォーマンスに大きな影響を与えないように実装する。

## 4. 受入基準（Acceptance Criteria）
- PowerShell (PSReadLine) の予測入力が、通常の入力文字よりも薄い色（グレー等）で表示され、視覚的に区別できること。
- `39` シーケンスによって、文字色がデフォルトに正しく戻ること。
- 既存の文字表示機能にデグレードが発生していないこと。

## 5. アウトオブスコープ（Out of Scope）
- 背景色の変更。
- 文字装飾（太字、斜体、下線等）。
- TrueColor の完全な再現（基本色優先）。