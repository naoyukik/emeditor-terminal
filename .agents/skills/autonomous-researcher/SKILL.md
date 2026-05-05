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

### Sequential Thinking 連携規約

本スキルでは、`sequential-thinking` を「常時実行の儀式」ではなく、調査品質を上げるための必須チェックポイントとして扱う。

- **必須タイミング**:
  - Phase 1 完了時: 要求の再記述、制約、成功条件、未確定事項の整理
  - Phase 3 開始前: 何が未確定で、何を質問しないと設計判断できないかの棚卸し
  - Phase 4 開始前: 比較案の軸、主リスク、推奨理由の妥当性確認
- **Phase 2 の扱い**:
  - 調査開始前に 1 回だけ使い、探索視点の不足がないか確認する
  - 調査中は毎回使わない。重要な発見が揃った時点で再度使い、次に掘るべき論点を整理する
- **主な用途**:
  - 調査観点の不足確認
  - 仮説と事実の分離
  - 次に読むべきファイル、確認すべき API、質問すべき論点の絞り込み
  - 案比較時の評価軸と見落としの点検
- **禁止事項**:
  - 単純な検索結果確認のたびに `sequential-thinking` を呼び出してはならない
  - 思考ログを増やすこと自体を目的化してはならない
- **記録方針**:
  - `evidence_report.md` には思考ログ全文ではなく、各チェックポイントで確定した要点のみを要約して残すこと
  - 要約には「何を確認したか」「何が未確定か」「次に何を調べるか」を含めること

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

- **IDE 統合機能の優先活用 (JetBrains MCP)**:
  - 調査の第一歩として JetBrains MCP ツールを積極的に活用し、コードベースを横断的に把握せよ。
  - `mcp_jetbrains_search_symbol`: クラス、メソッド、フィールドなどのシンボルを意味的に検索する。
  - `mcp_jetbrains_search_text` / `mcp_jetbrains_search_regex`: 高速な文字列検索により、スニペットレベルで用法を確認する。
  - `mcp_jetbrains_get_symbol_info`: シンボルの宣言場所、型、ドキュメントを詳細に取得し、意味論的な理解を深める。
- **専門サブエージェントへの委譲**:
  - 曖昧な要求の分析、広範なアーキテクチャマッピング、または複雑な依存関係の特定が必要な場合は、`codebase_investigator` subagent を呼び出せ。
  - `invoke_agent` を用い、構造化されたレポート（主要なファイルパス、シンボル、実用的な建築的洞察）を取得することで、自身のコンテキストを節約しつつ深い洞察を得ること。
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
- Sequential Thinking Checkpoints Summary
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
