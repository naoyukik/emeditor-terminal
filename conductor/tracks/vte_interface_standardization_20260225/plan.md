# Implementation Plan: vte_interface_standardization_20260225

## Phase 1: Preparation (Crate & Trait Implementation)
**目的:** `vte` クレートを導入し、`TerminalBufferEntity` に `vte::Perform` トレイトを実装する。

- [x] Task: `Cargo.toml` に `vte` クレートを追加。
- [x] Task: `TerminalBufferEntity` に `vte::Perform` トレイトを実装。
  - 主要メソッド（`print`, `execute`, `csi_dispatch` 等）を現在のバッファ操作ロジックに基づいて実装。
- [x] Task: `AnsiParserDomainService` 内で `buffer.lines`, `buffer.cursor` 等を直接参照・操作している箇所を、新設したトレイトメソッドの呼び出しに暫定的に置き換える。
- [x] Task: Conductor - User Manual Verification 'Phase 1: Preparation' (Protocol in workflow.md)

## Phase 2: Interface Implementation (Decoupling)
**目的:** `TerminalBufferEntity` のメソッドに、現在の `AnsiParserDomainService` が行っている具体的なロジックを統合し、フィールドを `private` 化する。

- [x] Task: `vte::Perform` の主要メソッド（`print`, `execute`, `csi_dispatch`, `osc_dispatch` 等）の実装を `TerminalBufferEntity` に集約。
  - `print`: 1文字の描画（既存の `put_char` ロジック）
  - `execute`: 制御文字（LF, CR, BS等）の実行（既存の `process_normal_char` の一部）
  - `csi_dispatch`: CSIシーケンス（H, J, K, m等）の実行
  - `osc_dispatch`: OSCシーケンス（タイトル設定等。現状は空実装）の実行
- [x] Task: `TerminalBufferEntity` のフィールドを `pub(crate)` から `private` に変更し、全ての外部アクセスを新設したインターフェースに限定する。
- [x] Task: `AnsiParserDomainService` のパース処理を、新設したメソッド呼び出しに完全に切り替える。
- [x] Task: `AnsiParserDomainService` 内の重複ロジック（`handle_csi` 内での直接的なバッファ操作等）を削除し、純粋な「パサー」として整理。
- [x] Task: Conductor - User Manual Verification 'Phase 2: Interface Implementation' (Protocol in workflow.md)

## Phase 3: Verification & Stabilization (Quality Assurance)
**目的:** `edit` 等の TUI アプリケーションを用いた実地検証を行い、デグレードがないことを確認する。

- [ ] Task: 既存のユニットテスト (`ansi_parser_domain_service::tests`) を実行し、全てパスすることを確認。
- [ ] Task: `cargo test` で全体の整合性を確認。
- [ ] Task: `edit` を起動し、描画の乱れやカーソル位置のズレ、日本語文字の整合性が維持されていることを確認。
- [ ] Task: `git log` 等のページャーアプリでの動作確認。
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Verification' (Protocol in workflow.md)

---

## Technical Notes
- `vte::Perform` トレイトの全メソッドを網羅するが、現状必要ないものについては「No-Op」として実装し、Issue #94 での拡張性を確保する。
- フィールドの `private` 化に伴い、Application層やGUI層でバッファの状態を取得するための `Getter` メソッド（`pub`）が必要になる可能性がある。これらは最小限に留め、読み取り専用とする。
