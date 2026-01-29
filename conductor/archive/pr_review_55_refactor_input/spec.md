# Track pr_review_55_refactor_input Specification

## 1. Overview
PR #55 "refactor: extract keyboard input logic to domain and infra layers" に対するレビューコメントに基づき、安全性、保守性、およびテストカバレッジを向上させる修正を行う。

## 2. Functional Requirements
- **Infra層の安全性向上**:
    - `KeyboardHook::install()` において、フック登録成功後に `TARGET_HWND` を設定するように順序を変更する。
    - `PostMessageW` の戻り値をチェックし、失敗時はエラーログを出力する。
- **GUI層の保守性向上**:
    - システムショートカット（Alt+Tab等）の除外リストを定数または関数として共通化し、`WM_APP_KEYINPUT` と `WM_SYSKEYDOWN` で共有する。
    - `lparam` に関するコメントを修正し、`GetKeyState` を使用している理由を明記する。
- **Domain層の品質向上**:
    - 欠落しているテストケース（Ctrl+Alt, F1-F12, PageUp/Down, Enter/Tab/Esc, Alt+特殊キー）を追加する。
    - Alt+数字キーの処理において、`vk_code` から ASCII 文字への変換を明示的な計算式に変更する。

## 3. Architecture Details
- **src/infra/input.rs**:
    - `install()` メソッド内のロジック順序変更。
    - `keyboard_hook_proc` 内のエラーハンドリング追加。
- **src/gui/custom_bar.rs**:
    - `is_system_shortcut` ヘルパー関数の追加（または定数配列）。
    - `WM_APP_KEYINPUT` と `WM_SYSKEYDOWN` ハンドラの修正。
- **src/domain/input.rs**:
    - `VtSequenceTranslator::translate` 内の Alt+数字ロジック修正。
    - `tests` モジュールの拡張。

## 4. Acceptance Criteria
- [ ] `KeyboardHook` のインストール処理が安全な順序になっていること。
- [ ] フックからのメッセージ送信失敗がログに記録されること。
- [ ] システムショートカットの除外ロジックが一箇所で定義されていること。
- [ ] `src/domain/input.rs` のテストカバレッジが向上し、指定されたケースを網羅していること。
- [ ] `cargo test` および `cargo check` が警告なしでパスすること。
