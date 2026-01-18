---
name: check-pr-review-comments
description: GitHub Copilot のプルリクエストレビューコメントを確認し、必要な修正や改善を行います。コードの品質向上とバグ修正を目的としています。
---

# steps
- name: Review PR Comments
  description: GitHub Copilot のプルリクエストレビューコメントを確認し、必要な修正や改善を特定します。
- name: Implement Changes
  description: 指摘された問題点に基づき、コードの修正や改善を行います。
- name: Verify Changes
  description: 変更が正しく適用され、コードの品質が向上していることを確認します。
- name: Document Changes
  description: 変更内容をドキュメント化し、将来の参考のために記録します。

# レビューコメントの取得方法

レビューコメントを取得する際、下記のコマンドを使用してください。
`PULL_NUMBER` はユーザーに提示しもらうか、もしくは既存の情報から取得してください。
```bash
# GitHub CLI api
# https://cli.github.com/manual/gh_api

gh api \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  /repos/OWNER/REPO/pulls/PULL_NUMBER/comments
```
このAPIはデフォルトでは、レビューコメントはIDの昇順で並べ替えられます。
