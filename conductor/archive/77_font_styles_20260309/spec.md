# Specification for Track #77: フォントスタイルの外部化と描画への反映

#### 1. Overview
Issue #77 に基づき、ターミナルで使用するフォントの構成情報（書体・サイズ・スタイル等）を外部の設定（レジストリ/INI）から読み込み、描画エンジン (`TerminalGuiDriver`) へ完全に反映させる。すでに保存・読込が実装されている `font_face`, `font_size` に加え、新たに `Weight` と `Italic` をサポートする。

#### 2. Functional Requirements
- **ドメインモデルの拡張**: `TerminalConfig` に `font_weight` (i32) および `font_italic` (bool) を追加。
- **設定ダイアログの強化**: `ChooseFontW` 連携時に `lfWeight`, `lfItalic` を取得し、`TerminalConfig` に反映・保存する。
- **永続化層の更新**: `EmEditorConfigRepositoryImpl` において、`Weight`, `Italic` を `[Terminal]` セクションに保存・読込可能にする。
- **描画反映 (Core)**: 
  - `TerminalGuiDriver` にハードコードされている `Consolas` と `lfHeight: 16` を排除し、`TerminalConfig` の `font_face` と `font_size` に置き換える。
  - 同時に、新規追加した `font_weight` と `font_italic` を `HFONT` 生成ロジック（`CreateFontIndirectW`）に適用する。

#### 3. Non-Functional Requirements
- **一貫性の保持**: 設定の反映タイミングは、Issue #110 と同様に「エディタの再起動またはターミナルの再開時」とする。
- **下位互換性**: 既存の設定（書体・サイズのみ）が存在する場合でも、デフォルト値（FW_NORMAL, false）で正常に動作すること。

#### 4. Acceptance Criteria
- 設定ダイアログで「任意のフォント（例: MS ゴシック）、任意のサイズ、太字、斜体」等を選択して保存できる。
- 保存した設定が、ターミナル内の文字描画にすべて目視で反映されている（書体、サイズ、太字、斜体）。
- EmEditor を再起動しても、選択したスタイルとフォント構成が維持されている。

#### 5. Out of Scope
- 下線 (Underline)、取り消し線 (StrikeOut) のサポート。
- 設定変更後のリアルタイム描画反映（Live Reload）。
