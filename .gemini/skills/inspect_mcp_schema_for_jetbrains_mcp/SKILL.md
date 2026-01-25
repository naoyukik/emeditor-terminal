---
name: mcp_schema_for_jetbrains_mcp
description: JetBrains MCP Serverのツールの詳細なSchemaを提供します。MCP Serverからエラーが返され、必要なパラメーターの情報が不明な場合、このスキルを参照してください。
---

- ここに記載されていないが、MCP Serverのツールの仕様が不明な場合、ユーザーに連絡して記載の追記を依頼してください。

# パラメーター仕様
## get_file_text_by_path
```json
{
    "type": "object",
    "properties": {
      "pathInProject": {
        "type": "string",
        "description": "Path relative to the project root"
      },
      "truncateMode": {
        "enum": [
          "START",
          "MIDDLE",
          "END",
          "NONE"
        ],
        "description": "How to truncate the text: from the start, in the middle, at the end, or don't truncate at all"
      },
      "maxLinesCount": {
        "type": "integer",
        "description": "Max number of lines to return. Truncation will be performed depending on truncateMode."
      },
      "projectPath": {
        "type": "string",
        "description": " The project path. Pass this value ALWAYS if you are aware of it. It reduces numbers of ambiguous calls. \n In the case you know only the current working directory you can use it as the project path.\n If you're not aware about the project path you can ask user about it."
      }
    },
    "required": [
      "pathInProject"
    ]
}
```
