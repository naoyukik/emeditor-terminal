# Specification: windows-rs 0.62 Update

## Overview
本トラックの目的は、`windows` クレートをバージョン 0.58 から最新の 0.62 (Release 73) へアップデートすることである。この移行に伴う破壊的変更（`windows-targets` の廃止、型シグネチャの厳密化、クレートの細分化など）を解決し、ライブラリの推奨する実装方針に合わせてコードベースを刷新する。これにより、将来的な保守性と安定性を確保する。

## Functional Requirements
- `Cargo.toml` において `windows` クレートを 0.62 に更新する。
- ビルド依存から古い `windows-targets` に依存する設定を排除し、`windows-link` に移行する。
- `windows-core`, `windows-registry` などの細分化された独立クレートを導入し、ライブラリの最新の推奨実装へ変更する。
- `w!` マクロや文字列ポインタ (`PCWSTR`, `PWSTR`) の呼び出しシグネチャを新しい仕様へ適応させる。
- `*const` と `*mut` の区別など、ポインタ型の厳密化に対応するため各レイヤーの Win32 API 呼び出しを修正する。
- （該当する場合）`VARIANT` や `PROPVARIANT` などの `Drop` 実装削除に伴い、手動でのメモリ管理を適切に行う。

## Non-Functional Requirements
- アーキテクチャの制約（GUI, Infra, Application, Domain）を維持し、`windows` 関連のコードは引き続き境界内部に封じ込めること。

## Acceptance Criteria
- 全てのモジュールが `cargo check` および `cargo build` で警告やエラーなくコンパイルできること。
- `cargo test` で既存のユニットテストがすべてパスすること。
- 実機検証において、最低限以下の「現状維持」が確認できること。
  - 描画の安定性（フリッカーなし）
  - ConPTYプロセスの起動および終了の正常動作
  - IME 入力および変換ウィンドウの追従
  - GUI 設定ダイアログの正常な開閉・設定の保存

## Out of Scope
- 本アップデートに直接関係のない新機能の追加や、ターミナル機能そのもののロジックの変更。