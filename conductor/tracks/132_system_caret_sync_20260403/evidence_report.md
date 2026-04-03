# Evidence Report: Win32 IME Coordinate Requirements

## 1. 調査対象 (Research Targets)
- `SetCaretPos` (Win32 API)
- `ImmSetCompositionWindow` (Win32 API / IMM)
- `COMPOSITIONFORM` (Win32 Structure)

## 2. 参照ソース (Primary Sources)
- [Microsoft Learn: SetCaretPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setcaretpos)
- [Microsoft Learn: COMPOSITIONFORM](https://learn.microsoft.com/en-us/windows/win32/api/imm/ns-imm-compositionform)
- [GitHub: Microsoft Terminal (HwndTerminal.cpp)](https://github.com/microsoft/terminal/blob/main/src/cascadia/TerminalControl/HwndTerminal.cpp)

## 3. 調査結果 (Findings)

### 3.1 `SetCaretPos`
- **座標系**: **クライアント座標 (Client Coordinates)**
- **公式定義**: "Specifies the new x and y coordinates (in client coordinates) of the caret."
- **結論**: 現在の実装（クライアント座標の使用）は正しい。

### 3.2 `ImmSetCompositionWindow` / `ImmSetCandidateWindow` / TSF 同期
- **Windows Terminal の実例**:
    - `external_sdk.local/microsoft/terminal/src/cascadia/TerminalControl/HwndTerminal.cpp` における `TsfDataProvider::GetCursorPosition` の実装を確認。
    - 仮想カーソル位置にフォントサイズを乗じてピクセル計算した後、**`ClientToScreen` を呼び出してスクリーン座標に変換している**。
    - 戻り値として `RECT` を返しており、その `left`, `top`, `right`, `bottom` すべてがスクリーン座標系で構成されている。
- **結論**:
    - `Imm32` API は内部的に TSF にブリッジされており、特にネストされたウィンドウでは明示的なスクリーン座標への変換が「候補窓の位置ズレ」を解消する鍵となる。
    - **`CANDIDATEFORM` の `rcArea` (Exclusion Rect)** も同様に、`ClientToScreen` を適用したスクリーン座標系で指定する必要がある。

## 4. 座標変換の技術的詳細 (Technical Details of Coordinate Transformation)
- `ClientToScreen` は `POINT` 構造体のみを扱う。
- `RECT`（除外領域）を変換する場合、`left/top` と `right/bottom` の 2 点を個別に変換するか、`MapWindowPoints` を使用して一括変換を行う必要がある。
- 本プロジェクトでは、安全性の観点から `Driver` 層で `MapWindowPoints` または個別変換を確実に実行する。

## 4. プロジェクト規約との整合性 (Alignment with Project Rules)
- **GUI Driver 層**: `src/gui/driver/ime_gui_driver.rs` において `ClientToScreen` を使用して座標変換を行うことは、レイヤードアーキテクチャの掟（Win32 API の Driver 層への封印）に合致する。
- **RAII**: `CaretHandle` によるライフサイクル管理を継続し、同期のタイミングのみを修正する。

## 5. 推奨される修正方針 (Recommended Implementation)
1. `SetCaretPos` には引き続きクライアント座標を渡す。
2. `ImmSetCompositionWindow` および `ImmSetCandidateWindow` に渡す `POINT` および `RECT` は、`ClientToScreen` を使用して**スクリーン座標**に変換してから渡す。
