# Specification - PR #40 Review Fixes

## 概要
PR #40 で指摘されたバグ、潜在的なクラッシュ要因、およびドキュメントの不備を修正する。

## 修正対象
1. **SGRパースの無限ループ (Critical)**:
   - `src/domain/terminal.rs` において、不正な SGR 38/48 シーケンス（サブパラメータ不足や不正な型）が来た場合にインデックス `i` が進まず、無限ループになる問題を修正する。
2. **Dim属性の適用範囲 (Bug)**:
   - `src/gui/renderer.rs` において、RGBカラー以外（ANSI, Xterm）に対しても Dim（輝度低減）属性が適用されるようにする。COLORREF変換後にRGB値を操作するロジックを導入する。
3. **フォント生成の安全性 (Safety)**:
   - `src/gui/renderer.rs` において、フォント生成 (`CreateFontIndirectW`) 失敗時に `HFONT::default()` (NULL) を返さず、デフォルトフォントへのフォールバックや適切なエラー処理を行う。
4. **ドキュメントとREADME (Docs)**:
   - `SendHFONT` の安全性に関するドキュメント（`# Safety` セクション）を復元する。
   - `README.md` の「バイブコーディング」という表現を修正する。
   - `src/domain/terminal.rs` にコロン区切りパースに関する設計意図のコメントを追加する。

## 受け入れ条件
- 不正な SGR シーケンス（例: `\x1b[38;3m`）を与えてもハングアップしないこと。
- ANSIカラー（例: `\x1b[31;2m`）でDim属性が視覚的に反映されること。
- コンパイル警告がないこと。
- PRのコメントすべてに対応していること。
