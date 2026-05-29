#!/usr/bin/env python3
import json
import os
import sys
from pathlib import Path
from datetime import datetime
# === CONFIGURATION ===
SAVE_INTERVAL = 10  # N回の人間のメッセージごとに保存を促す
STATE_DIR = Path.home() / ".mempalace" / "hook_state"
LOG_FILE = STATE_DIR / "hook.log"

# オプション: 自動インジェストを行いたいディレクトリがあれば設定する
MEMPAL_DIR = ""

def log(message):
    if not STATE_DIR.exists():
        return
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    try:
        with open(LOG_FILE, "a", encoding="utf-8") as f:
            f.write(f"[{timestamp}] {message}\n")
    except Exception:
        pass

def count_human_messages(transcript_path):
    if not transcript_path:
        return 0

    path = Path(os.path.expanduser(transcript_path))
    if not path.exists():
        return 0

    count = 0
    try:
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
            messages = data.get("messages", [])
            for msg in messages:
                if msg.get("type") == "user":
                    content = msg.get("content", "")
                    # content がリスト形式の場合の処理
                    if isinstance(content, list):
                        text = "".join(c.get("text", "") for c in content if isinstance(c, dict))
                    else:
                        text = str(content)

                    if "<command-message>" in text:
                        continue
                    count += 1
    except Exception as e:
        log(f"Error reading transcript: {e}")
    return count

def normalize_session_id(value):
    raw = str(value)
    return "".join(c for c in raw if c.isalnum() or c in ("-", "_")) or "unknown"

def is_stop_hook_active(value):
    if isinstance(value, bool):
        return value
    if isinstance(value, str):
        return value.strip().lower() == "true"
    return False

def main():
    # 状態ディレクトリの準備
    try:
        STATE_DIR.mkdir(parents=True, exist_ok=True)
    except OSError:
        # ディレクトリが作成できない環境（権限不足等）ではログ出力を諦める
        pass

    # stdin から JSON を読み込む
    try:
        input_data = json.load(sys.stdin)
    except Exception:
        input_data = {}

    session_id = normalize_session_id(input_data.get("session_id", "unknown"))
    
    stop_hook_active = is_stop_hook_active(input_data.get("stop_hook_active", False))
    transcript_path = input_data.get("transcript_path", "")

    # すでに保存サイクルに入っている場合は、無限ループ防止のため通常通り停止を許可
    if stop_hook_active:
        print(json.dumps({}))
        return

    # メッセージ数のカウント
    exchange_count = count_human_messages(transcript_path)
    
    # 最後に保存した時点のカウントをロード
    last_save_file = STATE_DIR / f"{session_id}_last_save"
    last_save = 0
    if last_save_file.exists():
        try:
            last_save = int(last_save_file.read_text().strip())
        except (ValueError, OSError):
            last_save = 0

    since_last = exchange_count - last_save
    log(f"Session {session_id}: {exchange_count} exchanges, {since_last} since last save")

    # 保存のタイミング判定
    if since_last >= SAVE_INTERVAL and exchange_count > 0:
        # 保存ポイントを更新
        try:
            last_save_file.write_text(str(exchange_count))
        except OSError as e:
            log(f"Failed to write last save file: {e}")

        log(f"TRIGGERING SAVE at exchange {exchange_count}")

        # オプション: MEMPAL_DIR が設定されている場合の自動インジェスト
        if MEMPAL_DIR and os.path.isdir(MEMPAL_DIR):
            import subprocess
            # 非同期で実行
            try:
                subprocess.Popen([sys.executable, "-m", "mempalace", "mine", MEMPAL_DIR], 
                                 stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            except Exception as e:
                log(f"Failed to start auto-ingest: {e}")

        # AI をブロックし、保存を命じる
        # この reason は AI にシステムメッセージとして提示される
        response = {
            "decision": "block",
            "reason": "AUTO-SAVE checkpoint. Save key topics, decisions, quotes, and code from this session to your memory system. Organize into appropriate categories. Use verbatim quotes where possible. Continue conversation after saving."
        }
        print(json.dumps(response))
    else:
        # まだ保存のタイミングではない
        print(json.dumps({}))

if __name__ == "__main__":
    main()
