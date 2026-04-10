# リサーチガイドライン (Phase 1-4 対応)

`autonomous-researcher` の実装前調査フェーズで使用する実務チェックリストである。

## Phase 1: Discovery

### 収集項目
- 何を解決する要求か（現状の痛み）
- 非機能制約（性能、互換性、安全性、期限）
- 成功条件と非スコープ

### ⚠️ 重要：調査専用トラックの規律
- 調査トラックにおいては、**実装の詳細や期待される挙動を `spec.md` に書かない**。
- `spec.md` のゴールは「`evidence_report.md` の作成」と「Issue への報告」に留める。
- 実装の詳細は、調査の結果得られた「エビデンス」の一部として `evidence_report.md` に記載する。

### 失敗パターン
- 目的より手段が先行している
- 調査段階で実装仕様（spec）を定義しようとしてしまう
- 非スコープが未定義で作業範囲が拡散する

## Phase 2: Codebase Exploration

### 必須観点
- 類似機能の実装経路（入口 -> 主要処理 -> 出口）
- レイヤー責務と依存方向（外側 -> 内側）
- 変更時の波及（設定、I/O、UI、ログ、エラー）

### 最低出力
- `file:line` 付き根拠
- 再利用可能コンポーネント一覧
- 変更影響範囲

## Phase 3: Clarifying Questions

### 質問カテゴリ
- エッジケース
- エラー処理（ユーザー表示/ログ/リトライ）
- 既存仕様との共存・置換
- 性能予算・セキュリティ境界
- 検証方針（自動/手動）

### 進行ルール
- 質問は番号付きで提示する
- 回答または明示委任が得られるまで次フェーズへ進まない

## Phase 4: Architecture Design

### 必須比較軸
- 実装コスト
- 保守性・分離性
- 規約適合性（アーキテクチャ/命名/unsafe 方針）
- リスクと検証可能性

### 推奨フォーマット
- Option A: Minimal Changes
- Option B: Clean Architecture
- Option C: Pragmatic Balance

各 Option で「変更対象」「依存方向」「主要リスク」「検証方法」を記述する。

## 外部調査クエリ例

### Win32 API
- **Query**: `site:learn.microsoft.com [API名] [error code or constant]`
- **Check**:
  - 対応 OS バージョン要件（例: ConPTY は Windows 10 1809+）
  - 戻り値の型（`HRESULT` / `BOOL` / `HANDLE`）
  - `GetLastError()` または `HRESULT` 解釈の要否

### Rust クレート
- **Query**: `[crate name] docs.rs [item name]`
- **Check**:
  - 使用予定バージョンの API 仕様
  - 破壊的変更・既知不具合（release note / issue）
  - `unsafe` 契約とスレッド安全性

### TUI / ターミナル仕様
- **Query**: `VT escape sequence [CSI/DSR] behavior [terminal name]`
- **Check**:
  - ターミナル実装差（Windows Terminal / VSCode / iTerm2 など）
  - 制御文字の互換性とフォールバック

## 記録ルール

- 参照 URL、調査日、重要事実、設計への反映点を残す。
- 情報が確定できない場合は「未確定」と明記し、次の調査手段を提示する。
