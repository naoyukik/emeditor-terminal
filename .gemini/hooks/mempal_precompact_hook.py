#!/usr/bin/env python3
import json
import os
import subprocess
import sys
from datetime import datetime
from pathlib import Path


STATE_DIR = Path.home() / ".mempalace" / "hook_state"
LOG_FILE = STATE_DIR / "hook.log"

# Optional: set to the directory you want auto-ingested before compaction.
# Example: MEMPAL_DIR = str(Path.home() / "conversations")
MEMPAL_DIR = ""


def log(message):
    if not STATE_DIR.exists():
        return
    timestamp = datetime.now().strftime("%H:%M:%S")
    try:
        with open(LOG_FILE, "a", encoding="utf-8") as f:
            f.write(f"[{timestamp}] {message}\n")
    except Exception:
        pass


def main():
    # Prepare state directory
    try:
        STATE_DIR.mkdir(parents=True, exist_ok=True)
    except OSError:
        # Silently fail if directory cannot be created
        pass

    try:
        input_data = json.load(sys.stdin)
    except Exception:
        input_data = {}

    session_id = str(input_data.get("session_id", "unknown"))
    log(f"PRE-COMPACT triggered for session {session_id}")

    # Optional: run mempalace ingest synchronously so memories land before compaction.
    if MEMPAL_DIR and os.path.isdir(MEMPAL_DIR):
        try:
            with open(LOG_FILE, "a", encoding="utf-8") as lf:
                subprocess.run(
                    [sys.executable, "-m", "mempalace", "mine", MEMPAL_DIR],
                    stdout=lf,
                    stderr=lf,
                    check=False,
                )
        except Exception as e:
            log(f"Failed to run auto-ingest: {e}")

    response = {
        "systemMessage": "⚠️  [Gemini CLI] COMPACTION IMMINENT: Context summary is about to occur. Detailed session history may be lost. Please save important topics, decisions, or code to MemPalace manually before continuing if necessary."
    }
    print(json.dumps(response))


if __name__ == "__main__":
    main()
