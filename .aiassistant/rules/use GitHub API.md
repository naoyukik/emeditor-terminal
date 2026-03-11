---
apply: by model decision
instructions: GitHubのデータ取得を行う際に確認する。トークン効率の最適化と情報の正確な取得を目的とする。
---

# GitHub 操作標準 (GitHub-Efficiency)

本プロジェクトでは、AIエージェントのコンテキスト（トークン）効率を最大化し、正確な情報を取得するため、以下の運用を標準とする。

## 1. 原則
- **GitHub CLI (gh) の優先**: `gh pr view`, `gh issue list` 等の標準コマンドで事足りる場合は、それらを積極的に使用すること。
- **GraphQL による最適化 (GraphQL-Scalpel)**: 大規模なデータ（大量のレビューコメント等）を取得する際は、不要な情報を削ぎ落としてトークンを節約するため、GraphQL を使用して必要なフィールドのみをピンポイントで取得すること。

## 2. GraphQL-Scalpel プロトコル
大規模データを取得する際の推奨手順：

1. **クエリ設計**: 必要なフィールドのみを要求する最小限のクエリを作成する。
2. **コマンド実行**: `run_shell_command` で `gh api graphql -f query='...' -f owner='...' -f name='...'` のように実行する。
   - ※クエリが非常に複雑な場合は、以前のように `temporary.local/query.json` を介して `--input` で読み込ませる手法も有効である。

## 3. 具体的活用例
（以下、クエリ例は維持）

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

## 4. 運用上の注意
- **コンテキスト（トークン）効率**: REST API 等で不要な大量のメタデータ（`node_id`, `url` 等）が含まれるレスポンスを避け、AIが即座に推論に集中できる最小限のデータ構造を維持せよ。
- **ユーザーの集中力保護**: `execute_terminal_command` は IDE のターミナルタブを強制的に開き、ユーザーのフォーカスを奪うため、明示的な指示がない限り使用を避けること。
