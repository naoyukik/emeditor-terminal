# Evidence Report: Fix Issue 135 (Terminal text overflow at startup)

## Discovery Summary
ターミナル起動時、特に Gemini CLI 実行時にテキストが右端を突き抜けたり、下部にはみ出したりする問題の根本原因を調査した。

### 課題
- 起動直後の ConPTY 論理サイズ（Cols/Rows）が物理的なウィンドウサイズと一致していない。
- Gemini CLI 等の起動が速いアプリでは、リサイズ命令が届く前に最初の出力が行われ、不適切な折り返しが発生する。

### 制約
- ウィンドウ作成直後は `GetClientRect` が正しい値を返さない（EmEditor 側での配置・リサイズ前）。
- `TerminalGuiDriver` のフォントメトリクスが初回の描画まで `None` である。

## Codebase Findings
### 根本原因の特定
1. **`src/gui/window/mod.rs` の `open_custom_bar`**:
   - `CreateWindowExW` 直後に `GetClientRect` を呼び出しているが、この時点ではサイズが 0 もしくは不正確である可能性が高い。
   - `renderer.get_metrics()` が `None` を返すため、フォールバック `(8, 16)` でカラム数が計算されている。
2. **フォントメトリクスの遅延初期化**:
   - `src/gui/driver/terminal_gui_driver.rs` の `get_font_for_style` 内で初めて計算されるため、起動時のサイズ計算に間に合っていない。

### キーとなる修正箇所
- `src/gui/driver/terminal_gui_driver.rs`: フォント設定変更時や初期化時にメトリクスを計算するパブリックメソッドを追加。
- `src/gui/window/mod.rs`: 
  - `open_custom_bar` で HDC を取得し、`renderer` のメトリクスを初期化。
  - `WM_SIZE` メッセージ受信時に、ConPTY へのリサイズ命令が確実かつ最優先で行われるように調整。

## Clarifying Questions (解決済み)
- 推奨案: 起動時サイズの厳密な計算（フォントメトリクスの先行取得）。
- スコープ: Issue 135 全体（右端および下部のはみ出し）。

## Architecture Options
- **Option A (推奨案)**: 起動前にフォントメトリクスを確定させ、`WM_CREATE` または `open_custom_bar` 内で正確なサイズで ConPTY を起動する。
- **Option B**: 初回の `WM_SIZE` が来るまで ConPTY の起動を遅延させる。UI の応答性が低下する懸念がある。

## Recommendation
**Option A** を採用。`TerminalGuiDriver` が自身のメトリクスを能動的に更新できるようにし、`open_custom_bar` でそれを活用する。

## Expected Behavior
- ターミナル起動時に `UDEV Gothic 35JPDOC 14pt` 等のフォントメトリクスを正確に取得し、現在のウィンドウサイズに収まるカラム数を算出する。
- Gemini CLI 起動直後から、右端で正しく折り返され、下部が見切れない状態を維持する。
