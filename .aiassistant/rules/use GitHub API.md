---
apply: by model decision
instructions: GitHubのデータ取得をユーザーが求めたとき、ghコマンドを使用する際に確認する。Windows PowerShell 5.1 環境における文字化けとトークン効率の最適化を目的とする。
---

# GitHub GraphQL API (gh) 運用標準

本プロジェクトでは、Windows PowerShell 5.1 環境における文字化け回避と、AIエージェントのコンテキスト（トークン）効率を極限まで高めるため、**GitHub REST API の使用を原則禁止し、GraphQL によるピンポイント取得を唯一の標準とする。**

## 1. GraphQL-Scalpel プロトコル
複雑な JSON エスケープや BOM 混入を避けるため、以下の手順を厳守すること。

### 手順
1. **クエリ設計**: 必要なフィールド（例: `number`, `title`, `body`）のみを要求する最小限のクエリを作成する。
2. **JSON ボディ作成**: `create_new_file` で `temporary.local/query.json` に `{ "query": "...", "variables": { ... } }` を書き込む。
3. **コマンド実行**: `run_shell_command` 内で `cmd /c` を介して `gh api --method POST /graphql --input temporary.local/query.json > temporary.local/gh_res.json` を実行する。
4. **内容取得**: `get_file_text_by_path` で結果を読み取る。

## 2. 具体的活用例

### PRのレビューコメント取得
```json
{
  "query": "query($owner: String!, $name: String!, $pr: Int!) { repository(owner: $owner, name: $name) { pullRequest(number: $pr) { reviews(last: 1) { nodes { comments(last: 100) { nodes { path line body } } } } } } }",
  "variables": { "owner": "naoyukik", "name": "emeditor-terminal", "pr": PULL_NUMBER }
}
```

### Issue一覧の取得
```json
{
  "query": "query($owner: String!, $name: String!) { repository(owner: $owner, name: $name) { issues(last: 10, states: OPEN) { nodes { number title } } } }",
  "variables": { "owner": "naoyukik", "name": "emeditor-terminal" }
}
```

## 3. 運用上の注意
- **文字化け回避**: `cmd /c` とリダイレクトを組み合わせ、PowerShell の標準出力をバイパスすること。
- **コンテキスト効率**: REST API のように不要な `node_id` や `url` が大量に含まれるレスポンスを避け、AIが即座に推論に集中できるデータ構造を維持せよ。
- **ユーザー保護**: `execute_terminal_command` は IDE のターミナルタブを強制的に開き、ユーザーのフォーカスを奪うため、明示的な指示がない限り使用を避ける。
