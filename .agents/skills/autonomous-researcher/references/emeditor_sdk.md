# EmEditor SDK Reference for Rust/Win32

本ドキュメントは、EmEditor プラグイン開発における重要な仕様、特に Rust での実装時に陥りやすい罠と正しい対処法をまとめたリファレンスである。

## 1. 重要なメッセージ ID (plugin.h より)

EmEditor から `PlugInProc` に送られるメッセージ ID は `WM_USER (1024)` を基点としている。

| 定数名 | 計算式 | 値 (10進) | 役割 |
| :--- | :--- | :--- | :--- |
| `EP_FIRST` | `WM_USER + 0x500` | **2304** | メッセージの開始オフセット |
| `EP_QUERY_PROPERTIES` | `EP_FIRST + 0` | **2304** | 設定ボタンの有無を返す |
| `EP_SET_PROPERTIES` | `EP_FIRST + 1` | **2305** | 設定ダイアログを表示する |
| `EP_GET_INFO` | `EP_FIRST + 10` | **2314** | プラグイン情報を返す |
| `EP_PRE_TRANSLATE_MSG`| `EP_FIRST + 11` | **2315** | メッセージループのフック |

### ⚠️ 重大な罠: EP_PRE_TRANSLATE_MSG (2315)
`2315` はマウス移動やキー入力のたびに **超高速で連打される** メッセージである。ここに重い処理（ダイアログ表示等）を記述すると、EmEditor が即座にフリーズする。設定ダイアログの実装時は、必ず `2305` と取り違えないよう注意せよ。

## 2. 実装パターンとベストプラクティス

### 2.1 設定ダイアログの表示 (PlugInProc)
`EP_SET_PROPERTIES` 受信時にダイアログを表示する際、以下の制御を徹底すること。

1.  **二重表示防止フラグ**: `DialogBoxParamW` はモーダルだが、EmEditor のメッセージポンプ経由で再帰的に `EP_SET_PROPERTIES` が飛んでくる可能性がある。`AtomicBool` 等で表示中フラグを管理し、重複呼び出しを無視せよ。
2.  **正しい親ウィンドウの取得**:
    - `wParam` に親 HWND が入るのが標準だが、環境により `lParam` に格納されるケースがある。
    - `let raw_hwnd = if w_param.0 != 0 { w_param.0 } else { l_param.0 as usize };` のように柔軟に取得すること。
3.  **戻り値**: 処理した場合は `LRESULT(1)` (TRUE) を返すのが基本。

### 2.2 アーキテクチャ分離 (Suffix Rule)
SDK 固有の処理は以下のレイヤーに閉じ込める。

- **Resolver (`_resolver.rs`)**: SDK メッセージ (`nMsg`) を解釈し、内部 DTO へ変換して Application 層へ。
- **IO Driver (`_io_driver.rs`)**: `Editor_DocGetLines` などの SDK マクロ（実体は `SendMessage`）をラップし、外部操作を封印する。

## 3. SDK サンプルの参照先
プロジェクト内の `sdk/` ディレクトリに以下の重要ファイルが存在する。

- `sdk/hello_plugin/plugin.h`: 定数定義の一次ソース。
- `sdk/hello_plugin/etlframe.h`: `PlugInProc` や `OnEvents` の C++ 実装例。
