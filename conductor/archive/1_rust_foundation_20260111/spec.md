# 仕様書: Rust プラグイン基盤の構築

## ゴール
- EmEditor プラグイン開発用に構成された Rust プロジェクトを初期化する。
- EmEditor SDK と通信するために必要な FFI (Foreign Function Interface) 層を実装する。
- 有効な EmEditor プラグインとして動作する `cdylib` (DLL) を正常にビルドする。
- コンパイルされた DLL が EmEditor にロードされ、シンプルなコマンド（例：「Hello World」）を実行できることを確認する。

## ユーザーストーリー
- 開発者として、`cargo build` を使用してプロジェクトをビルドし、依存関係とコンパイルを容易に管理したい。
- 開発者として、プラグインをネイティブ DLL として動作させ、EmEditor が外部ランタイムなしでロードできるようにしたい。
- ユーザーとして、プラグインをロードしたときに動作確認用のアクション（メッセージボックスなど）が表示され、基盤が強固であることを確認したい。

## 技術要件
### 開発環境
- **言語**: Rust (Edition 2021 以降)
- **ターゲット**: `x86_64-pc-windows-msvc` (64bit 版 EmEditor を想定)。
- **ビルドツール**: Cargo

### EmEditor 統合
- **エクスポート関数**: 標準的な EmEditor プラグインのエントリポイント（`DllMain`, `OnCommand`, `QueryStatus` など、SDK で要求されるもの）をエクスポートすること。
- **FFI**: C ABI に合わせるため、`libc` または `windows` クレートの型を使用すること。

### 成果物
- `crate-type = ["cdylib"]` を含む `Cargo.toml`。
- FFI 定義を含む `src/lib.rs`。
- コンパイルされた `.dll` ファイル。