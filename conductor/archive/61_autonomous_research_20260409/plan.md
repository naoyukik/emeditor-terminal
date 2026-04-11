# 調査計画 (Investigation Plan)

## フェーズ 1: 問題の把握と外部調査 (Phase 1: External Research)
- [ ] Task: Issue 61の詳細確認と課題抽出
    - [ ] GitHub Issue 61の内容を読み込み、具体的な事象や要望を整理する。
    - [ ] 課題解決に必要な技術的キーワードや関連APIをリストアップする。
- [ ] Task: 公式ドキュメントの調査
    - [ ] Microsoft Learn API や関連する公式ドキュメント (`microsoft_docs_search`, `google_web_search`) を使用し、対象APIの仕様と制約を確認する。
    - [ ] EmEditor SDK の固有仕様 (`references/emeditor_sdk.md`) との関連性を調査する。
- [ ] Task: Conductor - User Manual Verification 'フェーズ 1: 問題の把握と外部調査' (Protocol in workflow.md)

## フェーズ 2: 内部規約との整合性検証とエビデンスレポート作成 (Phase 2: Internal Alignment & Evidence Reporting)
- [ ] Task: プロジェクト影響の評価
    - [ ] 現在のコードベースのどこに影響が及ぶかを特定する (`grep_search`, `codebase_investigator`)。
    - [ ] 修正方針が「Strict Rigid レイヤードアーキテクチャ」に反しないか検証する。
- [ ] Task: エビデンスレポートの作成
    - [ ] 調査結果をまとめ、一次ソースのURLを明記した `evidence_report.md` を作成する。
- [ ] Task: Conductor - User Manual Verification 'フェーズ 2: 内部規約との整合性検証とエビデンスレポート作成' (Protocol in workflow.md)

## フェーズ 3: 報告と完了 (Phase 3: Reporting & Completion)
- [ ] Task: Issue への報告
    - [ ] GitHub CLI または関連スキルを使用して、作成したレポートの内容を Issue 61 に反映する。
- [ ] Task: Conductor - User Manual Verification 'フェーズ 3: 報告と完了' (Protocol in workflow.md)