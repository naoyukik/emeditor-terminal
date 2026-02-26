# Implementation Plan: vte クレート導入とパースエンジン換装

## Phase 1: 依存関係の追加とインターフェース準備
- [x] Task: `Cargo.toml` に `vte` クレート (v0.14) を追加する。
- [x] Task: `AnsiParserDomainService` またはその代替クラスにて、`vte::Perform` トレイトを実装する基盤を作成する。
- [x] Task: 既存のパース命令（文字出力、カーソル移動等）を `vte::Perform` の各メソッド（`print`, `execute`, `csi_dispatch` 等）へマッピングするためのマッパーを定義する。
- [x] Task: コードをコミットする。
- [x] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)

## Phase 2: パースエンジンの完全換装
- [x] Task: `src/domain/service/ansi_parser_domain_service.rs` 等の既存パースロジックを `vte::Parser` に置き換える。
- [x] Task: 複数バイトに渡る UTF-8 シーケンスを `vte::Parser::advance` に正しく供給するためのバイトバッファ処理を調整する。
- [x] Task: `csi_dispatch` 内で、既存でサポートしていた全シーケンス（SGR, DECSTBM, ED, EL, CUP 等）の呼び出しを実装する。
- [x] Task: コードをコミットする。
- [x] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)

## Phase 3: テストによる整合性検証
- [x] Task: 既存 `ansi_parser` 関連の単体テストを、`vte` ベースの実装に対してパスさせる。
- [x] Task: 複雑な TUI 出力のキャプチャデータを用いた回帰テストを実施し、表示の崩れがないことを確認する。
- [x] Task: 未使用となった手作りパース用の定数、Enum、メソッド等をクリーンアップ（リファクタリング）する。
- [ ] Task: コードをコミットする。
- [ ] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)

## Phase 4: 実機動作確認と最終調整
- [ ] Task: EmEditor 上でプラグインを動作させ、`pwsh.exe` や各種 TUI コマンド（`less`, `vim` 等）の表示整合性を確認する。
- [ ] Task: `Clippy` / `cargo fmt` を実行し、コード品質を確保する。
- [ ] Task: コードをコミットする。
- [ ] Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)
