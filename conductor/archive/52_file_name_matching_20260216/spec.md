# Track Specification - 52_file_name_matching

## 概要 (Overview)
`architecture_rules.md` に定められた「サフィックスルール（The Suffix Rule）」に基づき、プロジェクト全域のファイル命名と配置を厳格に再編する。構造体名とファイル名を一致させるだけでなく、役割に応じた接尾辞（`_resolver`, `_workflow`, `_entity`, `_driver` 等）を義務付け、物理的な隔離によるアーキテクチャ統制を実現する。

## 機能要件 (Functional Requirements)
- **サフィックスルールの適用**:
  - Presentation層: `_resolver.rs`, `_gui_driver.rs`, `_request.rs`, `_response.rs`
  - Application層: `_workflow.rs`, `_input.rs`, `_result.rs`
  - Domain層: `_entity.rs`, `_value.rs`, `_domain_service.rs`, `_repository.rs`
  - Infrastructure層: `_repository_impl.rs`, `_io_driver.rs`
- **構造体とファイル名の完全一致**:
  - 独立したファイルに存在する構造体について、ファイル名を構造体名の `snake_case` + 接尾辞にリネームする。
  - ※ `mod.rs` からのロジック抽出、および同一ファイル内の複数構造体の分割は、作業規模を考慮し Issue #68 に切り出された。
- **Windows API の封印**:
  - `windows` クレートの型が `_gui_driver.rs` および `_io_driver.rs` 以外に漏洩していないか厳格に監査し、必要に応じて内部型への変換ロジックを `_resolver.rs` 等に実装する。
- **ユビキタス言語の同期**:
  - ファイル名および内部の構造体・変数名を `architecture_rules.md` の「ユビキタス言語辞典」に完全に適合させる。

## 非機能要件 (Non-Functional Requirements)
- **300行制限の遵守**: ファイル分割により、各ファイルのサイズを300行以内に抑える。
- **依存方向の維持**: 外側（Presentation/Infra）から内側（Domain/Application）への単方向依存を物理的に強制する。

## 受入基準 (Acceptance Criteria)
- すべてのソースファイルが規定の接尾辞を持ち、その役割がファイル名から一意に特定できる。
- `mod.rs` にロジックが含まれていない。
- `cargo test` および `cargo clippy` がすべてパスする。
- Domain/Application層に `windows` クレートへの依存が含まれていない。
