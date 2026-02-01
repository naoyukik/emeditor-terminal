---
name: replying-to-pr-threads
description: Replies to specific GitHub Pull Request review threads using GraphQL API. Use when addressing individual review comments.
---

# GitHub PRレビューコメントへの返信方法

特定のレビューコメントスレッドに対して返信を行う場合、以下の手順を実行する。

## 1. スレッドIDの取得

GitHub MCP Serverの `pull_request_read` ツールを使用し、`method: "get_review_comments"` を指定してレビューコメント一覧を取得する。
レスポンス内の `reviewThreads` 配下にある `ID` (例: `PRRT_...`) がスレッドIDである。

## 2. 返信コマンドの実行

取得したスレッドID (`<THREAD_ID>`) と返信内容 (`<BODY>`) を用いて、以下の `gh` コマンドを実行する。
`run_shell_command` ツールを使用すること。

```bash
gh api graphql -f query='mutation($threadId: ID!, $body: String!) { addPullRequestReviewThreadReply(input: {pullRequestReviewThreadId: $threadId, body: $body}) { comment { url } } }' -F threadId="<THREAD_ID>" -F body="<BODY>"
```

このコマンドは GraphQL API を使用してスレッドに返信を追加する。
成功すると、作成されたコメントのURLが返される。