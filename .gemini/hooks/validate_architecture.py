#!/usr/bin/env python3
import sys
import json
import os

# ---------------------------------------------------------
# アーキテクチャ定義: { 接尾辞 : 必須ディレクトリ名 }
# ---------------------------------------------------------
# キー: ファイルの接尾辞
# 値: そのファイルが存在しなければならないディレクトリ名（パスの一部に含まれていればよい）
ARCH_RULES = {
    # Presentation Layer
    "_resolver.py": "presentation",
    "_request.py": "presentation",
    "_response.py": "presentation",

    # Application Layer
    "_workflow.py": "application",

    # Domain Layer
    "_entity.py": "domain",
    "_value.py": "domain",
    "_repository.py": "domain",  # インターフェース定義

    # Infrastructure Layer
    "_repository_impl.py": "infrastructure",
}

# 例外的に許可するファイル（プロジェクトルートや設定ファイルなど）
WHITELIST_FILES = [
    "main.py",
    "app.py",
    "__init__.py",
    "conftest.py",
    "setup.py"
]

def log(message):
    """stderrに出力（ログ用）"""
    sys.stderr.write(f"[ArchGuard] {message}\n")

def send_response(decision, reason=None, system_message=None):
    """stdoutにJSONを出力（Gemini CLIへの応答）"""
    response = {"decision": decision}
    if reason:
        response["reason"] = reason
    if system_message:
        response["systemMessage"] = system_message
    print(json.dumps(response))

def main():
    try:
        # stdinから入力を読み込む
        input_str = sys.stdin.read()
        if not input_str:
            return

        input_data = json.loads(input_str)
        tool_input = input_data.get("tool_input", {})

        # ファイルパスの取得
        file_path = tool_input.get("path") or tool_input.get("file_path")

        # Pythonファイル以外は監視対象外
        if not file_path or not file_path.endswith(".py"):
            send_response("allow")
            return

        filename = os.path.basename(file_path)
        # パスをディレクトリ要素に分解 (例: "src/domain/user/file.py" -> ["src", "domain", "user"])
        # 小文字に正規化して判定する
        path_parts = os.path.normpath(os.path.dirname(file_path)).lower().split(os.sep)

        # 1. ホワイトリストチェック
        if filename in WHITELIST_FILES:
            send_response("allow")
            return

        # 2. 接尾辞とディレクトリの整合性チェック
        matched_suffix = None
        required_dir = None

        # ファイル名がいずれかの接尾辞に該当するかチェック
        for suffix, layer_dir in ARCH_RULES.items():
            if filename.endswith(suffix):
                matched_suffix = suffix
                required_dir = layer_dir
                break

        # 接尾辞ルールにマッチしなかった場合（未知のファイル名）
        if not matched_suffix:
            allowed_suffixes = ", ".join(ARCH_RULES.keys())
            reason = (
                f"ファイル名 '{filename}' は許可されていない形式です。\n"
                f"アーキテクチャで定義された接尾辞を使用してください: {allowed_suffixes}"
            )
            log(f"BLOCKED (Invalid Name): {file_path}")
            send_response("deny", reason, f"🚫 命名規則違反: {filename} は不正です。")
            return

        # 3. ディレクトリ配置チェック
        # 必須ディレクトリ名がパスのどこかに含まれているか (例: "infrastructure" が "src/infrastructure/db" に含まれるか)
        if required_dir not in path_parts:
            reason = (
                f"配置エラー: '{filename}' は '{required_dir}' レイヤーに属するファイルですが、"
                f"現在のパス '{os.path.dirname(file_path)}' にはそのディレクトリが含まれていません。\n"
                f"正しい配置場所: .../{required_dir}/..."
            )
            log(f"BLOCKED (Wrong Layer): {file_path}")
            send_response("deny", reason, f"🚫 配置違反: {required_dir} ディレクトリに配置してください。")
            return

        # 全てのチェックを通過
        send_response("allow")

    except Exception as e:
        # エラー時はログを出して安全側に倒す（許可する）
        log(f"ERROR: {str(e)}")
        send_response("allow")

if __name__ == "__main__":
    main()

