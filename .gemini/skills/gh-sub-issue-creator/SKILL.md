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

### Method B: Using `gh api` (GitHub CLI)
If you prefer the terminal, use the following command to get the numeric ID for a specific issue number:

```bash
# Get numeric ID for issue #74
gh api repos/{owner}/{repo}/issues/74 --jq .id
```

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

If the `sub_issue_write` tool returns a 404 or 422 error, or fails for another reason, you can attempt to link the issues manually via the GitHub CLI as a fallback.

1. **Verify if the issue is already linked:**
   A 422 error often indicates that the issue is *already* a sub-issue. Verify this by checking the parent's sub-issues:
   ```bash
   gh api repos/{owner}/{repo}/issues/{issue_number}/sub_issues --jq ".[].number"
   ```
   If the target issue number appears in the output, the operation was already successful.

2. **Add Sub-issue directly via GitHub REST API:**
   If the tool is broken, use `gh api` to make the POST request yourself:
   ```bash
   gh api -X POST repos/{owner}/{repo}/issues/{parent_issue_number}/sub_issues -F sub_issue_id={internal_id}
   ```
   *(Replace `{owner}`, `{repo}`, `{parent_issue_number}`, and `{internal_id}` with actual values).*
