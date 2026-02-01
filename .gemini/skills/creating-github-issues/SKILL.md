---
name: creating-github-issues
description: Assists with GitHub Issue creation. Provides title format and required section templates (Background/Purpose, Tasks, Goal). Use when creating or updating Issues.
---

# GitHub Issue作成ガイドライン

## タイトル形式

```
type: 日本語での簡潔な説明
```

接頭辞（type）は半角小文字とする。

## Type一覧

| Type | 用途 |
|------|------|
| `feat` | 新機能の追加 |
| `fix` | バグ修正 |
| `refactor` | リファクタリング（機能変更を伴わないコードの整理・改善） |
| `style` | コードの意味に影響しない修正（空白、フォーマット等） |
| `docs` | ドキュメントの更新 |
| `test` | テストの追加・修正 |
| `chore` | ビルドプロセスやツール、依存ライブラリの更新 |

## 内容テンプレート

```markdown
## 背景・目的 (Background / Purpose)

なぜこの作業が必要なのか、現状の課題は何かを記述する。
関連する設計指針やガイドラインがある場合は明記する。

## タスク (Tasks)

- [ ] 具体的な作業1
- [ ] 具体的な作業2
- [ ] 具体的な作業3

## ゴール (Goal)

どのような状態になれば、この Issue を「完了（Closed）」とみなすことができるかを定義する。
```

## 運用ルール

- **適切な粒度**: 1つのIssueで扱う範囲を大きくしすぎない。多岐にわたる場合は分割を検討
- **リファクタリングの根拠**: `refactor` Issueでは、どのコード規約やアーキテクチャ方針に基づいた変更なのかを明示
- **透明性**: 思考過程や途中の気付きはIssueのコメント欄に随時記録
