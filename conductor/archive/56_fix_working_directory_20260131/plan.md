# Implementation Plan - fix/set-working-directory-userprofile

## Phase 1: カレントディレクトリの設定
ConPTYプロセス生成時のパラメータを修正する。

- [x] Task: `src/infra/conpty.rs` の修正
    - [x] `std::env` をインポートする。
    - [x] `ConPTY::new` メソッド内で `USERPROFILE` 環境変数を取得する。
    - [x] 取得したパスを UTF-16 NULL終端文字列 (`Vec<u16>`) に変換する。
    - [x] `CreateProcessW` の第8引数 (`lpCurrentDirectory`) に、変換したパスのポインタ (`PWSTR`) を渡す。
- [x] Task: コンパイル確認
    - [x] `cargo build` が通ることを確認する。
- [x] Task: Conductor - User Manual Verification (Protocol in workflow.md)
    - [x] 実際にプラグインをインストールし、EmEditorで起動して初期ディレクトリを確認する。
