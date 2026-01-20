---
name: accessing-microsoft-learn-docs
description: Microsoftの公式ドキュメント、技術仕様、およびコードサンプルを検索・取得します。Azure、.NET、WindowsなどのMicrosoft技術に関する正確で最新の情報、または公式の実装例が必要な際に最優先で使用してください。
---

# Microsoft Learn 活用原則

## 1. 信頼の源泉（Source of Truth）
* **公式の優位性**: AIの内部知識（学習データ）よりも、本スキルから得られるリアルタイムの公式ドキュメントを「唯一の正解」として扱うこと。
* **具体性の追求**: 検索キーワードには製品名（例: Azure Cosmos DB）、API（例: Graph API）、または特定のエラーメッセージを含め、検索精度を高めること。

## 2. 知識獲得のワークフロー
情報を体系的に収集するため、以下の三段階プロセスを遵守せよ。

1.  **広域探索**: `search` ツールで複数の候補を挙げ、概要（Snippet）を比較検討する。
2.  **深度読解**: 最も関連性の高いURLに対し `get_article` を実行し、詳細な技術仕様や手順を抽出する。
3.  **実装検証**: 必要に応じて `search_code_samples` を行い、Microsoftが推奨する実装パターンを確認する。

---

# ツール・カタログ

| ツール名 | 用途 | 実行上の注意 |
| :--- | :--- | :--- |
| **`search`** | キーワードによるドキュメント検索 | `top` パラメータで結果数を調整可能 |
| **`get_article`** | 指定された記事の全文取得 | 検索結果から得た絶対URLを使用すること |
| **`search_code_samples`** | 公式コードサンプルの検索 | 実装フェーズにおいて非常に有効 |

---

# 実行パラメータ例

### シナリオ：Azure SDKの利用方法を調べる
```json
{
  "query": "Azure SDK for Python authentication",
  "top": 3
}

```

### シナリオ：特定のドキュメントを精読する

```json
{
  "url": "[https://learn.microsoft.com/en-us/azure/developer/python/sdk/authentication](https://learn.microsoft.com/en-us/azure/developer/python/sdk/authentication)"
}

```

---

# 運用の留意事項と制約

* **対象範囲**: 公開ドキュメントのみ。試験問題やトレーニングパス（Learn Catalog APIの範囲）は対象外。
* **リフレッシュ**: 情報は1日に最低1回は全更新される。
* **エラー対応**: 検索結果が不十分な場合は、上位概念のキーワードで広めに再検索せよ。
