# Specification: システムキャレットの常時同期による IME 候補ウィンドウ位置の解決

## 1. 概要 (Overview)
TUI アプリケーション（特に Gemini CLI 等の Node.js ベースのもの）において、仮想カーソルの移動に追従して OS のシステムキャレットを能動的に同期させることで、IME（ATOK, Microsoft IME 等）の候補ウィンドウが常に正しい位置（仮想カーソル位置）に表示されるようにする。

## 2. 機能要件 (Functional Requirements)
- **能動的同期プロトコル**:
    - 文字入力、エスケープシーケンスによるカーソル移動、スクロール発生時にシステムキャレット（`SetCaretPos`）を同期。
    - 描画ループ（`on_paint`）の完了時にも最新のカーソル位置をキャレットに反映。
- **座標変換の実装**:
    - 仮想セル座標 (x, y) からウィンドウ物理座標 (pixel_x, pixel_y) への正確な変換。
    - IME ウィンドウ制御（`ImmSetCompositionWindow`, `ImmSetCandidateWindow`）において、**スクリーン座標**（`ClientToScreen`）を使用するように修正。
- **安全なライフサイクル管理**:
    - `CaretHandle` (RAII) によるシステムキャレットの生成・破棄。
    - `GetFocus` ガードにより、ターミナルがフォーカスを持っていない場合に EmEditor 本体の IME に干渉しないようにする。

## 3. 非機能要件 (Non-Functional Requirements)
- **パフォーマンス**: 描画ごとの同期処理が UI レスポンスを阻害しないこと。
- **アーキテクチャ**: 依存の方向性（GUI -> Application -> Domain）を維持し、`windows` クレートの依存を Driver 層に封印する。

## 4. 成功条件 (Acceptance Criteria)
- Gemini CLI 等の動的な TUI 環境において、ATOK を含む IME の候補ウィンドウが、常に現在の入力位置（仮想カーソル）の直下に表示されること。
- EmEditor 本体の検索バー等、他ウィンドウへのフォーカス移動時に IME の挙動が阻害されないこと。

## 5. スコープ外 (Out of Scope)
- システムキャレットの視覚的な「点滅」のカスタマイズ（位置同期を主目的とする）。
- 特定の IME 固有の不具合に対する個別のワークアラウンド（Win32 標準 API による汎用的解決を目指す）。
