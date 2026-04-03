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

## 4. リファレンス実装 (Windows Terminal)
- **Repository**: [microsoft/terminal](https://github.com/microsoft/terminal)
- **Local Path**: `external_sdk.local/microsoft/terminal/`
- **Search Method**:
  - `grep_search` を使用し、ローカルディレクトリ内のコードをキーワードや定数で網羅的に検索せよ。
  - ウェブ検索 (`site:github.com/microsoft/terminal`) よりも、ローカル検索によるコンテキスト把握を優先する。
- **Key Components to Research (Local Paths)**:
  - **ConptyHost (`src/host/`)**: ConPTY のホスト側の実装。プロセスの起動、パイプ、I/O 制御。
  - **TerminalCore (`src/terminal/parser/`)**: VT100/Xterm シーケンスのパース、ステートマシンの実装。
  - **TerminalAdapter (`src/terminal/adapter/`)**: シーケンスをアクションに変換するロジック。
  - **IME Support (`src/cascadia/TerminalControl/`)**: IME 制御（TSF/IMM32）の挙動、キャンバスへのレンダリング。
- **Check**:
  - `HRESULT` のエラー処理や、特定のエッジケース（日本語入力の不具合修正履歴など）。
  - 各シーケンスに対する標準的な挙動と、下位互換性のための対応。
