# Track Specification: Issue #74 - 設定基盤の構築 (EmEditor Config API)

## 1. 概要 (Overview)
ターミナルの構成設定（フォント、シェルパス等）をハードコードから脱却させ、EmEditor 標準の設定機構（レジストリ/INI）を通じて永続化・ロードできる基盤を構築する。

## 2. 機能要件 (Functional Requirements)
- **設定データ構造の定義**: 以下の項目を含む `TerminalConfig` 構造体を定義する。
    - `font_face`: 使用するフォント名 (String)
    - `font_size`: フォントサイズ (i32)
    - `shell_path`: 起動するシェルのフルパス (String)
- **EmEditor Config Repository の実装**: `EE_REG_QUERY_VALUE` および `EE_REG_SET_VALUE` メッセージを使用し、設定項目を個別のキーとして読み書きする `ConfigRepository` トレイトの実装。
- **初期値 (Default) の提供**: 設定が存在しない場合に、システム標準（例: Consolas, 10pt, pwsh.exe/powershell.exe）を適切に解決して提供する。
- **Application 層への統合**: `TerminalWorkflow` の初期化時に設定をロードし、各コンポーネントに設定値を伝搬させる。

## 3. 非機能要件 (Non-Functional Requirements)
- **ポータビリティ**: EmEditor の「ポータブル版（INI保存）」設定が有効な場合、SDK 経由で透過的に INI ファイルへの読み書きが行われること（`EE_REG_QUERY_VALUE` の仕様に準拠）。
- **型の安全性**: 文字列や数値の境界チェックを行い、不正な設定値が読み込まれてもアプリケーションがクラッシュしないようにする。

## 4. 承認基準 (Acceptance Criteria)
- [ ] `TerminalConfig` 構造体が定義され、必要な項目が網羅されている。
- [ ] 設定が存在しない初回起動時に、デフォルト値が正しくロードされる。
- [ ] レジストリ/INI の値を手動で書き換えた後、プラグインを再起動するとその値が反映される。
- [ ] （オプション）設定の保存処理を呼び出すと、指定したキーでレジストリ/INI に値が書き込まれる。

## 5. スコープ外 (Out of Scope)
- 設定画面（UI）の実装（本トラックは基盤のみ）。
- カラーテーマの外部化（後続の Issue #76 で実施）。
- 設定変更のリアルタイム監視（Issue #79）。
