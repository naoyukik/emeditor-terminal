# Implementation Plan - Track: refactor scroll logic to src/gui/scroll.rs

## Phase 1: ScrollManager の実装とテスト
`src/gui/scroll.rs` モジュールを作成し、コアロジックを実装する。既存のコードにはまだ手を加えず、純粋なロジックとして完成させる。

- [x] Task: `src/gui/scroll.rs` ファイルの作成とモジュール定義
    - [x] `src/gui/mod.rs` に `mod scroll;` を追加する。
    - [x] `src/gui/scroll.rs` を作成する。
- [x] Task: `ScrollManager` と `ScrollAction` の定義 (TDD: Test First)
    - [x] `ScrollAction` Enum を定義する (`ScrollTo`, `ScrollBy`, `None` 等)。
    - [x] `ScrollManager` 構造体と `new` メソッドを定義する。
    - [x] `ScrollManager` の基本機能（状態保持）に対するテストケースを作成する。
- [x] Task: メッセージハンドリングロジックの実装 (TDD)
    - [x] `handle_vscroll` メソッドのテスト（SB_LINEUP, SB_PAGEDOWN 等）を作成する。
    - [x] `handle_vscroll` メソッドを実装する。
    - [x] `handle_mousewheel` メソッドのテストを作成する。
    - [x] `handle_mousewheel` メソッドを実装する。
- [x] Task: Conductor - ユーザー手動検証 'Phase 1' (Protocol in workflow.md)
    - [x] `cargo test src/gui/scroll.rs` が成功することを確認する。

## Phase 2: CustomBar への統合
実装した `ScrollManager` を `src/gui/custom_bar.rs` に組み込み、既存のロジックを置き換える。

- [x] Task: `CustomBar` 構造体への `ScrollManager` の追加
    - [x] `src/gui/custom_bar.rs` を開き、`CustomBar` 構造体に `scroll_manager: ScrollManager` フィールドを追加する。
    - [x] 初期化ロジックを更新する。
- [x] Task: `WM_VSCROLL` 処理の置き換え
    - [x] `wnd_proc` 内の `WM_VSCROLL` ケースを修正し、`scroll_manager.handle_vscroll` を呼び出すように変更する。
    - [x] 返された `ScrollAction` に基づいて `terminal_service` を操作するコードを記述する。
- [x] Task: `WM_MOUSEWHEEL` 処理の置き換え
    - [x] `wnd_proc` 内の `WM_MOUSEWHEEL` ケースを修正し、`scroll_manager.handle_mousewheel` を呼び出すように変更する。
    - [x] 返された `ScrollAction` に基づいて処理を行う。
- [x] Task: スクロール情報更新処理 (`update_scroll_info`) の置き換え
    - [x] 既存の `update_scroll_info` 関数と同様の処理を `ScrollManager` 経由で行うように変更する。
- [x] Task: リグレッションテストとクリーンアップ
    - [x] 古いスクロール関連コード（定数やヘルパー関数）を削除する。
    - [x] コンパイルエラーがないことを確認する。
- [ ] Task: Conductor - ユーザー手動検証 'Phase 2' (Protocol in workflow.md)
    - [ ] アプリケーションをビルドし、スクロールバー操作、マウスホイール操作が以前と同様に機能することを確認する。
