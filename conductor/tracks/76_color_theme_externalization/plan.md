# 実装計画: カラーテーマ設定の外部化と永続化 (Issue #76)

## Phase 1: ドメイン層・インフラ層の拡張 (ThemeTypeの永続化対応)
- [x] Task: ドメインモデルの修正
- [x] Task: インフラ層（永続化ロジック）の実装 (TDD)
- [x] Task: Conductor - Clippy & fmt Check (Clippyは自動フォーマットを使用すること)
- [x] Task: Conductor - 'Phase 1: ドメイン層・インフラ層の拡張' の成果をコミット
- [x] Task: Conductor - User Manual Verification 'Phase 1: ドメイン層・インフラ層の拡張' (Protocol in workflow.md)

## Phase 2: GUI設定ダイアログへの統合
- [x] Task: リソース定義の更新 (`emeditor-terminal.rc`)
- [x] Task: GUI Driver (ダイアログ制御) の実装
- [x] Task: GUI層の単体/ロジックテスト
- [x] Task: Conductor - Clippy & fmt Check (Clippyは自動フォーマットを使用すること)
- [x] Task: Conductor - 'Phase 2: GUI設定ダイアログへの統合' の成果をコミット
- [x] Task: Conductor - User Manual Verification 'Phase 2: GUI設定ダイアログへの統合' (Protocol in workflow.md)

## Phase 3: System Default (Auto) テーマの解決と反映
- [x] Task: テーマ解決ロジックの実装
- [x] Task: ターミナル初期化時の適用
- [x] Task: Conductor - Clippy & fmt Check (Clippyは自動フォーマットを使用すること)
- [x] Task: Conductor - 'Phase 3: System Default (Auto) テーマの解決と反映' の成果をコミット
- [x] Task: Conductor - User Manual Verification 'Phase 3: System Default (Auto) テーマの解決と反映' (Protocol in workflow.md)

## Phase 4: 総合テストと最終検証
- [x] Task: ビルドと手動機能テスト
- [x] Task: Conductor - Clippy & fmt Check (Clippyは自動フォーマットを使用すること)
- [x] Task: Conductor - 'Phase 4: 総合テストと最終検証' の成果をコミット
- [x] Task: Conductor - User Manual Verification 'Phase 4: 総合テストと最終検証' (Protocol in workflow.md)

## Phase: Review Fixes
- [x] Task: build.rs の堅牢化と最適化
- [x] Task: ダークモード判定の命名と挙動の改善
- [x] Task: ThemeType マッピングの整合性修正 (Bug Fix)
- [x] Task: ドキュメント表現の調整
- [x] Task: スキル定義の修正
- [x] Task: Conductor - Clippy & fmt Check。Clippyはfixを使用すること。&&は使えないので個別に実行すること。
- [x] Task: Conductor - `.\install.ps1 -Path "$env:PLUGINS_DIR"` を実行
- [x] Task: Conductor - User Manual Verification 'Phase: Review Fixes' (Protocol in workflow.md)
- [x] Task: Conductor - 'Phase: Review Fixes' の成果をコミット
