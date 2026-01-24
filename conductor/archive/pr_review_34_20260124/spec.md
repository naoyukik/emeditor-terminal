# Specification: PR #34 Review Fixes

## Overview
PR #34 に対する GitHub Copilot のレビュー指摘に基づき、バグ修正と安全性向上を行う。

## Requirements
1. **Alt+Space の確実な許可**
   - `WM_SYSCOMMAND` (SC_KEYMENU) のブロック処理において、`Alt + Space` (0x20) 由来のものはブロックせず `DefWindowProcW` に通すこと。
   
2. **エラーハンドリングの改善**
   - `WM_SYSKEYDOWN` 内での ConPTY 書き込み処理 (`write_to_conpty`) の結果を無視せず、エラー時はログ (`log::error!`) を出力すること。

## Refactoring (Future Scope)
- システムショートカット判定ロジックの共通化は、今回は実施せず別途 Issue 起票またはタスク化する。
