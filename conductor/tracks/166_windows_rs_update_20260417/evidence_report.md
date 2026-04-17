# Evidence Report: windows-rs 0.73 Update

## Discovery Summary
- **課題**: `windows` クレートの 0.58 から 0.73 へのアップデート。
- **制約**:
    - アーキテクチャ（GUI, Infra, Application, Domain）の維持。
    - Windows 10/11 互換性の維持。
- **成功条件**:
    - 全てのモジュールがビルド可能。
    - 既存のテストがパス。
    - 実機（EmEditor）での主要機能（描画、入力、IME、ConPTY、設定）が正常。

## Codebase Findings
- **使用箇所**: `src/` 配下に 87 箇所の `windows::` 依存。
- **GitHub Release (0.58 -> 0.73) からの重要知見**:
    - **v0.73 (Release 73)**:
        - **ポインタ型の厳密化**: `*const` と `*mut` の区別がメタデータレベルで導入。
        - **`VARIANT`/`PROPVARIANT` の `Drop` 削除**: 自動解放されないため手動 `VariantClear` が必要。
    - **v0.72 (Release 72)**:
        - **`HSTRING`/`BSTR` の `Display` トレイト削除**: `println!` や `log!` での直接使用不可。`to_string()` 等が必要。
    - **v0.69 (Release 69)**:
        - **`windows-link` への完全移行**: `windows-targets` が非推奨化。
    - **v0.66 (Release 66)**:
        - **`extern "C"` への統一**: 従来の `cdecl` 等が整理。
    - **v0.62 (Release 62)**:
        - **クレートの細分化**: `windows-numerics`, `windows-future`, `windows-collections` が独立。
        - **`BOOL` の移動**: `windows-result` に `BOOL` 型が含まれるようになり、巨大な `windows` クレートへの依存を減らせる。
    - **v0.61 (Release 61)**:
        - **`BOOLEAN` -> `bool` へのリマップ**: 1バイトの `BOOLEAN` が Rust の `bool` に対応。
        - **`Ref`/`OutRef` の導入**: COM インターフェースの引数扱いの変更。
    - **v0.60 (Release 60)**:
        - **`HSTRING` の `Deref` 実装**: 文字列操作の簡略化。

## Key Files To Read
- `Cargo.toml`: 依存関係の定義。
- `src/infra/repository/emeditor_config_repository_impl.rs`: レジストリ操作。
- `src/infra/driver/emeditor_io_driver.rs`: ダークモード検出（レジストリ）。
- `src/gui/driver/ime_gui_driver.rs`: スレッドID取得、キャレット制御。
- `src/lib.rs`: DLL エントリポイント。

## Clarifying Questions
1. **Q**: レジストリ操作の `windows-registry` 移行は必須か？
   - **A**: はい。ユーザーの「推奨実装への変更」指示に従い、刷新します。
2. **Q**: `BOOL` 型のインポート元はどうすべきか？
   - **A**: 推奨に従い、可能な限り `windows-result` や `windows-core` からインポートするように整理します。

## Architecture Options
| 案 | 概要 | リスク |
| :--- | :--- | :--- |
| **Pragmatic Update** | 最小限のコンパイルエラー修正。 | 将来の技術負債。 |
| **Recommended Update** | 細分化されたクレート（`core`, `registry`, `link`, `result`, `strings`）を活用し刷新。 | 修正範囲が広くなるが、公式ベストプラクティスに準拠。 |

**推奨案**: **Recommended Update**

## Evidence
- `windows-rs` Release Notes (GitHub): v0.58〜v0.73 の破壊的変更履歴を全件確認済み。
- `windows-registry`, `windows-core`, `windows-result`, `windows-strings`: 最新の独立クレート群。
