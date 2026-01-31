# Track Specification: fix/set-working-directory-userprofile

## Overview
現在、EmEditor Terminal で `pwsh` を起動すると、カレントディレクトリが `System32` (EmEditorの実行コンテキスト依存) になってしまっている。これは誤操作によるシステム破壊のリスクがあり危険であるため、開始時のカレントディレクトリをユーザーのホームディレクトリ (`%USERPROFILE%`) に設定する。

## Functional Requirements
- `pwsh.exe` プロセス起動時、カレントディレクトリを環境変数 `USERPROFILE` の値に設定する。
- `USERPROFILE` が取得できない場合は、デフォルトの挙動（現状維持または適切なフォールバック）とするが、Windows環境では通常取得可能である。

## Technical Implementation
- `src/infra/conpty.rs` 内の `CreateProcessW` 呼び出しにおいて、`lpCurrentDirectory` パラメータを指定する。
- Rust の `std::env::var("USERPROFILE")` を使用してパスを取得し、Wide String (UTF-16, NULL終端) に変換して渡す。

## Acceptance Criteria
- [ ] EmEditor Terminal 起動直後のプロンプトが、ユーザーのホームディレクトリ (例: `C:\Users\Username`) を示していること。
- [ ] `System32` ではないこと。
