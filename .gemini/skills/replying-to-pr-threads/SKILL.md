---
name: replying-to-pr-threads
description: Replies to specific GitHub Pull Request review threads using GraphQL API. Use when addressing individual review comments.
---

# GitHub PRレビューコメントへの返信方法

特定のレビューコメントスレッドに対して返信を行う場合、以下の手順を実行する。

## 1. スレッドIDの取得

GitHub MCP Serverの `pull_request_read` ツールを使用し、`method: "get_review_comments"` を指定してレビューコメント一覧を取得する。
レスポンス内の `reviewThreads` 配下にある `ID` (例: `PRRT_...`) がスレッドIDである。

## 2. 返信コマンドの実行 (GraphQL-Scalpel)

取得したスレッドID (`<THREAD_ID>`) と返信内容 (`<BODY>`) を用いて、以下の手順で返信を実行する。
Windows PowerShell 5.1 環境における文字化けとエスケープ、およびコンテキスト（トークン）効率の最適化のため、以下の手順を厳守すること。

1. **JSONボディ作成**: `create_new_file` で `temporary.local/reply_query.json` を作成し、以下の内容を書き込む。
```json
{
  "query": "mutation($threadId: ID!, $body: String!) { addPullRequestReviewThreadReply(input: {pullRequestReviewThreadId: $threadId, body: $body}) { comment { url } } }",
  "variables": {
    "threadId": "<THREAD_ID>",
    "body": "<BODY>\n\nCommented by Gemini CLI"
  }
}
```
2. **コマンド実行**: `run_shell_command` で、`cmd /c` を介して `gh api` を実行し、結果をファイルにリダイレクトする。
```powershell
cmd /c "gh api --method POST /graphql --input temporary.local/reply_query.json > temporary.local/gh_res.json"
```
3. **結果確認**: 保存された `temporary.local/gh_res.json` を `get_file_text_by_path` で読み取り、成功したか（コメントURLが返されているか）を確認する。
