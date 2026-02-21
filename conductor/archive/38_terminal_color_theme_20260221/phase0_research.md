# Phase 0: SDKを通じた構成保存方法の調査結果

## 調査目的
EmEditorプラグインとしてターミナルのテーマや設定（構成情報）を永続化するための標準的な手段と、EmEditorのワークスペース機能（`.code-workspace` など）に対応するAPIの有無を確認する。

## 調査内容
`sdk/` 配下のヘッダファイル（主に `plugin.h`, `etlframe.h`）およびドキュメント（`EE_INFO`等のメッセージリファレンス）を調査した。

## 調査結果

### 1. 標準的な構成保存手段
EmEditorプラグインとして設定を保存するための標準APIとして、以下のマクロおよびメッセージが提供されていることが確認できた。

- **`Editor_RegSetValue` / `EE_REG_SET_VALUE`**
- **`Editor_RegQueryValue` / `EE_REG_QUERY_VALUE`**

これらの関数は、EmEditor本体の設定（レジストリを使用するか、INIファイルを使用するか）に応じて、**透過的に保存先を切り替えてくれる**という強力な利点がある。
キーとして `EEREG_EMEDITORPLUGIN` または `EEREG_PLUGINS` を指定し、プラグイン名を `pszConfig` に渡すことで、レジストリ（`HKEY_CURRENT_USER\Software\EmSoft\EmEditorPlugIns`）または INIファイル（`eePlugins.ini`）の該当セクションへ値を保存・取得できる。

### 2. ワークスペース（`.code-workspace` 等）との連携API
`sdk/` 配下で `workspace` や `code-workspace` に関連するマクロを調査した結果、以下のようなコマンドIDは存在した。
- `EEID_WORKSPACE_OPEN`
- `EEID_WORKSPACE_SAVE_AS`
- `EEID_LOAD_WORKSPACE`

しかし、これらはEmEditor本体の「ワークスペースを開く/保存する」コマンドを呼び出すためのものであり、**「特定のプラグインの独自設定をワークスペースファイル内に書き込む/読み込む」ための専用APIは提供されていない**。
EmEditorのワークスペース機能は基本的に開いているファイルやウィンドウ配置、マクロの状態などを保存するものであり、プラグイン側から自由にデータを拡張して永続化するエンドポイント（イベントハンドラ等）は見当たらなかった。

## 構成保存方針の決定
以上の調査に基づき、ターミナルプラグインの構成情報（テーマ設定など）の保存方針を以下のように決定する。

- **採用技術**: EmEditor標準のプラグイン設定保存API（`Editor_RegQueryValue` / `Editor_RegSetValue`）を使用する。
- **理由**:
  1. ユーザーのEmEditor環境（ポータブル版でINIを使うか、インストール版でレジストリを使うか）に自動的に適応できるため、最も安全かつ標準的なアプローチである。
  2. ワークスペースファイルへの直接介入はSDKでサポートされておらず、独自パース等を行うとEmEditor本体の動作と競合するリスクがあるため見送る。

この方針に従い、Phase 2 の「構成情報のロード処理」は、C++側から `Editor_RegQueryValue` で取得した設定文字列（テーマ名など）をRust側に渡す、あるいはRust側からWin32メッセージを用いて直接EmEditorに問い合わせる形で実装する。