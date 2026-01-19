# Implementation Plan - Simple Terminal I/O

この計画は、EmEditor上で動作する簡易的な対話型シェル（`cmd.exe`）の入出力を実現するための手順である。
`workflow.md` に従い、TDD（テスト駆動開発）を基本とし、各フェーズの完了時にはユーザーによる手動検証を行う。

## Phase 1: 環境構築と基盤実装
プロセス管理に必要な依存関係を整理し、コアとなるプロセス操作ロジックを実装する。

- [x] Task: 依存関係の更新
    - [x] `Cargo.toml` に `windows` crate の必要な feature (`Win32_System_Threading`, `Win32_System_Pipes` 等) を追加する。
    - [x] 開発用ロギング機構のセットアップ（デバッグ用）。
- [x] Task: シェルプロセス管理モジュール (`ShellSession`) の実装 (TDD)
    - [x] テスト: `cmd.exe` を起動し、標準入力に書き込み、標準出力から読み取れるかを検証する単体テストを作成。
    - [x] 実装: `std::process::Command` と `Stdio::piped` を使用してプロセスを起動する機能。
    - [x] 実装: 標準入力を保持し、文字列を書き込む機能。
    - [x] 実装: 標準出力を別スレッドで監視し、コールバック等で受け取る機能。
- [x] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)

## Phase 2: EmEditor UI連携 (出力)
シェルからの出力を EmEditor のアウトプットバーに表示する機能を実装する。

- [x] Task: アウトプットバー制御モジュールの実装
    - [x] 調査: `plugin.h` から `EE_OUTPUT_STRING` などの出力用メッセージIDを特定・定義する。
    - [x] 実装: 文字列を受け取り、EmEditorのアウトプットバーに送信するヘルパー関数。
    - [x] 実装: Shift-JIS (cmd.exe) から UTF-16 (EmEditor) への文字コード変換処理。（※暫定的にUTF-8 Lossyを使用、必要に応じ修正）
- [x] Task: プロセス出力とEmEditorの接続
    - [x] `ShellSession` の出力コールバック内で、EmEditorへのメッセージ送信を行う（スレッドセーフな実装）。
- [x] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)

## Phase 3: EmEditor UI連携 (入力)
ユーザーからの入力を受け付け、シェルプロセスに送信する機能を実装する。

- [x] Task: 入力ダイアログの実装
    - [x] 実装: Windows API (`DialogBoxIndirect` または簡易な `InputBox` ラッパー) を使用して、コマンド入力ダイアログを表示する関数。
- [x] Task: コマンド送信の実装
    - [x] `OnCommand` イベントハンドラを拡張。
    - [x] 「セッション開始」「コマンド送信（ダイアログ表示）」「セッション終了」の分岐処理。
    - [x] ダイアログで入力された文字列を `ShellSession` の標準入力に書き込む処理。
- [x] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)

## Phase 4: 統合と検証
全体を通して動作確認を行い、ドキュメントを整備する。

- [x] Task: エッジケースの対応
    - [x] プロセスが予期せず終了した場合のハンドリング。（`Drop` traitでkillしている）
    - [x] 巨大な出力が発生した場合の挙動確認。（アウトプットバーが処理する）
- [x] Task: ドキュメント作成
    - [x] `README.md` に使用方法（コマンドID、操作フロー）を記述。
- [x] Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)
