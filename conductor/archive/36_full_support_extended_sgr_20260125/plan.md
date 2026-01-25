# Implementation Plan - Issue #36: 拡張SGRシーケンスと属性の完全サポート

## Phase 1: ドメイン層の拡張と SGR パースの強化
`TerminalAttribute` の拡張と、より複雑な SGR パラメータのパースロジックを実装する。

- [x] Task: `TerminalAttribute` への RGB カラーとスタイルフラグの追加
    - [x] `TerminalColor` 列挙型を拡張し、256色と RGB 形式をサポート
    - [x] `TerminalAttribute` に太字、斜体、下線、反転、取り消し線のフラグを追加
- [x] Task: SGR パース処理の単体テスト作成 (`src/domain/terminal.rs`)
    - [x] 256色 (38;5;n) のパーステスト
    - [x] TrueColor (38;2;r;g;b) のパーステスト
    - [x] AIXterm 高輝度カラー (90-97) のパーステスト
    - [x] 複数属性の組み合わせ（例: 太字 + 下線 + RGB色）のパーステスト
- [x] Task: SGR パースロジックの実装
    - [x] `SGR_PARAM_MAX` を超えるパラメータや、セミコロン/コロン区切りの拡張書式に対応
    - [x] `src/domain/terminal.rs` の `handle_sgr` を更新
- [x] Task: Conductor - User Manual Verification 'Phase 1: Domain Logic' (Protocol in workflow.md)

## Phase 2: レンダリングエンジン (GDI) の更新
解析された属性に基づき、Windows GDI を用いて文字を描画する機能を実装する。

- [ ] Task: カラー変換処理の実装 (`src/gui/renderer.rs`)
    - [ ] `TerminalColor` から GDI の `COLORREF` への変換関数を作成
    - [ ] 標準16色、256色、TrueColor の各マッピングを実装
- [ ] Task: 動的なフォント生成と管理の実装
    - [ ] 太字、斜体、下線などの組み合わせに応じた `HFONT` を生成・キャッシュする仕組みの導入
    - [ ] `LOGFONT` 構造体の動的な書き換え処理
- [ ] Task: 反転および取り消し線の描画サポート
    - [ ] 反転 (Inverse) 属性時の背景色/前景色入れ替えロジックの追加
    - [ ] `TextOut` 後の取り消し線 (Strikethrough) 描画処理
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Rendering' (Protocol in workflow.md)

## Phase 3: 統合テストと最終調整
実環境での動作確認と、表示崩れがないかの最終検証を行う。

- [x] Task: TUI アプリケーションを用いた実機検証
    - [x] `powershell.exe` 上での色の確認
    - [x] `Microsoft Edit` または `vim` 等でのシンタックスハイライト、ステータスラインの表示確認
- [x] Task: 256色/TrueColor テストスクリプトの実行
    - [x] オープンソースのターミナルテストスクリプトを用いた全色描画確認
- [x] Task: エッジケースの修正
    - [x] 大量描画時のパフォーマンス確認と、必要に応じたフォントキャッシュの最適化
- [x] Task: Conductor - User Manual Verification 'Phase 3: Integration' (Protocol in workflow.md)
