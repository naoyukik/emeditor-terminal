# Evidence Report: システムキャレット同期による IME 位置解決

## 1. 調査概要 (Research Overview)
- **対象**: ConPTY 環境下での IME 候補ウィンドウ表示位置の不整合
- **目的**: Gemini CLI 等の TUI アプリケーションで、IME 候補ウィンドウが仮想カーソルに正しく追従しない問題を解決する。

## 2. 参照リソース (References)
- [Microsoft Learn: Carets](https://learn.microsoft.com/windows/win32/menurc/carets)
- [Microsoft Learn: SetCaretPos function](https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-setcaretpos)
- [GitHub: Windows Terminal - IME candidate window position issue](https://github.com/microsoft/terminal/issues)
- [GitHub: Alacritty - Windows IME positioning](https://github.com/alacritty/alacritty/issues)

## 3. 調査結果 (Findings)
- **システムキャレットの役割**: IME 候補ウィンドウは Windows の「システムキャレット」にアンカーされる。
- **能動的同期の必要性**: TUI アプリが自前でカーソルを描画する場合でも、`SetCaretPos` でシステムキャレットを同期し続けなければ、IME はフォーカス位置を見失う。
- **不可視設定**: `SetCaretPos` はキャレットが非表示 (`HideCaret`) であっても動作し、IME はその見えない座標を参照する。
- **実装の定石**: モダンなターミナル（Windows Terminal 等）は、VT シーケンスによる座標更新のたびにシステムキャレットを更新している。

## 4. プロジェクト規約との整合性 (Local Alignment)
- **レイヤー責務**: Win32 API (`SetCaretPos`, `CreateCaret`) は `Infrastructure` 層の `ImmGuiDriver` に封印する。
- **RAII パターン**: キャレットの生成・破棄を RAII 構造体で管理し、リソースリークを防ぐ。

## 5. 推奨される実装方針 (Recommended Implementation)
- `ImmGuiDriver` に 1x1 または 0x0 サイズの不可視キャレットを作成する。
- 仮想カーソル座標が変更されるすべてのパス（描画、シーケンス処理）で `SetCaretPos` を呼び出す。
- `TerminalWorkflow` には依存させず、`Application` 層または `Driver` 層で完結させる。
