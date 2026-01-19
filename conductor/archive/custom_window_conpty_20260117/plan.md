# Implementation Plan - Custom Window & ConPTY MVP

この計画は、EmEditor上で動作するカスタムバー形式のターミナルウィンドウを実現するための手順である。
`workflow.md` に従い、TDD（テスト駆動開発）を基本とし、各フェーズの完了時にはユーザーによる手動検証を行う。

## Phase 1: カスタムバーの実装 (Hello World)
EmEditor SDK の「カスタムバー」機能を調査し、単純なウィンドウを表示する。

- [x] Task: 調査とプロトタイピング
    - [x] SDKの `EEID_SHOW_CUSTOM_BAR` および関連するメッセージ、構造体 (`CUSTOM_BAR_INFO` 等) の定義を `plugin.h` から特定する。
    - [x] Rust (`windows` crate) で `RegisterClassW`, `CreateWindowExW` を使用して、カスタムバー内に埋め込むためのウィンドウクラスを定義・作成する機能を実装する。
    - [x] `OnCommand` でカスタムバーを表示し、作成したウィンドウをドッキングさせる処理を実装する。
- [x] Task: 描画テスト
    - [x] 作成したウィンドウの `WM_PAINT` を処理し、GDI (`TextOutW` 等) を使用して "Hello ConPTY" という文字列を描画する。
- [x] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)

## Phase 2: ConPTY の実装 (バックエンド)
Windows Pseudo Console (ConPTY) を作成し、プロセス (`cmd.exe`) を起動する機能を実装する。

- [x] Task: ConPTY モジュールの実装
    - [x] `CreatePseudoConsole`, `CreateProcessW` (with `EXTENDED_STARTUPINFO_PRESENT`), `ClosePseudoConsole` 等の API バインディングを確認・実装する。
    - [x] `ConPTY` 構造体を実装し、PTYのライフサイクルを管理する。
    - [x] 入出力パイプ (`CreatePipe`) のセットアップ。
- [x] Task: プロセス起動と出力取得
    - [x] `cmd.exe` を ConPTY 経由で起動する。
    - [x] 出力パイプからデータを読み取るスレッドを実装し、ログに出力して動作確認。 (ReadFile 0x80070006 エラー修正済み)
- [x] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)

## Phase 3: 統合と描画
ConPTY からの出力をカスタムバー上のウィンドウに描画する。

- [x] Task: 描画バッファの実装
    - [x] ターミナル画面（行・列の文字グリッド）を保持するデータ構造 (`TerminalBuffer`) を実装する。
    - [x] ConPTY からの出力を解析し、バッファを更新する（今回は単純な追記のみ、エスケープシーケンスは可能な範囲で除去または無視）。
- [x] Task: UIへの反映
    - [x] バッファ更新時に `InvalidateRect` を呼び出し、再描画をリクエストする。
    - [x] `WM_PAINT` でバッファの内容を GDI で描画する。
- [x] Task: 入力連携 (部分的完了 - バックスペースに既知の問題あり)
    - [x] カスタムバー上の `WM_CHAR` / `WM_KEYDOWN` をフックし、ConPTY の入力パイプに書き込む。
    - [x] (既知の問題) バックスペースキーの動作が不安定
      - Backspaceの修正 - \x08→\x7fに変更して対応済み
- [x] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)

## Phase 4: リファクタリングとクリーンアップ
- [x] Task: 安定性向上
    - [x] リサイズ (`WM_SIZE`) 時の ConPTY リサイズ (`ResizePseudoConsole`) 連動。
    - [x] プラグイン終了時 (`EVENT_CLOSE`) のプロセス終了処理。
- [x] Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)
