---
name: autonomous-researcher
description: Execute a pre-implementation investigation workflow modeled on feature-dev Phase 1-4 (Discovery, Codebase Exploration, Clarifying Questions, Architecture Design). Use this when requirements are ambiguous, architecture decisions are needed, or external validation (Win32 API, Rust crates, EmEditor SDK) is required before coding in Conductor tracks.
---

# Autonomous Researcher

## 概要

このスキルは、実装前の調査フェーズを厳密に進行するための手順書である。  
目的は「曖昧さを潰す」「既存コードの文脈を把握する」「設計案を比較して選ぶ」を完了し、根拠ある実装着手条件を作ることである。  
本スキルの適用範囲は Phase 4 までであり、実装開始は対象外とする。

## 調査専用トラック (Investigation-Only Tracks)

Conductor において「調査」を目的としたトラック（実装を含まない）を扱う場合、以下のルールを厳守すること。

- **`spec.md` の責務**: 調査自体のスコープ（何を調べるか）、調査手法（どのスキルを使うか）、および成果物（レポート作成、Issue 報告）の定義に限定する。**将来の実装に関する詳細な挙動（期待される挙動）をここに記述してはならない。**
- **`evidence_report.md` の責務**: 調査結果、根本原因の特定、および**将来の実装で達成すべき「期待される挙動 (Expected Behavior)」**をここに集約する。これにより、調査と実装の関心を分離する。
- **継続性**: 調査トラックの完了後、その `evidence_report.md` をインプットとして、別途「実装トラック」を立ち上げる運用とする。

## 強制ワークフロー (Phase 1-4)

実装提案・コード編集・パッチ作成の前に、必ず Phase 1 から順に完了させること。

- Phase 3 の質問が未解決のまま Phase 4 に進んではならない。  
  例外は、ユーザーが「判断を委任する」と明示した場合のみである。
- EmEditor SDK を扱う場合は `references/emeditor_sdk.md` を先に確認すること。
- 調査ログは `assets/evidence_report_template.md` に従って記録すること。

### Phase 1: Discovery (要求理解)

**目的**: 何を達成すべきかを明確化する。  
**実施内容**:

- 要求を再記述し、解くべき問題を明文化する。
- スコープ・非スコープ・制約（期限/互換性/性能/セキュリティ）を列挙する。
- 成功条件（受け入れ条件）を定義する。
- 不明瞭な要求はこの段階で質問する。

**出力**:

- Discovery Summary（課題、制約、成功条件）
- 未確定事項リスト（Phase 3 で回収する前提）

### Phase 2: Codebase Exploration (既存実装調査)

**目的**: 既存コードと設計慣習を把握する。  
**実施内容**:

- 最低 3 視点で並行調査する。
  - 類似機能の実装経路（エントリーポイントから終端まで）
  - レイヤー構造と依存方向（外側 -> 内側）
  - 関連機能との結合点（設定、I/O、エラー処理、UI）
- 重要ファイルは必ず実読し、`file:line` で根拠を残す。
- 必要に応じて `conductor/code_styleguides/` と照合する。

**出力**:

- Codebase Findings（既存パターン、再利用候補、変更影響範囲）
- Key Files To Read（`file:line` 付き）

### Phase 3: Clarifying Questions (曖昧さ解消)

**目的**: 設計判断に必要な前提を確定する。  
**実施内容**:

- Phase 1/2 の結果から、判断不能点を体系化する。
  - エッジケース
  - エラー処理方針
  - 互換性（既存仕様との共存/置換）
  - 性能・セキュリティ要件
  - 検証観点
- 質問は番号付きで提示し、回答待ちを明示する。

**ゲート条件**:

- ユーザー回答（または明示的委任）を得るまで、Phase 4 へ進まない。

### Phase 4: Architecture Design (設計案比較)

**目的**: 複数の実装方針を比較し、着手方針を確定する。  
**実施内容**:

- 2〜3 案を作成して比較する（最低でも以下 3 類型を含む）。
  - Minimal Changes: 既存再利用最大・差分最小
  - Clean Architecture: 分離性・保守性優先
  - Pragmatic Balance: 工数と品質の均衡
- 各案について次を明示する。
  - 変更対象ファイル/レイヤー
  - 依存方向・責務分離の妥当性
  - 主リスクと回避策
  - 検証方針（テスト/手動確認ポイント）
- 推奨案を 1 つ示し、採用判断をユーザーに確認する。

**出力**:

- Architecture Options（比較表）
- Recommendation（採用理由付き）

## 根拠ベース検証 (Evidence Discipline)

- **Web 検索と Web Fetch の活用**: 外部情報の取得には `google_web_search` や `microsoft_docs_search` などの検索ツールを積極的に使用し、必要に応じて `web_fetch` や `microsoft_docs_fetch` で詳細な仕様を確認せよ。これにより、ローカルの知識だけでなく最新かつ公式な情報を反映させること。
- 外部仕様は必ず 1 次ソースを優先する（Learn Microsoft, docs.rs, RFC, 公式リポジトリ, Web等の公式リファレンス）。
- 調査時は `references/research_guidelines.md` のクエリとチェック項目を使う。
- 参照 URL と調査日、重要事実、制約、破壊的変更の有無を記録する。
- 外部情報とローカル規約（`conductor/code_styleguides/`）の整合を検証する。

## 最終成果物 (実装前レポート)

実装提案に進む前に、`assets/evidence_report_template.md` を使って以下を必ず提出する。

- Discovery Summary
- Codebase Findings
- Clarifying Questions と回答状況
- Architecture Options と推奨案
- Evidence（URL、要点、規約整合、未確定リスク）

Conductor の spec/plan がある場合は、その近傍に `evidence_report.md` を保存すること。

## ガイドライン

- **積極的な情報収集**: 不明点や外部 API の挙動については、自身の知識に頼らず Web 検索ツールを活用して裏付けを取ること。
- **"I don't know" を恐れない**: 調査で確証が得られない場合、推測でコードを書くのではなく、不足している情報をユーザーに報告し、さらなる調査方針を相談する。
- **1次ソースを優先する**: ブログ記事や Stack Overflow よりも、公式ドキュメント（Learn Microsoft, docs.rs, RFC）を優先的に参照する。
- **Regressions への警戒**: ライブラリのバージョンアップによる破壊的変更や、OSの挙動変更がないか、GitHub Issue やリリースノートを検索する。

## リソース

- **`references/research_guidelines.md`**: 効果的な検索クエリとチェックリスト。
- **`references/emeditor_sdk.md`**: EmEditor SDK 固有の注意点と既知仕様。
- **`assets/evidence_report_template.md`**: 実装前調査レポートのテンプレート。
