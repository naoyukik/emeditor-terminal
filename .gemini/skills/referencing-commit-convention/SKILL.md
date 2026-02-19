---
name: referencing-commit-convention
description: Standardizing project history through strict commit message conventions. This skill MANDATES the use of Japanese for the description part (e.g., 'type: 日本語での説明') and 'ref: IssueNumber'. It ensures every commit is atomic and follows the Conventional Commits specification, facilitating a transparent and navigable audit trail. NEVER use English for the description part.
---
このスキルはコミットメッセージの作成を支援する。

## コミットメッセージ規約

[Conventional Commits](https://www.conventionalcommits.org/en/) に基づき、以下の形式で記述する。

### 形式
```
type: 日本語での説明（50文字以内）

ref: チケット番号
```

- **1行目 (Header)**: 型 (type) と **日本語の説明** を記述する。
- **2行目**: 空行。
- **3行目 (Footer)**: `ref: ` に続けて Issue 番号（ブランチ名の先頭数値）を記述する。存在しない場合は省略すること。

### Type の種類
- `feat`: 新機能
- `fix`: バグ修正
- `refactor`: リファクタリング
- `docs`: ドキュメントのみの変更
- `style`: コードの意味に影響しない変更（空白、フォーマットなど）
- `test`: テストの追加・修正
- `chore`: ビルドプロセスやツールの変更、依存関係の更新

### サンプル
```
feat: スクロールバックバッファのinterfaceを追加

ref: 23
```