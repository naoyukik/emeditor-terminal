---
name: check-pr-review-comments-graphql
description: Analyzing GitHub Pull Request review comments and managing follow-up tasks. This skill utilizes GraphQL to efficiently fetch review comments and categorizes them into critical fixes, future improvements, or out-of-scope items. Mandatory when a PR has received feedback to systematically address reviewer suggestions and ensure all necessary changes are tracked in Conductor.
---

# steps
- name: Review PR Comments
  description: GitHub Copilot のプルリクエストレビューコメントを確認し、必要な修正や改善を特定します。
- name: Document Changes
  description: 変更内容をドキュメント化し、将来の参考のために記録します。

# レビューコメントの取得方法 (GitHub-Efficiency)
情報の正確な取得とコンテキスト（トークン）効率を最大化するため、以下の手順で GraphQL を使用してください。

1. **コマンド実行**: `run_shell_command` を使用し、直接 `gh api graphql` を実行して情報を取得します。
  - 以前のように JSON ファイルを作成したり、`cmd /c` を介したりする必要はありません。

```powershell
gh api graphql -f query='
  query($owner: String!, $name: String!, $pr: Int!) {
    repository(owner: $owner, name: $name) {
      pullRequest(number: $pr) {
        reviews(last: 1) {
          nodes {
            comments(last: 100) {
              nodes { path line body }
            }
          }
        }
      }
    }
  }' -f owner='naoyukik' -f name='emeditor-terminal' -F pr=PULL_NUMBER
```

2. **内容確認**: `run_shell_command` の実行結果から、レビュー内容を直接確認してください。

# レビューコメントの報告
レビューコメントを取得したら、ユーザーに以下のフォーマットで報告してください。
> **【PR Review Comments】**
> - **Total Comments**: `X` 件のレビューコメントが見つかりました
> - **Comments Summary**:
> - 1. `コメント内容1` - 指摘内容の概要
> - 2. `コメント内容2` - 指摘内容の概要

# 妥当性やスコープの吟味
レビューコメントの内容を吟味し、実装すべき修正点や改善点を特定してください。必要に応じて、ユーザーに確認を取ることも検討してください。
また、指摘を「今回のPRの変更が影響を及ぼし、今回のPRで必須なもの」と「将来的な改善（別タスク）に回すべきもの」に明確に切り分けます。切り分けは後続の `分類と対応方針` を参照。
吟味した結果、トータル的に実装する必要がないと判断すれば、その旨をユーザーに報告してください。

## 分類と対応方針
レビューコメントを以下のカテゴリに分類し、対応方針を決定せよ。
1.  Critical / Safety: セキュリティ、クラッシュ、安全性、コンパイル警告に関わるもの。
    -> Action*: 今回のPRで必ず修正する。
2.  Bug Fix: PRの機能が正しく動作しない原因となるもの。
    -> Action*: 今回のPRで修正する。
3.  Refactoring / Design: 設計の美しさ、カプセル化、将来の拡張性に関するもの。
    -> Action*: 今回は修正せず、"Future Ticket" として起票する。
4.  Optimization: パフォーマンス改善（現状でボトルネックでない場合）。
    -> Action*: 今回は修正せず、"Future Ticket" として起票する。
5.  Nitpick: 些細な指摘（スコープ外のエッジケースなど）。
    -> Action*: 無視、または丁重に断る。

# タスク管理

タスクを忘れずに実行できるように、Conductorのtrackにタスクを追加してください。
現在進行中のtrackがある場合はそこに追加。アーカイブされている場合等で現在進行中のtrackが存在しない場合、新規にtrackを作成してください。

# 終了
タスク管理まで完了したら、実装は行わずにこのスキルを終了してください
