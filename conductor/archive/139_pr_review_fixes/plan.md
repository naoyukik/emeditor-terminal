# 実装計画 (Implementation Plan)

## フェーズ 1: 問題の把握と詳細設計 (Discovery & Detailed Design)
- [x] Task: `autonomous-researcher` による詳細調査と `evidence_report.md` の作成
- [x] Task: 調査結果に基づいた `plan.md` の自己洗練（以降のタスクの具体化）
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 1' (調査結果と具体的プランの承認)

## フェーズ 2: フックの堅牢化とドキュメント修正 (Implementation & Fixes)
- [ ] Task: Python フックの例外ハンドリング実装
    - [ ] `.gemini/hooks/mempal_save_hook.py`: `mkdir` を `main()` 内へ移動し `try...except` で保護。失敗時は `log` 関数を無効化。
    - [ ] `.gemini/hooks/mempal_precompact_hook.py`: 同様に `mkdir` を `main()` 内へ移動し `try...except` で保護。
- [ ] Task: プロジェクトドキュメントの修正
    - [ ] `AGENTS.md`: 「ドットフォルダのコミット」セクションを絶対パスからリポジトリ相対パス指定（`.gemini/...`）に書き換え。
    - [ ] `conductor/tracks.md`: トラック項目のリンク行にインデント（スペース2つ）を追加し、リスト構造を修正。
- [ ] Task: アーカイブ済みレポートの修正
    - [ ] `conductor/archive/61_autonomous_research_20260409/evidence_report.md`: L58 の行番号を `L236` から `L266` へ更新。
- [ ] Task: 品質チェックとコミット
    - [ ] 実行：`cargo fmt` および `cargo clippy`
    - [ ] `AGENTS.md` の規約（個別 add）に従ってコミット。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 2: フックの堅牢化とドキュメント修正'
