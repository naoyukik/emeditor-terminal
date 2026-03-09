# Specification for Track #77: フォントスタイルの外部化と描画への反映

#### 1. Overview
Issue #77 に基づき、ターミナルで使用するフォントのスタイル（太字/斜体/Weight 等）を外部の設定（レジストリ/INI）から読み込み、描画エンジン (`TerminalGuiDriver`) へ反映させる。

#### 2. Functional Requirements
- **ドメインモデルの拡張**: `TerminalConfig` に `font_weight` (i32) および `font_italic` (bool) を追加。
- **設定ダイアログの強化**: `ChooseFontW` 連携時に `lfWeight`, `lfItalic` を取得し、`TerminalConfig` に反映・保存する。
- **永続化層の更新**: `EmEditorConfigRepositoryImpl` において、`Weight`, `Italic` を `[Terminal]` セクションに保存・読込可能にする。
- **描画反映**: `TerminalGuiDriver` のフォント生成ロジックを更新し、保存された設定値（Weight/Italic）を使用して `HFONT` を作成する。

#### 3. Non-Functional Requirements
- **一貫性の保持**: 設定の反映タイミングは、Issue #110 と同様に「エディタの再起動またはターミナルの再開時」とする。
- **下位互換性**: 既存の設定（書体・サイズのみ）が存在する場合でも、デフォルト値（FW_NORMAL, false）で正常に動作すること。

#### 4. Acceptance Criteria
- 設定ダイアログで「MS ゴシック、10pt、太字、斜体」等を選択して保存できる。
- 保存した設定が、ターミナル内の文字描画に目視で反映されている（太字・斜体の描画確認）。
- EmEditor を再起動しても、選択したスタイルが維持されている。

#### 5. Out of Scope
- 下線 (Underline)、取り消し線 (StrikeOut) のサポート。
- 設定変更後のリアルタイム描画反映（Live Reload）。
