# Specification: PR #60 Review Fixes

## 1. Overview
PR #60 に対するレビューコメントに基づき、`src/gui/custom_bar.rs` の定数定義を修正する。

## 2. Changes
### 2.1. `src/gui/custom_bar.rs`
*   独自定義されている定数 `SB_VERT` とその上のコメント `// Scroll Bar Constants (Self-defined to avoid dependency issues)` を削除する。
*   代わりに `windows::Win32::UI::WindowsAndMessaging::SB_VERT` をインポートして使用する。

## 3. Acceptance Criteria
*   `cargo check` が通り、コンパイルエラーが発生しないこと。
*   `cargo test` が成功すること。
