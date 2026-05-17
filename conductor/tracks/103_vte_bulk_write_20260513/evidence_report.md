# 調査レポート (Evidence Report) - vteの一括書き込み最適化 (Issue 103)

## Discovery Summary
- **課題**: 現在の `TerminalProtocolHandler` は `vte` パーサーから文字を受け取るたびに即座に `TerminalBufferEntity::print_cell(char)` を呼び出している。このため、1文字ごとに書記素クラスタ判定や行折り返し判定が走り、大量出力時のパフォーマンス低下を招いている。
- **解決策**:
    1. `TerminalProtocolHandler` に `accumulator: String` を導入し、属性や状態の変化が発生するまで文字を蓄積する。
    2. `TerminalBufferEntity` に `print_string(&str)` を実装し、蓄積された文字列を一括で処理する。
- **制約**:
    - 属性変更（SGR）や制御文字受信、エスケープシーケンス開始のタイミングで必ずバッファをフラッシュしなければならない。
    - 書記素クラスタの判定ロジック（`pending_cluster`）との整合性を保つ必要がある。

## Codebase Findings
### 1. `TerminalBufferEntity` (src/domain/model/terminal_buffer_entity.rs)
- `print_cell(char)` のロジックをベースに、文字列スライスを受け取る `print_string(&str)` を作成可能。
- 既存の `pending_cluster` 処理は、文字列全体の処理が終わった後に未完了分を保持する形で維持する。

### 2. `TerminalProtocolHandler` (src/domain/service/terminal_protocol_handler.rs)
- `accumulator` をフラッシュするプライベートメソッドを追加し、`vte::Perform` トレイトの全メソッドの先頭でこれを呼び出す。

### 3. `Workflow` (conductor/workflow.md)
- 「トラックの初期化」セクションにブランチ作成の手順を追加する。

## Expected Behavior
- 大量のテキスト出力において、CPU負荷が低減し、描画が滑らかになる。
- 既存の結合文字や絵文字の描画が損なわれない。

## Specific Fix Locations
- **`src/domain/model/terminal_buffer_entity.rs`**: `print_string` メソッドの追加。
- **`src/domain/service/terminal_protocol_handler.rs`**: `accumulator` フィールドの追加、`Perform` 実装の各メソッドへの `flush` 呼び出し追加。
- **`conductor/workflow.md`**: 「トラックの作成」手順への Git 操作追記。

## Architecture Options
| 案 | メリット | デメリット |
| :--- | :--- | :--- |
| **A. Handlerでバッファリング (推奨)** | 既存の vte ロジックをほぼ変えずに性能向上。 | フラッシュ漏れのリスク。 |
| **B. Parser自体をスライス対応にする** | 理論上最高速。 | vte クレート自体の修正が必要（困難）。 |

**推奨案: A** (実装の容易さと安全性のバランスが最適)。
