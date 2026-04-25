# Specification - Mouse Event Pass-through (SGR 1006)

## Overview
マウス操作（クリック、スクロール、ドラッグ）を VT シーケンス (SGR 1006) に変換し、ConPTY へ送信する機能を実装する。これにより、EmEditor ターミナル内で Vim や tmux などの TUI アプリケーションにおけるマウス操作が可能になる。

## Functional Requirements
1. **マウスイベントの変換**:
    - `WM_LBUTTONDOWN`, `WM_LBUTTONUP`, `WM_RBUTTONDOWN`, `WM_RBUTTONUP`, `WM_MBUTTONDOWN`, `WM_MBUTTONUP` を VT シーケンスに変換する。
    - マウストラッキングが有効な場合（特にドラッグ中）、`WM_MOUSEMOVE` を VT シーケンスに変換する。
    - `WM_MOUSEWHEEL` および `WM_MOUSEHWHEEL` を VT シーケンスに変換する。
2. **SGR 1006 サポート**:
    - エンコーディング形式として `<Pb;Px;PyM` (プレス) / `m` (リリース) を使用する。
    - 座標は 1 ベースのターミナル座標系に変換する。
3. **修飾キーの統合**:
    - Ctrl, Alt, Shift の各修飾キーの状態をボタンパラメータの特定ビットに反映させる。
4. **競合解決 (Shift バイパス)**:
    - Shift キーが押されている間は VT 変換をバイパスし、EmEditor 本体の標準機能（テキスト選択、ドラッグ＆ドロップ等）を優先させる。
5. **モード管理**:
    - TUI アプリがマウストラッキング（DEC SET 1000, 1002, 1003 等）を要求している場合のみ、マウスイベントを送信する。

## Non-Functional Requirements
- **低遅延**: スムーズな操作感のため、マウスイベントは最小限の遅延で ConPTY へ送信されること。
- **正確性**: 座標がターミナルのセル単位で正確にマッピングされていること。

## Acceptance Criteria
- [ ] Vim (`set mouse=a` 設定時) でマウスクリックが認識されること。
- [ ] TUI アプリケーションでマウスホイールによるスクロールが機能すること。
- [ ] tmux のペインのリサイズなどのマウスドラッグ操作が機能すること。
- [ ] マウストラッキング有効時でも、Shift + クリック/ドラッグで EmEditor 本体のテキスト選択が行えること。
- [ ] ログ等により、モダンな SGR 1006 エンコーディングが使用されていることが確認できること。

## Out of Scope
- レガシーなマウスエンコーディング (X10, URXVT) のサポート。
- Shift バイパス挙動のカスタマイズ（当面は Windows Terminal に準拠）。
