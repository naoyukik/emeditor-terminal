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
  - [x] `--dest` 引数によるインストール先ディレクトリの指定機能を追加する。
- [ ] Task: Conductor - User Manual Verification 'ビルド・インストールプロセスの確認' (Protocol in workflow.md)

## フェーズ 3: ソースコードの安全性向上
- [ ] タスク: `src/lib.rs` の修正
  - [ ] `QueryStatus` に `pbChecked` の null チェックを追加する。
  - [ ] `EVENT_CLOSE` に将来的な用途を説明するコメントを追加する。
- [ ] Task: Conductor - User Manual Verification 'コード修正後の動作確認' (Protocol in workflow.md)
