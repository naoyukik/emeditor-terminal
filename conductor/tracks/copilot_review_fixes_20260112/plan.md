# 計画書: GitHub Copilot 指摘への対応

## フェーズ 1: ドキュメントとガイドラインの修正
- [x] タスク: タイポの修正
  - [x] `conductor/product-guidelines.md` の「入出力の」を「入出力の」に修正する。
- [x] タスク: インストールガイドの修正
  - [x] `docs/INSTALL.md` の手動インストールパスを `$env:LOCALAPPDATA\Programs\EmEditor\PlugIns` に更新する。
- [x] Task: Conductor - User Manual Verification 'ドキュメント修正の確認' (Protocol in workflow.md)

## フェーズ 2: ビルド・インストールプロセスの改善
- [x] タスク: `build.rs` の堅牢化
  - [x] `CARGO_MANIFEST_DIR` を使用して `exports.def` を絶対パスで指定するように修正する。
- [x] タスク: `install.ps1` の機能強化
  - [x] `cargo build` の成功判定 (`$LASTEXITCODE`) を追加する。
  - [x] `-Dest` 引数によるインストール先ディレクトリの指定機能を追加する。
- [x] Task: Conductor - User Manual Verification 'ビルド・インストールプロセスの確認' (Protocol in workflow.md)

## フェーズ 3: ソースコードの安全性向上
- [x] タスク: `src/lib.rs` の修正
  - [x] `QueryStatus` に `pbChecked` の null チェックを追加する。
  - [x] `EVENT_CLOSE` に将来的な用途を説明するコメントを追加する。
## フェーズ 4: 第2回レビュー指摘への対応
- [x] タスク: コード品質の向上 (Rust)
  - [x] `src/lib.rs`: `EVENT_CLOSE` に `#[allow(dead_code)]` を追加する。
  - [x] `build.rs`: `CARGO_MANIFEST_DIR` 取得時の `unwrap()` を `expect()` に変更し、詳細なエラーメッセージを追加する。
- [x] タスク: ドキュメントと引数の整合性修正
  - [x] `docs/INSTALL.md`: PowerShell の引数指定を `--release` から `-Release` に修正する。
  - [x] `conductor/tracks/copilot_review_fixes_20260112/plan.md`: 計画書内の記述 `--dest` を `-Dest` に修正する。
- [x] タスク: テストの追加
  - [x] `src/lib.rs`: 定数やエクスポート関数の基本的なユニットテストを追加する。
- [ ] Task: Conductor - User Manual Verification '第2回修正の確認' (Protocol in workflow.md)

## フェーズ 5: 第3回レビュー指摘への対応
- [x] タスク: 計画書の記述修正
  - [x] `conductor/tracks/copilot_review_fixes_20260112/plan.md`: フェーズ1のタイポ修正タスクの記述を、「誤字を修正する」と明確な表現に変更する（修正前後の文字列が同一で混乱を招くため）。
- [x] タスク: 命名規則のドキュメント化
  - [x] `docs/INSTALL.md`: Cargoパッケージ名（ハイフン区切り）と生成されるDLL名（アンダースコア区切り）の違いについて注記を追加する。
- [x] Task: Conductor - User Manual Verification '第3回修正の確認' (Protocol in workflow.md)

## フェーズ 6: 最終的な修正
- [x] タスク: 仕様書の整合性修正
  - [x] `conductor/tracks/copilot_review_fixes_20260112/spec.md`: `--dest` の記述を `-Dest` に修正する。
- [x] タスク: スクリプトの可読性向上
  - [x] `install.ps1`: エラーメッセージ内の変数展開を明確にするため、`$LASTEXITCODE` を `${LASTEXITCODE}` に変更する。