# Evidence Report - Mouse Event Pass-through (SGR 1006)

## Discovery Summary
Issue #26 の要件に基づき、マウス操作を VT シーケンスに変換して ConPTY へ送信する機能を実装する。Windows Terminal の挙動を基準とし、特に SGR 1006 エンコーディングと Shift キーによるバイパス（ローカル選択優先）をサポートする。

## Codebase Findings

### 1. メッセージ捕捉と配信
- **ファイル**: `src/gui/resolver/window_message_resolver.rs`
- **内容**: 現在は `on_lbuttondown` でフォーカスを当てる処理のみ。ここに `on_rbuttondown`, `on_mbuttondown`, `on_mousemove` などを追加し、`TerminalWorkflow` へイベントを渡す必要がある。
- **座標変換**: ピクセル座標からセル座標への変換は、`renderer.get_metrics()` から得られる文字幅・高さを使用して行う。

### 2. モード管理
- **ファイル**: `src/domain/model/terminal_buffer_entity.rs`
- **内容**: 以下の情報を保持するフィールドを追加する。
  - `mouse_tracking_mode`: None, Default (1000), ButtonEvent (1002), AnyEvent (1003)
  - `use_sgr_mouse_encoding`: bool (1006)
- **ファイル**: `src/domain/service/terminal_protocol_handler.rs`
- **内容**: `csi_dispatch` 内で `?1000h`, `?1002h`, `?1003h`, `?1006h` などのシーケンスをパースし、バッファのモードを更新する。

### 3. VT シーケンス変換
- **ファイル**: `src/domain/service/vt_sequence_translator_domain_service.rs`
- **内容**: `translate_mouse_event(button, x, y, modifiers, is_release)` メソッドを追加する。
- **形式**: `\x1b[<Pb;Px;PyM` (プレス) / `m` (リリース)。
- **ボタン番号 (Pb)**:
  - 左: 0, 中: 1, 右: 2
  - 移動: +32
  - スクロール: 64 (Up) / 65 (Down)
  - Shift: +4, Alt: +8, Ctrl: +16

### 4. 競合解決
- **ロジック**: `window_message_resolver.rs` にて `GetKeyState(VK_SHIFT)` を確認し、押下されている場合は `TerminalWorkflow` への送信を行わず、`WindowGuiDriver::default_window_proc` を呼び出す。

## Clarifying Questions
1. **マウス移動イベントの頻度**: `WM_MOUSEMOVE` をそのまま送ると ConPTY への負荷が高くなる可能性がある。前回の座標から変化があった場合のみ送る制御が必要か？
   - *回答案*: Windows Terminal も座標変化時のみ送っているため、同様の実装とする。
2. **ダブルクリックの扱い**: `WM_LBUTTONDBLCLK` などは個別に扱う必要があるか？
   - *回答案*: 多くの TUI は DOWN/UP のペアで判断するため、DOWN として扱えば十分である。

## Architecture Options

### Option A: Minimal Implementation
- `window_message_resolver` 内で直接シーケンスを組み立てて送る。
- **短評**: 責務の分離に反する（GUI 層がプロトコルを知りすぎている）。

### Option B: Layered Implementation (Recommended)
- GUI 層: ピクセルからセル座標への変換と、Shift キー判定のみ行う。
- Application 層 (`TerminalWorkflow`): 現在のモードを確認し、変換が必要なら Domain Service を呼ぶ。
- Domain 層 (`VtSequenceTranslator`): シーケンスの生成ロジックに専念する。
- **短評**: 「Strict Rigid レイヤードアーキテクチャ」に合致し、テストも容易。

## Recommendation
**Option B** を採用する。

## Evidence
- `external_sdk.local\microsoft\terminal\src\terminal\input_value\mouseInput.cpp`: SGR 1006 のエンコーディングロジックを確認。
- `external_sdk.local\microsoft\terminal\src\host\VtIo.cpp`: マウスモード有効化のシーケンスを確認。
