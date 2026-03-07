---
name: gh-sub-issue-creator
description: Handles the creation of GitHub Sub-issues. Use this skill when you need to add existing issues as sub-issues to a parent Epic/Issue. It provides the correct usage of the `sub_issue_write` tool and fallback procedures using GitHub CLI (`gh api`) if the tool fails.
---

# GitHub Sub-issue Creator

This skill provides the exact protocol for adding sub-issues to a parent issue in a GitHub repository.

## The Core Concept

When using the `sub_issue_write` tool, the `sub_issue_id` parameter **MUST NOT** be the human-readable issue number (e.g., `74`). Instead, it requires the **internal Node ID or Global Issue ID** (a large numeric identifier like `3971828927`) returned by GitHub's REST or GraphQL API.

## Step 1: Obtain the Internal Numeric `id` of the Target Sub-issue

Before calling `sub_issue_write`, you must retrieve the internal numeric `id` (NOT the issue number) of the issue you want to attach.

### Method A: Use the `issue_read` Tool (Recommended)
Call `issue_read` with the `method: "get"` and the human-readable `issue_number` of the **child** issue.
The output will contain an `"id"` field with a large numeric value.

```json
// Example: Get info for issue #74
{
  "owner": "naoyukik",
  "repo": "emeditor-terminal",
  "issue_number": 74,
  "method": "get"
}
// Output snippet: {"id": 3971828927, "number": 74, ...}
```
Use the value `3971828927`.

### Method B: Using GitHub GraphQL API (GraphQL-Scalpel)
Windows PowerShell 5.1 環境における文字化けとコンテキスト（トークン）効率の最適化のため、以下の手順を厳守すること。

1. **JSONボディ作成**: `create_new_file` で `temporary.local/query_id.json` を作成し、以下を書き込む。
```json
{
  "query": "query($owner: String!, $repo: String!, $number: Int!) { repository(owner: $owner, name: $repo) { issue(number: $number) { id } } }",
  "variables": { "owner": "{owner}", "repo": "{repo}", "number": {issue_number} }
}
```
2. **コマンド実行**: `run_shell_command` で、`cmd /c` を介して実行し、出力をファイルに保存。
```powershell
cmd /c "gh api --method POST /graphql --input temporary.local/query_id.json > temporary.local/gh_id.json"
```
3. **内容取得**: `get_file_text_by_path` で結果（例: `{"data":{"repository":{"issue":{"id":"I_..."}}}}`）を読み取り、IDを取得する。

### Method C: When Creating a New Issue
If you just created the issue using `issue_write`, the output already contains the `id`.
```json
{"id":"3971828927","url":"..."}
```

---

## Step 2: Use the `sub_issue_write` Tool

Once you have the internal `id`, call the `sub_issue_write` tool.

```json
// Correct Example
{
  "owner": "naoyukik",
  "repo": "emeditor-terminal",
  "issue_number": 73,        // The human-readable number of the PARENT issue
  "sub_issue_id": 3971828927,// The internal ID of the CHILD issue
  "method": "add"
}

// INCORRECT Example (Will result in 404 error)
{
  "owner": "naoyukik",
  "repo": "emeditor-terminal",
  "issue_number": 73,
  "sub_issue_id": 74,        // ERROR: Do not use the issue number here!
  "method": "add"
}
```

## Step 3: Fallback Procedure (If `sub_issue_write` fails)

If the `sub_issue_write` tool returns a 404 or 422 error, or fails for another reason, you can attempt to link the issues manually via the GitHub GraphQL API as a fallback.
Windows PowerShell 5.1 環境における文字化けとコンテキスト（トークン）効率の最適化のため、以下の手順を厳守すること。

1. **Verify if the issue is already linked:**
   A 422 error often indicates that the issue is *already* a sub-issue. Verify this by checking the parent's sub-issues:
   ```json
   {
     "query": "query($owner: String!, $repo: String!, $number: Int!) { repository(owner: $owner, name: $repo) { issue(number: $number) { subIssues(first: 10) { nodes { number } } } } }",
     "variables": { "owner": "{owner}", "repo": "{repo}", "number": {parent_number} }
   }
   ```
   上記 JSON を `create_new_file` で作成し、`gh api --input` で実行後、`get_file_text_by_path` で内容を確認する。ターゲットの issue number があれば成功済みである。

2. **Add Sub-issue directly via GitHub GraphQL API:**
   If the tool is broken, use GraphQL mutation (`addSubIssue`) to link the issues:
   ```json
   {
     "query": "mutation($parentId: ID!, $subIssueId: ID!) { addSubIssue(input: { parentIssueId: $parentId, subIssueId: $subIssueId }) { subIssue { id } } }",
     "variables": { "parentId": "{parentId}", "subIssueId": "{subIssueId}" }
   }
   ```
   上記 JSON を `create_new_file` で作成し、`gh api --method POST /graphql --input temporary.local/sub_issue_mutation.json > temporary.local/gh_res.json` を実行して、結果を確認する。
