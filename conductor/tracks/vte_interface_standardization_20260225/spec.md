# Track Specification: vte_interface_standardization_20260225

## 1. Overview
後続の `vte` クレート導入（Issue #94）を見据え、Domain層の `TerminalBufferEntity` に `vte::Perform` トレイト互換の標準的な命令セットを実装し、パサーとの疎結合化を実現する。

## 2. Functional Requirements
- **vte::Perform 互換インターフェースの実装**:
  - `print`, `execute`, `hook`, `put`, `unhook`, `osc_dispatch`, `csi_dispatch`, `esc_dispatch` の各メソッド（およびそのサブコマンド）を `TerminalBufferEntity` に実装する。
  - 現在 `AnsiParserDomainService` がバッファのフィールドを直接操作している箇所を、これらの標準メソッドに置き換える。
- **カプセル化の強化**:
  - `TerminalBufferEntity` のフィールド（`lines`, `cursor`, `current_attribute` 等）を `pub(crate)` から `private` に変更し、外部（特にパサー）からの直接アクセスを遮断する。
- **互換性維持**:
  - `edit` などの TUI アプリケーションの描画、および既存のユニットテストにおいてデグレードがないことを保証する。

## 3. Non-Functional Requirements
- **Zero-Copy Readiness**: 将来の `vte` 導入時にゼロコピーパースが可能となるよう、データの所有権や参照の扱いに配慮したシグネチャとする。
- **Domain Purity**: `windows` クレートへの依存を一切持ち込まず、純粋な Rust ロジックで完結させる。

## 4. Acceptance Criteria
- [ ] `TerminalBufferEntity` に `vte::Perform` トレイトで定義されている主要なメソッドが全て実装されていること。
- [ ] `AnsiParserDomainService` がバッファのフィールド（`lines`, `cursor` 等）を直接参照・変更していないこと。
- [ ] `cargo test` が全てパスし、既存のパース動作に影響がないこと。
- [ ] ウィンドウリサイズやスクロール、TUIアプリ（`edit`）の表示崩れが発生していないこと。

## 5. Out of Scope
- 本トラックでは `vte` クレート自体の導入（`Cargo.toml` への追加やパースエンジンの換装）は行わない。それは Issue #94 で実施する。
- 既存の `AnsiParserDomainService` のパースアルゴリズム自体の修正（ステートマシンの厳密化等）は行わない。
