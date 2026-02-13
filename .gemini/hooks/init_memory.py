#!/usr/bin/env python3
import json
import sys

def main():
    # 1. ログ出力（stderrへ送る） [cite: 277, 512]
    sys.stderr.write("Hook: Synchronizing with Memory MCP...\n")

    # 2. stdinからの入力読み取り（必要に応じて利用可能） [cite: 273, 412]
    try:
        # SessionStartイベントの入力データ（session_id, cwdなど） [cite: 428, 491]
        input_data = json.load(sys.stdin)
    except Exception:
        input_data = {}

    # 3. 追加コンテキストの定義 [cite: 458, 492]
    memory_instruction = (
        "自己認識のため、自分の名前を宣言する"
        "スキル `referencing-commit-convention` `rust-coding-conventions` `think` `operating-git` を読み込んでください。"
        "Memory MCPから最新のプロジェクト文脈をロードしてください。"
        "特に、現在の設計方針、未完了のタスク、および重要な技術的決定事項を優先的に取得し、 "
        "準備ができたら報告せよ。"
    )

    # 4. JSONレスポンスの構築 [cite: 429]
    response = {
        "hookSpecificOutput": {
            "additionalContext": memory_instruction
        },
        "systemMessage": "🧠 Memory Syncing Active (via Python)"
    }

    # 5. stdoutへの出力（JSONのみ！） [cite: 274, 512]
    print(json.dumps(response))
    sys.exit(0) # 正常終了 [cite: 281, 413]

if __name__ == "__main__":
    main()
