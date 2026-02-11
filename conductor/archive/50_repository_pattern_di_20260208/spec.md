# Specification: RepositoryパターンとDIの導入

## Overview
本トラックでは、Issue #50に基づき、現在のモノリシックな構造を改善し、DIP（依存性逆転の原則）を実現するためのリファクタリングを行う。
具体的には、Domain層にRepositoryトレイトを定義し、Infra層でその実装を提供、Application層（`TerminalService`）へDI（Constructor Injection）を通じて依存関係を注入する形へ変更する。

## Goals
- `TerminalService` から具体的な `ConPTY` や `WriteFile` への直接依存を排除する。
- Domain層に `TerminalOutputRepository` と `ConfigurationRepository` トレイトを定義する。
- ユニットテストにおいてモックを用いたテストが可能になる状態にする。

## Functional Requirements
1.  **Repositoryトレイトの定義**
    - `src/domain/repository/terminal_output_repository.rs`: ターミナルへの入出力（resize, send_input）を抽象化。
    - `src/domain/repository/configuration_repository.rs`: 設定情報（フォント、色、動作設定）の取得を抽象化。

2.  **Infra層での実装**
    - `src/infra/repository/conpty_repository_impl.rs`: `TerminalOutputRepository` を実装し、内部で `ConPTY` を制御する。
    - `src/infra/repository/emeditor_config_repository_impl.rs`: `ConfigurationRepository` を実装し、EmEditorの設定情報を取得する。

3.  **Application層へのDI**
    - `TerminalService` のコンストラクタを変更し、`Box<dyn TerminalOutputRepository>` および `Box<dyn ConfigurationRepository>` を受け取るようにする。
    - `set_conpty` などの動的セッターを廃止し、生成時の注入（Constructor Injection）に統一する。

4.  **設定情報の取得（ハイブリッド方式）**
    - 基本設定は `TerminalService` 生成時にリポジトリから取得して保持する（キャッシュ）。
    - EmEditorの設定変更イベントや明示的な更新要求に応じて、キャッシュを更新するメソッドを提供する。

## Non-Functional Requirements
- **Testability**: 実際のWin32 APIを呼ばずに `TerminalService` のロジックをテストできること。
- **Performance**: 設定情報の取得が毎フレームの描画処理のボトルネックにならないこと。
- **Code Style**: プロジェクトの `rust_win32.md` 規約に準拠する。

## Out of Scope
- クリップボード操作のリファクタリング（今回は対象外）。
- EmEditorプラグインのUI部分の大規模な改修。
