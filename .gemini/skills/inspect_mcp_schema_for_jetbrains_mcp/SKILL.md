---
name: inspect_mcp_schema_for_jetbrains_mcp
description: JetBrains MCP Serverのツールの仕様を提供します。MCP Serverから必要なパラメーターの情報が得られない場合、このスキルを参照してください。
---

ここに記載されていないが、MCP Serverのツールの仕様が不明な場合、ユーザーに連絡して記載の追記を依頼してください。

# get_file_text_by_path
## パラメーター仕様
ここではtruncateModeパラメーターの仕様を示します。その他は既存で定義されているとおりです。
```yaml
- truncateMode":
    - "type": "string"
    - "enum": ["START", "MIDDLE", "END", "NONE"]
    - "description": "How to truncate the text: from the start, in the middle, at the end, or don't truncate at all"
```
