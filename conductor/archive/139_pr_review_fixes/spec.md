# 概要 (Overview)
PR 139 に寄せられた 5 件のコードレビュー指摘事項に対応し、品質と堅牢性を向上させる。

# 要件 (Functional Requirements)
1. `conductor/tracks.md` のリストインデント修正。
2. `AGENTS.md` のドットフォルダ規約を「絶対パス」から「リポジトリ相対パス」へ変更。
3. `.gemini/hooks/mempal_save_hook.py` の `mkdir` 処理をトップレベルから `main()` 内の try-except ブロックへ移動。
4. `.gemini/hooks/mempal_precompact_hook.py` も同様に `mkdir` の例外リスクを排除。
5. アーカイブ済み `evidence_report.md` の行番号不整合を最新ソースに合わせて修正。

# 非機能要件 (Non-Functional Requirements)
- MemPalace フックは HOME ディレクトリへのアクセス権がない環境でもサイレントにフェイルし、後続の圧縮・ブロック指示を妨げないこと。

# 成果物と完了基準 (Deliverables & Acceptance Criteria)
- [ ] 全ての指摘事項が修正され、コミットされていること。
- [ ] 修正後のフックが正常に動作するか、または例外で落ちないことが確認されていること。
- [ ] PR 139 に修正がプッシュされていること。