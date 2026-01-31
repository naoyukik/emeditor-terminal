# Track Specification: PR #57 Review Fixes

## Overview
PR #57 でのコードレビュー指摘に対応する。主な修正点は `src/infra/conpty.rs` における `current_dir_w` のライフタイム問題の解消である。

## Functional Requirements
- `src/infra/conpty.rs` 内で、`current_dir_w` の定義を `unsafe` ブロックの外側に移動し、`CreateProcessW` 呼び出しまで確実に生存するように修正する。

## Acceptance Criteria
- [ ] `current_dir_w` の定義位置が修正されていること。
- [ ] `cargo build` が正常に通過すること。
- [ ] EmEditor Terminal が正常に起動し、初期ディレクトリが `%USERPROFILE%` であること（リグレッションテスト）。
