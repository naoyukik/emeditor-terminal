# 仕様書 (Specification): ターミナルのバッファ・履歴管理のアーキテクチャ抜本的再設計 (Issue 187)

## 概要 (Overview)
`TerminalBufferEntity` に集中している表示グリッド、スクロールバック履歴、ビューポート、カーソル、選択、マウス状態、書記素バッファリングの責務を整理し、段階的に分割可能な構造へ再設計する。今回は挙動互換を優先し、特に screen と history / viewport の分離、および GUI / Workflow の読み取り依存面の縮小を第一目的とする。

## 機能要件 (Functional Requirements)
1. **screen と history / viewport の責務分離**
   - 現在画面のセル更新やカーソル移動などの screen 操作と、スクロールバック履歴および visual row 解決などの history / viewport 操作を内部的に分離する。
2. **Protocol Handler 依存面の整理**
   - `TerminalProtocolHandler` が巨大な `TerminalBufferEntity` 全体に依存せず、screen 更新に必要な操作面に限定してアクセスする構造へ寄せる。
3. **GUI 読み取り面の明示化**
   - renderer と resolver が raw buffer 全体を握らず、描画とスクロールに必要な読み取り専用面を通じて情報取得する形へ整理する。
4. **段階移行互換の維持**
   - `TerminalWorkflow` および既存 GUI 呼び出し側は一度に全面切替しない。互換ファサードを残しつつ、今後の分割に耐える形へ移行する。

## 非機能要件 (Non-Functional Requirements)
- **保守性**: `terminal_buffer_entity.rs` の神クラス状態を解消し、責務ごとの分割を進めやすい構造にすること。
- **互換性**: 既存の描画、スクロール、選択、マウス追跡の挙動は原則として維持すること。
- **アーキテクチャ整合**: `conductor/code_styleguides/architecture_rules.md` の責務分離ルールに従うこと。

## 受け入れ条件 (Acceptance Criteria)
- [ ] `TerminalBufferEntity` の内部責務が screen と history / viewport で明確に分離されていること。
- [ ] `TerminalProtocolHandler` が history / viewport へ直接依存していないこと。
- [ ] GUI 側が描画に必要な読み取り面を経由して情報取得できること。
- [ ] 既存 parser / screen / scrollback 挙動を確認するテストが維持または追加されていること。
- [ ] `cargo build` と関連テストが成功すること。
