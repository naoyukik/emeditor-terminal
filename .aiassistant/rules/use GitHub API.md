---
apply: by model decision
instructions: GitHubのデータ取得をユーザーが求めたとき、ghコマンドを使用する際に確認する
---

---
apply: by model decision
instructions: GitHubのデータ取得をユーザーが求めたとき、ghコマンドを使用する際に確認する。Windows PowerShell 5.1 環境における文字化けとトークン効率の最適化を目的とする。
---

# GitHub API / CLI (gh) 運用標準

本プロジェクトでは、Windows PowerShell 5.1 環境における文字化け回避と、AIエージェントのコンテキスト（トークン）節約のため、以下の **"Silent-gh Protocol"** および **"GraphQL-Scalpel"** を標準とする。

## 1. 基本方針 (Silent-gh Protocol)
PowerShell 5.1 の標準リダイレクト (`>`) は UTF-16LE への自動変換を行い、文字化けを誘発するため使用を禁止する。

- **書き込み**: `run_shell_command` 内で `cmd /c` を介して生の UTF-8 バイト列をファイルに出力する。
  - **推奨例**: `cmd /c "gh issue list --limit 10 --json title,number > temporary.local/gh_out.json"`
- **読み取り**: 保存された UTF-8 ファイルは、PowerShell の `Get-Content` ではなく、JetBrains MCP の `get_file_text_by_path` を使用して読み取る。これにより、シェルのエンコーディング干渉を受けずに日本語を正確に解釈できる。

## 2. データの取得 (GraphQL-Scalpel)
大規模なデータ（PRのレビューコメント、複雑なコミット履歴等）を取得する際は、REST API よりも GraphQL を優先し、必要なプロパティのみをピンポイントで抽出すること。

- **エスケープとBOMの回避**: 複雑な JSON エスケープや BOM 混入を避けるため、クエリを含む JSON ボディを `create_new_file` で作成し、`gh api` の `--input` フラグで読み込ませる。
- **手順**:
  1. `create_new_file` で `temporary.local/query.json` に `{ "query": "..." }` を書き込む。
  2. `run_shell_command` で `cmd /c "gh api --method POST /graphql --input temporary.local/query.json > temporary.local/gh_res.json"` を実行。
  3. `get_file_text_by_path` で結果を読み取る。

## 3. レビューコメントの取得例 (REST版)
特定のプルリクエストのレビューコメントを取得する場合：

```powershell
cmd /c "gh api /repos/naoyukik/emeditor-terminal/pulls/PULL_NUMBER/comments > temporary.local/pr_comments.json"
```
実行後、`get_file_text_by_path(pathInProject="temporary.local/pr_comments.json")` で内容を確認すること。

## 4. 注意事項
- **ユーザー保護**: `execute_terminal_command` は IDE のターミナルタブを強制的に開き、ユーザーのフォーカスを奪うため、明示的な指示がない限り使用を避ける。
- **GitHub MCP**: トークン消費が激しいため、上記 CLI 手順を優先する。
