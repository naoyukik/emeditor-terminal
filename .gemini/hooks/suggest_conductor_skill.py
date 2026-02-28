import sys
import json

def main():
    try:
        raw_input = sys.stdin.read()
        if not raw_input:
            print(json.dumps({"decision": "allow"}))
            return

        input_data = json.loads(raw_input)
        prompt = input_data.get("prompt", "")

        # /conductor で始まる、あるいは含むメッセージを検出
        if "/conductor" in prompt:
            print(json.dumps({
                "decision": "allow",
                "systemMessage": "💡 Conductor関連の操作が検出されました。必ず 'conductor-protocol' スキルをアクティベートし、その手順（spec.md/plan.mdの作成、ユーザー検証等）を厳格に遵守してください。"
            }))
        else:
            print(json.dumps({"decision": "allow"}))

    except Exception:
        print(json.dumps({"decision": "allow"}))

if __name__ == "__main__":
    main()
