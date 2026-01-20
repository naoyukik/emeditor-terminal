# Specification: ConPty Cursor Rendering

## 1. Overview
このトラックでは、ConPty (Pseudo Console) からの出力を解析し、カスタムターミナルウィンドウ上にカーソルを描画する機能を実装します。
初期フェーズでは「Bar (バー)」スタイルのカーソルのみをサポートし、将来的に他のスタイル（Block, Underline）や点滅への対応を容易に拡張できる設計とします。

## 2. Goals
- ConPtyからの出力ストリームに含まれるカーソル制御シーケンスを解析する。
- ターミナルの内部状態としてカーソル位置（行・列）と可視性を管理する。
- ウィンドウ描画処理（WM_PAINT等）において、文字グリッドの上にオーバーレイとしてカーソルを描画する。

## 3. Functional Requirements

### 3.1. VT Sequence Parsing
以下のANSIエスケープシーケンスを解析し、内部状態を更新する必要があります。
- **Cursor Position (CUP):** `ESC [ <y> ; <x> H`
    - カーソルの論理位置（行、列）を更新する。
- **Cursor Visibility (DECTCEM):** `ESC [ ? 25 h` (Show) / `l` (Hide)
    - カーソルの表示/非表示フラグを更新する。

### 3.2. State Management
- `Terminal` 構造体（またはそれに準ずる状態管理クラス）に以下のフィールドを追加する。
    - `cursor_x`: usize (1-based or 0-based, consistency with existing logic)
    - `cursor_y`: usize
    - `cursor_visible`: bool

### 3.3. Rendering
- **Style:** "Bar" (縦線) スタイル固定。
    - 幅: 1px または 2px (視認性を考慮して調整)
    - 高さ: フォントのラインの高さに合わせる
    - 色: テキストの反転色または固定色（白/システムカラー）
- **Positioning:**
    - 論理位置 (row, col) を画面上のピクセル座標 (x, y) に変換して描画する。
    - `x = col * char_width + padding_left`
    - `y = row * line_height + padding_top`
- **Overlay:**
    - テキスト描画の後、最前面に描画する。

### 3.4. Limitations (Phase 1)
- **Shape Override:** アプリケーションからのカーソル形状変更要求 (`DECSCUSR`) は無視し、常に "Bar" スタイルを使用する。
- **Blinking:** カーソルの点滅アニメーションは実装しない（常時点灯）。

## 4. Non-Functional Requirements
- **Performance:** カーソル位置の更新は頻繁に発生するため、更新時の再描画は最小限（カーソル周辺のみ、またはバッファリングされた描画）に抑えることが望ましいが、初期実装ではウィンドウ全体の再描画でも可とする（要検討）。
- **Smoothness:** カーソル移動がユーザー入力に対して遅延なく追従すること。

## 5. Out of Scope
- カーソルの点滅 (Blinking)
- Block, Underline スタイルの描画
- IME入力時のキャレット制御（Windowsのシステムキャレット位置の更新など）
