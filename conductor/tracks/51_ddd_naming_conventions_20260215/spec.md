# Specification: Track 51 - DDDに基づく命名規則の適用

## 1. 概要
本トラックでは、Issue #51に基づき、コードベース全体の変数名、フィールド名、および型名を監査し、ドメイン駆動設計（DDD）に基づく明確な命名規則を適用する。

## 2. 目的
- 汎用的な変数名（`tmp`, `data`, `buf`）を、その役割を示すドメイン用語（`window_data`, `input_buffer`等）に置き換える。
- boolean型変数を述語形式（`is_xxx`, `has_xxx`, `can_xxx`）に統一する。
- ガイドライン（`architecture_rules.md` および `rust_win32.md`）に定義された命名規則を強制し、コードの可読性を向上させる。

## 3. 実施事項
### 3.1 汎用名の排除
- `src/gui/window/` 配下等で多用されている `data`（`TerminalWindowData`への参照）を、`window_data` 等の具体的な名称に変更する。
- `send_input` 等の引数 `data: &[u8]` を `input_bytes` 等に変更する。
- その他、`buf`, `tmp`, `res` 等の無機質な名称を監査し、意味のある名称へリネームする。

### 3.2 Boolean変数の形式統一
- `src/domain/terminal.rs` の属性フラグ（`bold`, `italic`, `visible` 等）を `is_bold`, `is_italic`, `is_visible` 等に変更する。
- `src/domain/model/input.rs` 等の修飾キーフラグ（`ctrl`, `shift`, `alt`）を、述語形式（`is_ctrl_pressed`等、または規約に沿った形式）に検討・統一する。

### 3.3 ユビキタス言語の適用
- ターミナルの「バッファ」や「履歴」に関連する用語が、全レイヤーで一貫して使用されていることを確認・修正する。

## 4. 非機能要件
- **動作の維持**: ロジックの変更は行わず、純粋な識別子のリネームのみを行う。
- **段階的な検証**: 各モジュールごとのリネーム後にビルドを行い、既存機能への影響がないことを確認する。

## 5. 受け入れ基準 (Acceptance Criteria)
- [ ] プロジェクト全体から、意味の不明瞭な汎用変数名（`data`, `buf`等）が駆逐されている。
- [ ] すべての boolean 変数が述語形式（`is_`, `has_`, `can_`等）になっている。
- [ ] 変更後もプロジェクトが正常にビルド・動作する。
