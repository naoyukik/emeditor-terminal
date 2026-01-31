# Implementation Plan - fix/set-working-directory-userprofile

## Phase 1: カレントディレクトリの設定
ConPTYプロセス生成時のパラメータを修正する。

- [ ] Task: `src/infra/conpty.rs` の修正
    - [ ] `std::env` をインポートする。
    - [ ] `ConPTY::new` メソッド内で `USERPROFILE` 環境変数を取得する。
    - [ ] 取得したパスを UTF-16 NULL終端文字列 (`Vec<u16>`) に変換する。
    - [ ] `CreateProcessW` の第8引数 (`lpCurrentDirectory`) に、変換したパスのポインタ (`PWSTR`) を渡す。
- [ ] Task: コンパイル確認
    - [ ] `cargo build` が通ることを確認する。
- [ ] Task: Conductor - User Manual Verification (Protocol in workflow.md)
    - [ ] 実際にプラグインをインストールし、EmEditorで起動して初期ディレクトリを確認する。
