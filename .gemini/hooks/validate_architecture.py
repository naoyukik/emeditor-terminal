#!/usr/bin/env python3
import sys
import json
import os
import re

# アーキテクチャ定義
ARCH_RULES = {
    "_resolver.rs": "gui/resolver",
    "_gui_driver.rs": "gui/driver",
    "_request.rs": "gui",
    "_response.rs": "gui",
    "_workflow.rs": "application",
    "_input.rs": "application",
    "_result.rs": "application",
    "_entity.rs": "domain/model",
    "_value.rs": "domain/model",
    "_domain_service.rs": "domain/service",
    "_repository.rs": "domain/repository",
    "_repository_impl.rs": "infra/repository",
    "_io_driver.rs": "infra/driver",
}

WHITELIST_FILES = ["mod.rs", "lib.rs", "main.rs", "build.rs"]

def send_response(decision, reason=None, system_message=None):
    response = {"decision": decision}
    if reason: response["reason"] = reason
    if system_message: response["systemMessage"] = system_message
    print(json.dumps(response))

def validate_file(file_path, content=None):
    if not file_path.endswith(".rs"):
        return None

    filename = os.path.basename(file_path)
    path_dir = os.path.dirname(file_path).replace("\\", "/").lower()

    if filename in WHITELIST_FILES:
        return None

    # 1. 命名規則・配置のチェック
    matched_suffix = None
    required_dir = None
    for suffix, layer_dir in ARCH_RULES.items():
        if filename.endswith(suffix):
            matched_suffix = suffix
            required_dir = layer_dir
            break

    if not matched_suffix:
        return f"🚫 命名規則違反: '{filename}' には有効な接尾辞（Suffix Rule）が必要です。"

    if required_dir not in path_dir:
        return f"🚫 配置違反: '{filename}' は '{required_dir}' 配下に配置してください。"

    # 2. Windows API 隔離命令のチェック
    if content and ("domain" in path_dir or "application" in path_dir):
        if re.search(r'\buse\s+windows\b', content) or re.search(r'\bwindows::\b', content):
            return "🚫 隔離命令違反: Domain層およびApplication層で 'windows' クレートを直接使用することは禁じられています。Pure Rust定義を使用せよ。"

    return None

def main():
    try:
        sys.stderr.write("DEBUG: validate_architecture.py CALLED\n")
        input_str = sys.stdin.read()
        if not input_str:
            send_response("allow")
            return

        input_data = json.loads(input_str)
        # Gemini CLI hook input could be the arguments themselves or wrapped
        args = input_data.get("tool_input", input_data)

        # 1. 直接的なファイルパス指定の取得
        file_path = args.get("file_path") or args.get("pathInProject") or args.get("filePath") or args.get("path")
        content = args.get("text") or args.get("content")

        targets = []
        if file_path:
            targets.append((file_path, content))

        # 2. シェルコマンドからのパス抽出
        command = args.get("command", "")
        if command:
            # src/ 配下の .rs ファイルっぽいものを探す
            matches = re.findall(r'(src/[^\s"\'=,]+\.rs)', command)
            for m in matches:
                targets.append((m, None))

        if not targets:
            send_response("allow")
            return

        errors = []
        for path, text in targets:
            err = validate_file(path, text)
            if err:
                errors.append(err)

        if errors:
            send_response("deny", "\n".join(errors), "アーキテクチャの掟に反する操作を検知したため、AcePilotがこれを阻止した。規約を遵守せよ。")
        else:
            send_response("allow")

    except Exception as e:
        sys.stderr.write(f"ERROR: {str(e)}\n")
        send_response("allow")

if __name__ == "__main__":
    main()
