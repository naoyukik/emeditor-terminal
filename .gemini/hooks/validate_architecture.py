#!/usr/bin/env python3
import sys
import json
import os
import re

# アーキテクチャ定義 (Suffix vs Directory)
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
    "_protocol_handler.rs": "domain/service",
    "_repository.rs": "domain/repository",
    "_repository_impl.rs": "infra/repository",
    "_io_driver.rs": "infra/driver",
}

# 依存許可ルール (From Layer -> Allowed Target Paths)
DEPENDENCY_RULES = {
    "gui/resolver": ["application", "domain", "gui/driver"],
    "gui/driver": ["domain"],
    "application": ["domain"],
    "domain": [],
    "infra/repository": ["domain", "infra/driver"],
    "infra/driver": ["domain"],
}

WHITELIST_FILES = ["mod.rs", "lib.rs", "main.rs", "build.rs", "resource.rs"]

def send_response(decision, reason=None, system_message=None):
    # decision は "allow" または "deny"
    response = {"decision": decision}

    if reason:
        # reason は AIエージェントへの修正指示
        response["reason"] = reason

    if system_message:
        # systemMessage がユーザーの画面に直接表示される
        response["systemMessage"] = system_message

    # stdout に JSON を出力
    print(json.dumps(response))
    sys.stdout.flush()

def check_naming_and_location(file_path):
    filename = os.path.basename(file_path)
    path_dir = os.path.dirname(file_path).replace("\\", "/").lower()

    if filename in WHITELIST_FILES:
        return None

    matched_suffix = None
    required_dir = None
    for suffix, layer_dir in ARCH_RULES.items():
        if filename.endswith(suffix):
            matched_suffix = suffix
            required_dir = layer_dir
            break

    if not matched_suffix:
        return f"🚫 命名規則違反: '{filename}' には有効な接尾辞（Suffix Rule）が必要です。architecture_rules.md を確認せよ。"

    if required_dir not in path_dir:
        return f"🚫 配置違反: '{filename}' は '{required_dir}' 配下に配置してください。"

    return None

def check_windows_api_isolation(file_path, content):
    if not content:
        return None

    path_dir = os.path.dirname(file_path).replace("\\", "/").lower()
    if "domain" in path_dir or "application" in path_dir:
        if re.search(r'\buse\s+windows\b', content) or re.search(r'\bwindows::\b', content):
            return "🚫 隔離命令違反: Domain層およびApplication層で 'windows' クレートを直接使用することは禁じられています。Pure Rust定義を使用せよ。"
    return None

def validate_dependence(file_path, content):
    if not content:
        return None

    path_dir = os.path.dirname(file_path).replace("\\", "/").lower()

    current_layer = None
    for suffix, layer_dir in ARCH_RULES.items():
        if layer_dir in path_dir:
            current_layer = layer_dir
            break

    if not current_layer:
        return None

    allowed_targets = DEPENDENCY_RULES.get(current_layer, [])
    
    # use crate::... または完全修飾パス crate::... を検出
    all_refs = re.findall(r'\bcrate::([^\s;:(]+)', content)

    for ref in all_refs:
        # 具象実装への直接依存チェック
        if "infra" in ref and "impl" in ref:
            # mod.rs 等の WHITELIST は既に除外されているが、
            # resolver からの具象依存も原則禁止（Composition Root は lib.rs 等に寄せる）
            return f"🚫 DIの掟違反: 具象実装 '{ref}' を直接参照することは禁じられています。Repository Trait を使用せよ。"

        # 許可レイヤーチェック
        is_allowed = False
        for allowed in allowed_targets:
            if ref.startswith(allowed.replace("/", "::")):
                is_allowed = True
                break

        # gui 配下のレイヤー間での crate::gui:: 参照を許容する
        if current_layer.startswith("gui/") and ref.startswith("gui"):
            is_allowed = True

        if ref.startswith(current_layer.replace("/", "::")) or ref.startswith("common") or ref.startswith("get_instance_handle"):
            is_allowed = True

        if not is_allowed:
            return f"🚫 依存の掟違反: レイヤー '{current_layer}' から '{ref}' への依存は許可されていません。"

    return None

def main():
    try:
        raw_input = sys.stdin.read()
        if not raw_input:
            send_response("allow")
            return

        input_data = json.loads(raw_input)
        args = input_data.get("tool_input", {})

        file_path = args.get("file_path") or args.get("pathInProject") or args.get("filePath") or args.get("path")
        content = args.get("text") or args.get("content")

        targets = []
        if file_path:
            targets.append((file_path, content))

        command = args.get("command", "")
        if command:
            matches = re.findall(r'(src/[^\s"\'=,]+\.rs)', command)
            for m in matches:
                targets.append((m, None))

        if not targets:
            send_response("allow")
            return

        errors = []
        for path, text in targets:
            if not path.endswith(".rs"):
                continue

            filename = os.path.basename(path)
            if filename in WHITELIST_FILES:
                continue

            err_naming = check_naming_and_location(path)
            if err_naming: errors.append(err_naming)

            if text:
                err_winapi = check_windows_api_isolation(path, text)
                if err_winapi: errors.append(err_winapi)

                err_dep = validate_dependence(path, text)
                if err_dep: errors.append(err_dep)

        if errors:
            combined_err = "\n".join(errors)
            # 常に allow を返し、警告としてエラーを表示する（ワーニング化）
            send_response("allow", system_message=f"⚠️ アーキテクチャ警告:\n{combined_err}")
        else:
            send_response("allow")

    except Exception as e:
        sys.stderr.write(f"CRITICAL ERROR in hook: {str(e)}\n")
        send_response("allow")

if __name__ == "__main__":
    main()
