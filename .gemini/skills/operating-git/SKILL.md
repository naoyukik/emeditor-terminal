---
name: operating-git
description: Managing Git workflows with zero tolerance for 'git add .' or 'git add -A'. This skill enforces individual file staging and mandatory folder-level staging for the conductor/ directory. It MANDATES that commit messages MUST have a Japanese description part following the type (e.g., 'type: 日本語での説明'). Verify diffs before every commit to ensure atomicity and prevent accidental inclusion of untracked artifacts or secrets.
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

## コミットメッセージの作成

コミットメッセージの内容および形式については、**必ず `referencing-commit-convention` スキルを参照し、日本語での記述を徹底すること**。
本スキルは Git 操作の手順とステージングの制約（git add . 禁止等）を司る。

## 例

ブランチ名: `23-add-scrollback-buffer`

```
feat: スクロールバックバッファのinterfaceを追加

ref: 23
```
