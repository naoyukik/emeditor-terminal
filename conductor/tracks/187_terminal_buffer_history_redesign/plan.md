# 実装計画 (Implementation Plan): ターミナルのバッファ・履歴管理のアーキテクチャ抜本的再設計 (Issue 187)

## フェーズ 1: 依存面の棚卸しと分離方針の固定
- [x] Task: `TerminalBufferEntity` の責務を screen、history / viewport、interaction state の 3 系統に分類し、今回の対象を screen と history / viewport に固定する。
- [x] Task: `TerminalProtocolHandler`、`TerminalWorkflow`、`TerminalGuiDriver`、`window_message_resolver` が buffer に何を要求しているかを列挙し、読み取り面と更新面に分ける。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 1' (分離方針の確認)

## フェーズ 2: history / viewport 読み取り責務の独立
- [x] Task: visual row 解決、履歴長、viewport offset、スクロール操作を専用責務へ寄せ、`TerminalBufferEntity` から直接的な履歴計算ロジックを削減する。
- [x] Task: `get_line_at_visual_row`、`get_history_len`、`get_viewport_offset`、`scroll_to`、`scroll_lines`、`reset_viewport` の責務境界を整理する。
- [x] Task: scrollback と current screen を跨ぐ visual row 解決のテストを追加または更新する。

## フェーズ 3: screen 更新面の整理
- [ ] Task: カーソル移動、消去、スクロール領域、セル挿入削除などの screen 操作群を明確にまとめ、`TerminalProtocolHandler` が依存する更新面を縮小する。
- [x] Task: `TerminalProtocolHandler` が history / viewport を直接知らない構造へ整理する。
- [x] Task: ANSI パース、SGR、cursor visibility、mouse mode の回帰テストを維持する。

## フェーズ 4: Workflow / GUI の読み取り面の縮小
- [x] Task: `TerminalWorkflow` に描画とスクロール用の読み取り API を追加し、GUI が raw buffer を握る箇所を段階的に減らす。
- [x] Task: `TerminalGuiDriver` と `window_message_resolver` を、必要な読み取り専用面経由で動作するよう更新する。
- [x] Task: IME anchor、viewport、height / width、selection の利用経路を確認し、互換を維持する。

## フェーズ 5: 統合検証
- [x] Task: `cargo build` を実行し、コンパイル整合性を確認する。
- [x] Task: `cargo test` を実行し、parser・scrollback・描画関連の主要回帰を確認する。
- [ ] Task: 必要に応じて `cargo clippy` を実行し、設計整理に伴う警告を解消する。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 5' (実装結果と挙動確認)
