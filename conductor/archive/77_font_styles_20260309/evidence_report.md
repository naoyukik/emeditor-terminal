# Evidence Report: Track #77 フォントスタイルの外部化と描画への反映

## 調査レポート (Evidence Report)

- **[Source URL]**:
  - https://learn.microsoft.com/windows/win32/api/wingdi/ns-wingdi-logfontw
  - https://learn.microsoft.com/windows/win32/api/commdlg/ns-commdlg-choosefontw
  - `sdk/plugin-library/plugin.h` (Internal EmEditor SDK)
- **[Context]**:
  - フォントの Weight (太字) および Italic (斜体) 属性を Win32 GDI で扱うための仕様調査。
  - EmEditor SDK におけるフォント操作定数および設定永続化手法の確認。
- **[Key Findings]**: 
  - **Win32 LOGFONTW**:
    - `lfWeight`: `i32`型。400 (`FW_NORMAL`), 700 (`FW_BOLD`) 等。
    - `lfItalic`: `u8`型。1 (`TRUE`) で斜体。
  - **Win32 CHOOSEFONTW**:
    - ユーザーの選択結果は `lpLogFont` 経由で取得可能。
  - **EmEditor SDK**:
    - フォントスタイル定数: `SMART_COLOR_FONT_BOLD` (2), `SMART_COLOR_FONT_ITALIC` (3), `BITS_FONT_BOLD` (0x10), `BITS_FONT_ITALIC` (0x20)。
    - コマンド ID: `EEID_FONT` (4218) で標準のフォントダイアログを開ける。
    - 情報取得: `EI_GET_VIEW_FONT` (382) 等でエディタ側のフォント設定を参照可能。
- **[Constraint Alignment]**:
  - **設計判断**: プラグイン独自のターミナルフォント設定を保持するため、SDK の共通設定参照ではなく、以前の Issue #110 で確立した `EE_REG_QUERY_VALUE` / `EE_REG_SET_VALUE` による独自キーの読み書きを継続採用する。
  - **規約遵守**: Win32 API 型 (`LOGFONTW`) は `Infrastructure` レイヤー（永続化）と `GUI` レイヤー（描画・ダイアログ）に閉じ込め、`Domain` 層 (`TerminalConfig`) はプリミティブ型 (`i32`, `bool`) で管理する。
