# 実装計画 (Implementation Plan)

## フェーズ 1: 問題の把握と詳細設計 (Discovery & Detailed Design)
- [ ] Task: `autonomous-researcher` による詳細調査と `evidence_report.md` の作成
    - 各フックにおける例外発生時の理想的なフォールバック挙動の調査。
- [ ] Task: 調査結果に基づいた `plan.md` の自己洗練（以降のタスクの具体化）
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 1' (調査結果と具体的プランの承認)

## フェーズ 2: フックの堅牢化とドキュメント修正 (Implementation & Fixes)
- [ ] Task: フックファイルの修正
    - `.gemini/hooks/mempal_save_hook.py` と `.gemini/hooks/mempal_precompact_hook.py` の `mkdir` を `main()` 内に移動し、`try...except` で囲む。
- [ ] Task: ドキュメントとマークダウンの修正
    - `AGENTS.md` のコミット規約の文言を相対パス指定に修正する。
    - `conductor/tracks.md` の `*Link:*` 行のインデントを修正する。
    - `conductor/archive/.../evidence_report.md` の行番号を最新の `mod.rs` と一致させる。
- [ ] Task: コンパイル警告の解消
    - 実行：`cargo fmt` および `cargo clippy`
- [ ] Task: コミットの作成
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 2: フックの堅牢化とドキュメント修正'