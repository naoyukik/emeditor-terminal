# プロジェクト情報

## GitHubリポジトリー
https://github.com/naoyukik/emeditor-terminal

## プロジェクト・ガイドライン
本プロジェクトの開発においては、以下のドキュメントに記載されたルールを厳守すること。
なお、`conductor/` ディレクトリ全体がプロジェクトのガイドライン、設計資料、および開発プロセスを包含している。

- **コーディング・アーキテクチャ規約**: [`conductor/code_styleguides/rust_win32.md`](conductor/code_styleguides/rust_win32.md)
  - レイヤードアーキテクチャ、Repositoryパターン、DIの導入、DDD命名規則、テスト戦略など。
- **Git 操作ガイドライン**: [`conductor/git.md`](conductor/git.md)
  - **重要**: `git add .` および `git add -A` の使用は固く禁ずる。ファイルは必ず個別に指定すること。
  - コミットメッセージの形式（Conventional Commits + チケット番号）。
- **Issue 作成ガイドライン**: [`conductor/github_issue.md`](conductor/github_issue.md)

## 開発参考資料
- EmEditor Plugin SDK: [EmEditor Plugin SDK 公式ドキュメント](https://www.emeditor.com/sdk/)
  - EmEditor のサンプルコードやプラグインの開発に関するドキュメント。
  - このプロジェクト内の `sdk/` ディレクトリにも SDK 関連の資料が含まれている。
- Learn Microsoft: `learn microsoft` MCP Serverによってドキュメントを検索可能。
  - Windows APIや関連技術の公式ドキュメント。

## 開発環境
- 言語: Rust (Win32 API)

## Conductor
Conductor Extensionが呼び出されて会話する際、日本語を使用してください。