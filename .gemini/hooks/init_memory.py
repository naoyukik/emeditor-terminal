#!/usr/bin/env python3
import json
import sys

def main():
    # 1. ログ出力（stderrへ送る）
    sys.stderr.write("Hook: Synchronizing with Memory Palace...\n")

    # 2. stdinからの入力読み取り
    try:
        input_data = json.load(sys.stdin)
    except Exception:
        input_data = {}

    # 3. 追加コンテキストの定義
    # Memory MCP (Knowledge Graph) から MemPalace (AAAK) への移行を反映
    memory_instruction = (
        "自己認識のため、自分の名前を宣言せよ。"
        "スキル referencing-commit-convention, rust-coding-conventions, autonomous-researcher, operating-git を読み込め。"
        "mempalace_get_taxonomy を実行して、Memory Palace の最新のインデックス（地図）を読み取れ。"
        "特に architecture-decisions, conductor-workflow, bug-analysis ルームの直近の知見を優先的に取得し、"
        "準備ができたら報告せよ。"
    )

    # 4. JSONレスポンスの構築
    response = {
        "hookSpecificOutput": {
            "additionalContext": memory_instruction
        },
        "systemMessage": "🏰 MemPalace Syncing Active (via Python)"
    }

    # 5. stdoutへの出力（JSONのみ！）
    print(json.dumps(response))
    sys.exit(0) # 正常終了

if __name__ == "__main__":
    main()
