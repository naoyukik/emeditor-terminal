# Specification: PR #65 Review Fixes

## 1. 目的
PR #65 (RepositoryパターンとDIの導入) に対する GitHub Copilot のレビュー指摘事項を修正し、コードの安全性、リソース管理、およびドキュメントの整合性を確保する。

## 2. 修正事項

### 2.1 リソース管理の修正
- `cleanup_terminal` (src/gui/window/mod.rs) において、`TerminalService` を確実に drop するよう修正する。`std::mem::take` を用いて `service` を `None` に差し替える。
- `WM_DESTROY` ハンドラ (src/gui/window/handlers.rs) において、ConPTY 関連のリソース（プロセス、ハンドル、読み取りスレッド）が確実に解放されるよう、`cleanup_terminal` を呼び出すか同等の処理を実装する。

### 2.2 安全性とエラーハンドリング
- `SendHWND` (src/gui/terminal_data.rs) に対する `unsafe impl Send/Sync` の `SAFETY` 根拠コメントを復元・詳細化する。
- `TerminalService::resize` (src/application/service.rs) において、`output_repo.resize` の戻り値を確認し、エラーが発生した場合はログ出力（`log::error!` 等）を行う。

### 2.3 ドキュメントの整合性
- `conductor/archive/50_repository_pattern_di_20260208/spec.md` 内のファイル名記述を、実際の実装ファイル名（`*_impl.rs`）に合わせて更新する。
- `conductor/archive/50_repository_pattern_di_20260208/plan.md` において、未実装のテスト項目（Mockを用いたテスト）のチェックを外し、進捗状況を正確にする。

## 3. スコープ外
- `TerminalService` に対する新規ユニットテストの追加。これは設計変更後の安定性を確認するため、後続の別タスク（Future Ticket）として扱う。
