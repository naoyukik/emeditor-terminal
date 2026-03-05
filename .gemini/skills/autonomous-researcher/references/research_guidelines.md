# リサーチクエリのガイドライン

効果的な調査を行うための検索クエリとアプローチの指針。

## 1. Win32 API 調査
- **Query**: `site:learn.microsoft.com [API名] [エラーコード/定数]`
- **Check**:
  - 対応OSバージョン (ConPTYは1809以降など)
  - 戻り値が `HRESULT` か `BOOL` か
  - `GetLastError()` の呼び出しが必要か

## 2. Rust クレート/エコシステム調査
- **Query**: `[crate名] documentation [メソッド名]`
- **Check**:
  - `docs.rs` の最新バージョン
  - GitHub Issue での既知のバグや Regression
  - Rustの `unsafe` セーフティガイドライン

## 3. TUI/ターミナルエミュレーション調査
- **Query**: `VT escape sequence [CSI/DSR] behavior [xterm/vte]`
- **Check**:
  - 各ターミナル（Windows Terminal, VSCode, iTerm2等）での挙動の差異
  - 制御文字のパース規則
