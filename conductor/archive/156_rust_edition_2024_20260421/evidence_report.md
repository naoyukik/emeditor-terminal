# Evidence Report: Rust Edition 2024 Migration (Issue #156)

## Discovery Summary
- **課題**: Rust Edition 2024 (rustc 1.95.0) への移行に伴い、言語仕様の厳格化によるコンパイルエラーおよび警告が発生する。
- **制約**: エディション移行に必須な最小限の修正にとどめ、既存の主要機能（起動、IME連携、描画）を回帰させない。
- **成功条件**: `edition = "2024"` 設定下で `cargo check` および `cargo clippy` が成功し、実機での主要動作が正常であること。

## Codebase Findings (Critical Error/Warning Points)

### 1. Extern Blocks Must Be Unsafe (Error)
- **ファイル**: `src/gui/driver/scroll_gui_driver.rs:19`
- **内容**: Edition 2024 では `extern` ブロックに `unsafe` キーワードが必須。
- **修正案**: `extern "system"` を `unsafe extern "system"` に変更。

### 2. Unsafe Attribute Usage (Error)
- **ファイル**: `src/lib.rs` (51, 74, 83, 95, 103, 107, 111, 116行目)
- **内容**: `#[no_mangle]` 属性を `#[unsafe(no_mangle)]` に変更する必要がある。

### 3. Unsafe Operation in Unsafe Function (Warning/Error)
- **ファイル**: 
    - `src/gui/common/mod.rs`: `pixels_to_points`, `points_to_pixels` 関連。
    - `src/gui/driver/config_gui_driver.rs`: `update_font_label`, `settings_dlg_proc` 等。
- **内容**: `unsafe fn` の中でも `unsafe` な操作（Win32 API 呼び出し等）を `unsafe {}` ブロックで囲む必要がある。
- **修正案**: 各関数のボディ内で Win32 API 呼び出し箇所を明示的に `unsafe {}` ブロック化し、適切な Safety Comment を付与する。

### 4. Dependency Issues
- **現状**: `time` クレートの最新バージョンが rustc 1.88+ を要求していたため、`rustup update stable` (1.95.0) を実施。環境は最新になったため、今後の `cargo update` で依存関係の問題も解消される。

## Clarifying Questions
1. **コミット粒度**: 段階的な移行を希望されているため、エラー修正（`extern`, `no_mangle`）と警告解消（`unsafe {}` ブロック化）を別個の PR/コミットにするか、フェーズごとにまとめるか。
    - **AcePilot案**: フェーズ 2 で警告解消と依存更新、フェーズ 3 でエディション変更と残りのエラー修正を一括で行う。

## Architecture Options
- **Minimal Changes (推奨)**: 言語仕様の要求通りに最小限のキーワード追加と属性変更を行う。
- **Clean Unsafe Management**: 警告解消に伴い、Win32 API 呼び出しの各所に詳細な Safety Comment を付与し、既存の `unsafe fn` の必要性を再検討する。

## Recommendation
- **推奨案**: **Minimal Changes + Safety Documentation**。
    - Edition 2024 が要求する明示的な `unsafe` ブロック化は、プロジェクトの `unsafe` 局所化方針（Rust Coding Convention）に合致するため、これを機にすべての `unsafe` 呼び出し箇所を精査し、ドキュメント化する。

## Evidence (Tool Output / Reference)
- `cargo check --all-targets` (rustc 1.95.0, edition 2024): `extern blocks must be unsafe`, `unsafe attribute used without unsafe`, `warning[E0133]: unsafe_op_in_unsafe_fn`
- [Rust Edition Guide (2024)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
