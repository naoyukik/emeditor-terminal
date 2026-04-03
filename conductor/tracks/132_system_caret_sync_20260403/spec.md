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


## 6. 詳細実装設計 (Detailed Implementation Design)

### 6.1 影響範囲の特定と現状評価
コードベースを調査した結果、前任の実装（コミット \2b1ffe5\）において、同期のフック（\WM_PAINT\, \WM_IME_STARTCOMPOSITION\, \WM_IME_COMPOSITION\, \WM_APP_REPAINT\）や、\GetFocus\ ガード、\CaretHandle\ (RAII) は既に組み込まれていることが判明した。
したがって、本トラックの核心的な修正箇所は \src/gui/driver/ime_gui_driver.rs\ の \sync_system_caret\ 関数に絞られる。

### 6.2 座標変換プロトコル
1. 
enderer.cell_to_pixel(x, y) でクライアント座標 (pixel_x, pixel_y) を取得する。
2. SetCaretPos(pixel_x, pixel_y) を実行する（これはOS仕様通りクライアント座標のままでよい）。
3. POINT { x: pixel_x, y: pixel_y } を作成し、ClientToScreen(hwnd, &mut pt) を実行してスクリーン座標へ変換する。
4. 除外領域となる RECT (
c_exclude) をクライアント座標で構築する。
5. MapWindowPoints(hwnd, HWND(0), rc_exclude as *mut RECT as *mut POINT, 2) 等を使用して、RECT 全体をスクリーン座標へ一括変換する。
6. 変換後のスクリーン座標を持つ pt および 
c_exclude を、COMPOSITIONFORM と CANDIDATEFORM に適用し、ImmSetCompositionWindow と ImmSetCandidateWindow を呼び出す。

### 6.3 実装対象ファイル
- **src/gui/driver/ime_gui_driver.rs**: 
  - sync_system_caret 関数内の座標計算と IME API 呼び出し部分の改修。
  - ClientToScreen および MapWindowPoints のインポート (windows::Win32::Graphics::Gdi または windows::Win32::UI::WindowsAndMessaging)。


### 追記: 問題の正確な定義
当初『候補窓』が飛ぶと認識されていたが、実際には『入力中文字（コンポジション文字列）』そのものが画面端に飛んでしまう（オフセットされる）現象であることが判明した。Windows Terminal の開発者が直面したのもこの入力位置のズレであり、スクリーン座標への変換によってこの根本原因を解決する。
