# 実装計画 (Implementation Plan): リサイズ・起動時のフォント設定リセット問題の修正

## フェーズ 1: 問題の把握と詳細設計 (Discovery & Detailed Design)
- [x] Task: `autonomous-researcher` スキルを使用し、ターミナルプラグインのリサイズ (`on_size`) および起動時におけるフォント設定（サイズ、ウェイト等）のロード処理を調査し、`evidence_report.md` を作成する。
    - [x] ターミナル起動時の設定ロードフローを特定する。
    - [x] `on_size` イベントハンドラ内での設定ロード/更新処理を特定する。
    - [x] なぜ `font_size=0` や `weight=0` となるのか、根本原因（例：EmEditor本体のAPI呼び出し失敗、ハンドルの取得ミスなど）を解析する。
- [x] Task: 調査結果に基づき、本 `plan.md` の後続タスク（修正箇所、バリデーションロジックの追加等）を「どのファイルのどの行をどう変えるか」レベルまで具体的に書き換える。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 1' (調査結果と具体的プランの承認)

## フェーズ 2: フォント設定ロード処理の修正とバリデーションの実装 (Implementation & Validation)
- [x] Task: インフラ層およびリポジトリ層の修正を行い、不正なフォント設定（`0`）のロードを防止する。
    - [x] `src/infra/driver/emeditor_io_driver.rs` の `emeditor_query_u32` 内で `data` を `default` で初期化するように修正する。
    - [x] `src/infra/repository/emeditor_config_repository_impl.rs` の `load` 内で、取得した `font_size` および `font_weight` が `0` 以下の場合に `TerminalConfig::default()` の值を使用するバリデーションを追加する。
    - [x] `src/infra/repository/emeditor_config_repository_impl.rs` の `load` 内で、`font_size=0` をロードした際に警告ログを出力するようにする。
- [x] Task: 修正内容に対するユニットテストを追加・実行する。
    - [x] `src/infra/repository/emeditor_config_repository_impl.rs` のテストに、設定が `0` の場合にデフォルト値にフォールバックすることを確認するテストケースを追加する。（Win32依存のため実機検証で代替）
    - [x] `cargo test` を実行し、全テストのパスを確認する。
- [x] Task: 修正内容に対するコードのフォーマットと静的解析を実行する。
    - [x] `cargo clippy` を実行し、警告を解消する。
    - [x] `cargo fmt` を実行する。
- [x] Task: `AGENTS.md` の規約 (Conventional Commits) に従い、フェーズ 2 の成果物をコミットする。
- [x] Task: エディタを起動・リサイズし、フォント設定が維持されていること（ログおよび実際の描画）を確認する手動テストを実施する。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 2' (実装内容の動作確認と承認)

## フェーズ 3: 最終レビューとクリーンアップ (Final Review & Cleanup)
- [x] Task: `architecture-validator` スキル（または該当ツール）を実行し、アーキテクチャの依存関係に違反がないか確認する。
- [x] Task: Issueのクローズ条件（Acceptance Criteria）が全て満たされているか最終確認する。
- [x] Task: Conductor - ユーザー手動検証 'フェーズ 3' (全タスク完了の承認)
