# PR #42 Review Fixes 実装計画

## フェーズ 1: バグ修正 (IME)
リファクタリングで消失したIME機能を復元する。

- [ ] Task: `custom_bar.rs` に IME メッセージハンドラ (`WM_IME_SETCONTEXT` 等) を復元する
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 1: バグ修正' (Protocol in workflow.md)

## フェーズ 2: リファクタリング (Design & Cleanup)
コード品質を向上させる。

- [ ] Task: `TerminalService` のフィールド可視性を修正 (pub -> private)
- [ ] Task: `AnsiParser` に `Default` トレイトを実装
- [ ] Task: コンパイル警告（未使用インポート、定数）の解消
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 2: リファクタリング' (Protocol in workflow.md)
