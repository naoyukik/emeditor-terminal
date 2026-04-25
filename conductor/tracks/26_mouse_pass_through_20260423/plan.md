# Implementation Plan - Mouse Event Pass-through (SGR 1006)

## Phase 1: 調査と詳細設計 (Discovery & Detailed Design)
- [x] Task: `autonomous-researcher` による詳細調査と `evidence_report.md` の作成
- [x] Task: 調査結果に基づいた `plan.md` の以降のタスクの具体化。
- [x] Task: Conductor - User Manual Verification 'Phase 1' (調査結果と具体的プランの承認)

## Phase 2: Domain / Infrastructure 層の実装 (TDD)
- [x] Task: `src/domain/model/terminal_types_entity.rs` にマウス関連の型を追加
    - `MouseTrackingMode` (None, Default, ButtonEvent, AnyEvent) の定義。
- [x] Task: `src/domain/model/terminal_buffer_entity.rs` への状態管理フィールドの追加
    - `mouse_tracking_mode`, `use_sgr_mouse_encoding` フィールドの追加と getter/setter。
- [x] Task: `src/domain/service/terminal_protocol_handler.rs` へのモード設定パースロジックの追加
    - `?1000`, `?1002`, `?1003`, `?1006` の `h` (SM) / `l` (RM) 処理。
- [x] Task: `src/domain/service/vt_sequence_translator_domain_service.rs` へのマウスイベント変換ロジックの実装
    - `translate_mouse_event` メソッドの追加 (SGR 1006 形式)。
- [x] Task: `cargo clippy` および `cargo fmt` の実行
- [x] Task: Conductor - User Manual Verification 'Phase 2' (Domain層のユニットテスト合格確認)

## Phase 3: Application / GUI 層の統合と検証
- [x] Task: `src/application/terminal_workflow.rs` に `handle_mouse_event` メソッドを追加
    - バッファのモードを確認し、必要なら変換して `send_input` する。
- [x] Task: `src/gui/resolver/window_message_resolver.rs` に各マウスメッセージハンドラを追加
    - `on_lbuttondown` (更新), `on_lbuttonup`, `on_rbuttondown/up`, `on_mbuttondown/up`, `on_mousemove`, `on_mousewheel` (更新)。
    - Shift キー押下によるバイパス判定の実装。
    - ピクセル座標からセル座標への変換。
- [x] Task: 実機（EmEditor + Vim/tmux 等）によるマウス操作の手動検証
- [x] Task: `cargo clippy` および `cargo fmt` の実行
- [x] Task: Conductor - User Manual Verification 'Phase 3' (実機動作確認の承認)

## Phase: Review Fixes
- [x] Task: Apply review suggestions 5601d63
