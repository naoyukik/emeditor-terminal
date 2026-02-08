# 計画: ウィンドウプロシージャの整理とディスパッチの効率化

## Phase 1: 基盤整理とデータ分離
`window.rs` (旧 `custom_bar.rs`) を軽量化するための準備として、データ構造と定数を適切なモジュールへ移動する。

- [ ] Task: `src/infra/editor.rs` に EmEditor SDK 固有の定数 (`EE_*`) および構造体 (`CUSTOM_BAR_INFO`) を移動・集約する。
- [ ] Task: `src/gui/terminal_data.rs` を新規作成し、`TerminalData` 構造体、`TERMINAL_DATA` 静的変数、および `get_terminal_data()` 関数を移動する。
- [ ] Task: `src/gui/mod.rs` を更新し、`terminal_data` モジュールを公開、`custom_bar` (後の `window`) の参照を維持する。
- [ ] Task: Conductor - User Manual Verification 'Phase 1: 基盤整理とデータ分離' (Protocol in workflow.md)

## Phase 2: ウィンドウ管理の再構築
メインのファイルリネームと、`wnd_proc` の論理的な抽出・整理を行う。

- [ ] Task: `src/gui/custom_bar.rs` を `src/gui/window.rs` へリネームし、`mod.rs` の宣言を更新する。
- [ ] Task: `window.rs` の `wnd_proc` 内の各メッセージ処理（`WM_PAINT`, `WM_SIZE`, `WM_VSCROLL` 等）を、`on_paint`, `on_size` などのプライベート関数へ抽出する。
- [ ] Task: `open_custom_bar` 関数内の ConPTY 出力読み取りスレッド起動ロジックを抽出し、別関数またはサービス層へ委譲して `window.rs` を軽量化する。
- [ ] Task: Conductor - User Manual Verification 'Phase 2: ウィンドウ管理の再構築' (Protocol in workflow.md)

## Phase 3: 仕上げと品質確認
ガイドラインへの準拠確認と、最終的なコード品質の向上を行う。

- [ ] Task: `src/gui/window.rs` の行数を確認し、300行以内であることを保証する。必要に応じてさらなる抽出を行う。
- [ ] Task: `cargo clippy` および `cargo fmt` を実行し、全ての警告とフォーマット違反を解消する。
- [ ] Task: Conductor - User Manual Verification 'Phase 3: 仕上げと品質確認' (Protocol in workflow.md)
