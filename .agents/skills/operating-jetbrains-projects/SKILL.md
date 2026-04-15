---
name: operating-jetbrains-projects
description: Operates JetBrains IDEs (IntelliJ, etc.) via MCP. Supports project exploration, code reading, structural refactoring, and execution/verification using full IDE capabilities for efficient development.
---

# 鉄則（Absolute Principles）
1. **プロジェクトパスの明示**: すべての呼び出しで `projectPath` をフルパスで指定せよ。
2. **推測より確認**: ファイル構造やシンボル情報は必ず `list_directory_tree` や `get_symbol_info` で裏付けを取れ。
3. **精密な外科手術**: 編集は `replace_text_in_file` で最小限の差分のみを適用せよ。
4. **構造的整合性**: 名前変更には必ず `rename_refactoring` を用い、テキスト置換を避けよ。
5. **高度な読み取り**: 巨大なファイルでは `read_file` の `slice` や `lines` モード、または `indentation` モードを使い、必要な部分だけを効率的に読み取れ。
6. **execute_terminal_command**: 基本的に使用しない。`run_shell_command` でどうしても実行できない場合のみ、ユーザーに確認を取ってから使用すること。

# ツール・カタログ（Toolbox Reference）

## 1. 探索とナビゲーション（Exploration & Navigation）
大規模プロジェクトの全容を迅速に把握するための手段です。

| ツール名 | 用途 | 備考 |
| :--- | :--- | :--- |
| `list_directory_tree` | プロジェクト構造の俯瞰 | `maxDepth` を指定して使用 |
| `get_repositories` | VCS ルートの取得 | マルチリポジトリ構成の把握に便利 |
| `find_files_by_name_keyword` | ファイル名による特定 | インデックスによる高速検索 |
| `find_files_by_glob` / `search_file` | パターンでのファイル検索 | 特定の拡張子やディレクトリの絞り込み |
| `search_text` / `search_regex` | 文字列・パターンによる一括検索 | **推奨**: スニペットと座標（1-based）を返す |
| `search_symbol` | シンボル（クラス・メソッド・フィールド）検索 | 識別子断片によるセマンティックな検索 |
| `get_all_open_file_paths` | 現在開いているエディタの把握 | 作業中の文脈を素早く理解するために有用 |

## 2. 読解と分析（Reading & Understanding）
コードの意味と依存関係を正確に理解するための手段です。

| ツール名 | 用途 | 備考 |
| :--- | :--- | :--- |
| `get_file_text_by_path` | ファイル内容の読み込み（全文） | 比較的小さなファイルや全文把握に |
| `read_file` | 高度なファイル読み取り | `slice`, `lines`, `indentation` モードを使い分け |
| `open_file_in_editor` | エディタで指定ファイルを開く | ユーザーへの提示やエディタ操作の連動に |
| `get_symbol_info` | 型情報・定義・参照の取得 | IDEのクイックドキュメントと同等の正確な情報 |
| `get_project_dependencies` | 外部ライブラリ等の依存関係確認 | プロジェクト構成の理解 |
| `get_project_modules` | モジュール構成の把握 | 多階層プロジェクトの分析 |

## 3. 編集とリファクタリング（Editing & Refactoring）
安全性と整合性を保ちながらコードを修正するための手段です。

| ツール名 | 用途 | 備考 |
| :--- | :--- | :--- |
| `replace_text_in_file` | テキストの部分置換 | **最優先ツール**。一意な `oldText` を指定 |
| `create_new_file` | 新規ファイルの作成 | `overwrite` フラグに注意 |
| `rename_refactoring` | シンボルの名前変更 | 変数・クラス・関数の改名に必須 |

## 4. 検証とデバッグ（Verification & Debugging）
修正の正しさを証明し、品質を担保するための手段です。

| ツール名 | 用途 | 備考 |
| :--- | :--- | :--- |
| `get_file_problems` | 構文エラー・警告の取得 | 編集後の特定ファイルのチェック |
| `build_project` | プロジェクト全体のビルド | コンパイルエラーの包括的取得に必須 |
| `execute_terminal_command` | 任意コマンドの実行 | `rm -rf` 等の破壊的コマンドには細心の注意 |

- **execute_terminal_command** については基本的に使用しない。`run_shell_command` でどうしても実行できない場合、ユーザーに確認を取ってから使用すること。


# ワークフロー例

### ステップ：精密なコード修正
1. `search_text` で修正対象を検索し、スニペットで文脈を確認。
2. `read_file` または `get_file_text_by_path` で正確な修正箇所を特定。
3. `replace_text_in_file` で修正を適用。
4. `get_file_problems` で構文エラーがないことを確認。
5. `build_project` を実行し、プロジェクト全体の整合性を検証。

# エラー処理
- `file not found` の場合、`list_directory_tree` や `search_file` でパスを再確認せよ。
- `text not found` の場合、`read_file` 等を再実行し、正確な `oldText` を取得し直せ。
