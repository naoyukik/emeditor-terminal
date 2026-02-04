# Specification: Refactor Scroll Logic to `src/gui/scroll.rs`

## 1. Overview
`src/gui/custom_bar.rs` に実装されているスクロールバー制御ロジックを、新規作成する `src/gui/scroll.rs` へ抽出し、リファクタリングを行う。
UIイベント処理（Windowsメッセージ）と状態管理を `ScrollManager` 構造体にカプセル化することで、`custom_bar.rs` の複雑度を下げ、保守性とテスト容易性を向上させる。

## 2. Architecture Design
*   **Module**: `src/gui/scroll.rs`
*   **Struct**: `ScrollManager`
    *   スクロール状態（`SCROLLINFO` 等）を保持・管理する。
    *   `custom_bar.rs` はこの構造体のインスタンスを保持する。
*   **Enum**: `ScrollAction`
    *   スクロール操作の結果としてアプリケーションが取るべき行動を定義する。
    *   例: `ScrollTo(usize)`, `ScrollBy(isize)`, `Redraw` 等。

## 3. Functional Requirements

### 3.1. ScrollManager Implementation
*   **状態管理**:
    *   現在のスクロール位置、ページサイズ、範囲（最小/最大）を管理する。
*   **メッセージハンドリング**:
    *   `handle_vscroll(wparam, lparam)`: `WM_VSCROLL` メッセージを解析し、適切な `ScrollAction` を返す。
    *   `handle_mousewheel(delta)`: `WM_MOUSEWHEEL` メッセージを解析し、適切な `ScrollAction` を返す。
*   **UI反映**:
    *   `update_scroll_info(...)`: 最新の状態に基づき `SCROLLINFO` 構造体を構築し、Windows API (`SetScrollInfo`) を呼び出してスクロールバーの表示を更新する。

### 3.2. Integration with CustomBar
*   `custom_bar.rs` 内の既存のスクロール処理ロジックを削除し、`ScrollManager` への委譲に置き換える。
*   `ScrollManager` から返された `ScrollAction` に基づき、`TerminalService` への操作（ビューポートの移動など）を行う。

## 4. Acceptance Criteria
*   `src/gui/scroll.rs` が作成され、スクロールロジックが移動していること。
*   `WM_VSCROLL` (ドラッグ、ラインスクロール、ページスクロール) が以前と同様に正常に動作すること。
*   `WM_MOUSEWHEEL` (ホイールスクロール) が以前と同様に正常に動作すること。
*   リファクタリング後も既存の機能（テキスト描画、入力など）にリグレッションがないこと。
*   `ScrollManager` に対する単体テストが記述されていること。
