---
name: operating-git
description: Manage Git workflows including staging, commits, and status. Enforces project-specific rules like prohibiting 'git add .' and mandating folder-level staging for the 'conductor/' directory. Activate this skill before any git commands to ensure compliance with commit conventions.
---

# Git操作ガイドライン

## ステージング規則

**重要**: `git add .` および `git add -A` の使用は厳禁。

- 原則として、ファイルは必ず個別に指定する。
- **例外**: `conductor/` 配下のドキュメント類は、管理の整合性を保つためフォルダごと追加すること。
  - **Good**: `git add conductor/`
- `git add` の前後に `git diff` または `git diff --staged` で差分を確認すること。

```bash
# Good
git add src/domain/terminal.rs src/application/service.rs
git add conductor/

# Bad
git add .
git add -A
```

## コミットの粒度

- **原子的なコミット**: 1コミット = 1つの論理的な単位（機能追加、バグ修正、リファクタリング等）
- **混合禁止**: リファクタリングと機能追加を同じコミットに含めない

## コミットメッセージ形式

```
type: 日本語での簡潔な説明（50文字以内）

ref: チケット番号
```

- **1行目**: 型 (type) と日本語の説明
- **2行目**: 空行
- **3行目**: `ref: ` + Issue番号（ブランチ名の先頭数値）。存在しない場合は省略

## Type一覧

| Type | 用途 |
|------|------|
| `feat` | 新機能 |
| `fix` | バグ修正 |
| `refactor` | リファクタリング |
| `docs` | ドキュメントのみの変更 |
| `style` | コードの意味に影響しない変更（空白、フォーマット等） |
| `test` | テストの追加・修正 |
| `chore` | ビルドプロセスやツールの変更、依存関係の更新 |

## 例

ブランチ名: `23-add-scrollback-buffer`

```
feat: スクロールバックバッファのinterfaceを追加

ref: 23
```
