---
name: JetBrains MCP server
description: IntelliJを直接操作可能なMCPサーバーです。コードリーディングやファイル作成等の操作を行う際、正確、安全、かつ効率的にコードベースを操作することが可能なため最優先で使用すること。
---
# Absolute Principles
1. **プロジェクトパスの明示**: すべてのツール呼び出しにおいて、`projectPath` パラメータが既知の場合は必ずフルパスで指定し、曖昧さを回避すること。
2. **推測より確認**: ファイル構造や内容を推測してはならない。必ず `list_directory_tree` や `get_file_text_by_path` を使用して現状を把握してから行動すること。
3. **破壊的変更の回避**: コードの変更は外科手術のように精密に行うこと。ファイル全体を書き換えるのではなく、可能な限り `replace_text_in_file` を使用して差分のみを適用すること。
4. **構造的リファクタリング**: 変数名やクラス名の変更には、単純な置換ではなく、必ず `rename_refactoring` を使用し、プロジェクト全体の整合性を保つこと。
5. ツールを使用する際は、ユーザーに対して「何をするために、どのツールを、どのようなパラメータで実行するか」を簡潔に説明してから実行に移ること。

# Tool Usage Guidelines

## 1. Exploration & Navigation
- プロジェクトの全体像を把握するには、まず `list_directory_tree` を使用せよ。`maxDepth` を適切に設定し、トークンを浪費しないように注意すること。
- (以下のツールは不具合のため使用不可)特定のファイルを探す際は、低速な全探索ではなく、以下のインデックス付き検索ツールを優先せよ：
    - ファイル名がわかっている場合: `find_files_by_name_keyword`
    - パターンで探す場合: `find_files_by_glob`
    - コード内のテキストを探す場合: `search_in_files_by_text` または `search_in_files_by_regex`

## 2. Reading & Understanding
- ファイルの内容を読む際は `get_file_text_by_path` を使用せよ。
- 定義元や型情報を知りたい場合は、推測せず `get_symbol_info` を使用してIDEの静的解析情報を取得せよ。
- 依存関係の調査には `get_project_dependencies` および `get_project_modules` を活用せよ。

## 3. Editing & Refactoring
- **ファイルの編集**: `replace_text_in_file` を最優先で使用せよ。これにより、変更箇所以外の誤った書き換え（ハルシネーション）を防ぐことができる。
    - `oldText` は一意に特定できる十分な長さのコンテキストを含めること。
- **新規作成**: `create_new_file` を使用する際は、親ディレクトリの存在を確認し、既存ファイルを意図せず上書きしないよう `overwrite` フラグに注意せよ。
- **リネーム**: シンボル（変数、関数、クラス）の名前変更は、テキスト置換ではなく `rename_refactoring` を使用すること。これは絶対的なルールである。

## 4. Verification & Debugging
- コードを修正した後は、必ず `get_file_problems` を実行し、新たな構文エラーや警告が発生していないか確認すること。
- 実行構成（Run Configuration）が存在する場合は、`get_run_configurations` でリストを取得し、`execute_run_configuration` で実行して動作確認を行うこと。
- ターミナルコマンドが必要な場合は `execute_terminal_command` を使用するが、システムに不可逆な変更を与えるコマンド（rm -rf など）の実行には細心の注意を払うこと。

## 5. Error Handling
- ツール実行が失敗した場合（例: `file not found`）、エラーメッセージを分析し、パスの誤りやテキストの不一致を修正して再試行すること。無闇に同じコマンドを繰り返さないこと。

### パラメーター例
- search_in_files_by_text

  ```json
  {
    "searchText": "example",
    "projectPath": "/path/to/projectPath",
    "maxUsageCount": 10
  }
  ```

- search_in_files_by_regex

  ```json
  {
    "regexPattern": "fn \\w+",
    "projectPath": "Z:/projects/emeditor-terminal",
    "fileMask": "*.rs",
    "maxUsageCount": 15
  }
  ```

- find_files_by_name_keyword

  ```json
  {
    "nameKeyword": "terminal",
    "projectPath": "Z:/projects/emeditor-terminal",
    "fileCountLimit": 10
  }
  ```
- find_files_by_glob

  ```json
  {
    "globPattern": "**/*.rs",
    "projectPath": "Z:/projects/emeditor-terminal",
    "fileCountLimit": 20
  }
  ```
