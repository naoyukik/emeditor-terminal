---
apply: by model decision
instructions: GitHubのデータ取得をユーザーが求めたとき、ghコマンドを使用する際に確認する
---

# List review comments on a pull request
Lists all review comments for a specified pull request. By default, review comments are in ascending order by ID.
```bash
gh api \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  /repos/OWNER/REPO/pulls/PULL_NUMBER/comments
```
