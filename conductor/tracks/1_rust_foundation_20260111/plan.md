# 計画書: Rust プラグイン基盤の構築

## フェーズ 1: プロジェクトの初期化
- [x] タスク: Rust プロジェクトの初期化
  - [x] 新しい Cargo ライブラリプロジェクトを作成する (`cargo new --lib`)。
  - [x] `Cargo.toml` を `crate-type = ["cdylib"]` を使用するように構成する。
  - [x] 必要な依存関係（`windows` 等）を追加する。
  - [x] Clippy と rustfmt によるチェック体制を整える。
- [ ] タスク: Conductor - ユーザー手動検証 'プロジェクトの初期化' (workflow.md のプロトコルに従う)

## フェーズ 2: EmEditor SDK 統合 (FFI)
- [ ] タスク: EmEditor FFI 型の定義
  - [ ] EmEditor SDK ヘッダーに対応する Rust の構造体/型を作成する（`LOAD_INFO`, `ET_COMMAND` など）。
- [ ] タスク: DLL エントリポイントの実装
  - [ ] `#[no_mangle]` を使用して `DllMain` エントリポイントを実装する。
  - [ ] 必要な EmEditor エクスポート関数（`OnCommand`, `QueryStatus` など）をスタブとして実装する。
- [ ] タスク: Conductor - ユーザー手動検証 'EmEditor SDK 統合 (FFI)' (workflow.md のプロトコルに従う)

## フェーズ 3: 「Hello World」動作確認
- [ ] タスク: Hello World ロジックの実装
  - [ ] 実行を確認するために、`OnCommand` または `DllMain` (ProcessAttach) に `MessageBoxW` の呼び出しを追加する。
- [ ] タスク: ビルドおよび配置スクリプトの作成
  - [ ] DLL をコンパイルするためのビルドスクリプトまたは手順を作成する。
  - [ ] EmEditor が認識できるように DLL を配置する場所をドキュメント化する。
- [ ] タスク: Conductor - ユーザー手動検証 'Hello World 動作確認' (workflow.md のプロトコルに従う)