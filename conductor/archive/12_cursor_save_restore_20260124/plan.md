# Implementation Plan - Track 12: カーソル位置の保存/復元 (DECSC/DECRC) の実装

## Phase 1: 基礎構造とユニットテストの準備
`Terminal` 構造体に保存用フィールドを追加し、期待される挙動を検証するテストコードを記述する。

- [ ] Task: `src/domain/terminal.rs` の `Terminal` 構造体に `saved_cursor` フィールドを追加
    - [ ] `Option<(usize, usize)>` 型のフィールドを定義し、`new` 関数で `None` 初期化する
- [ ] Task: 保存・復元ロジックのユニットテスト作成
    - [ ] `DECSC` で位置が保存されることを確認するテスト
    - [ ] `DECRC` で位置が復元されることを確認するテスト
    - [ ] 未保存状態での `DECRC` が無視されることを確認するテスト
    - [ ] 画面サイズ縮小時に行・列がクリップされることを確認するテスト
- [ ] Task: Conductor - ユーザー手動検証 'Phase 1' (Protocol in workflow.md)

## Phase 2: コアロジックの実装
テストをパスするように、`Terminal` 構造体に保存・復元メソッドを実装する。

- [ ] Task: `Terminal` への保存・復元メソッドの実装
    - [ ] `save_cursor()` メソッドの実装
    - [ ] `restore_cursor()` メソッドの実装（クリッピングロジックを含む）
- [ ] Task: ユニットテストの実行と修正
    - [ ] `cargo test` を実行し、Phase 1 で作成したテストがすべてパスすることを確認する
- [ ] Task: Conductor - ユーザー手動検証 'Phase 2' (Protocol in workflow.md)

## Phase 3: VTシーケンスパーサーの統合
外部からの文字列入力に対して `ESC 7` と `ESC 8` を認識し、実装したメソッドを呼び出すようにする。

- [ ] Task: VTシーケンスパーサーの更新
    - [ ] `src/domain/terminal.rs` (または関連するパーサー) で `ESC 7` (DECSC) をハンドルする
    - [ ] `src/domain/terminal.rs` (または関連するパーサー) で `ESC 8` (DECRC) をハンドルする
- [ ] Task: 統合テストの追加
    - [ ] 文字列入力経由で保存・復元が機能することを検証するテストを追加
- [ ] Task: Conductor - ユーザー手動検証 'Phase 3' (Protocol in workflow.md)

## Phase 4: 実機動作確認と仕上げ
実際のプラグイン環境で動作を確認し、コードの品質を整える。

- [ ] Task: EmEditor 上での動作確認
    - [ ] 実際に `DECSC`/`DECRC` を発行する TUI アプリケーション（またはテストスクリプト）を用いて動作を確認
- [ ] Task: リンターとフォーマッタの実行
    - [ ] `cargo clippy` および `cargo fmt` を実行し、警告がないことを確認
- [ ] Task: Conductor - ユーザー手動検証 'Phase 4' (Protocol in workflow.md)
