# Track Specification: 15-refactor-architecture

## 1. Overview
現状、`src/` 直下に混在しているファイルを役割ごとに整理し、レイヤードアーキテクチャ（Domain, Infra, GUI）を導入する。特に `custom_bar.rs` から描画ロジックを `TerminalRenderer` として分離し、コードの可読性と保守性を向上させる。

## 2. Functional Requirements
### 2.1 ディレクトリ再編成
- `src/domain/`: ビジネスロジック。外部（Win32等）に依存しないコード。
    - `terminal.rs` を移動。
- `src/infra/`: 外部APIやOSへの依存。
    - `conpty.rs`, `editor.rs` を移動。
- `src/gui/`: ユーザーインターフェースと描画。
    - `custom_bar.rs` を移動。
    - `renderer.rs` を新規作成。

### 2.2 描画ロジックの分離 (`renderer.rs`)
- `struct TerminalRenderer` を定義。
- `custom_bar.rs` の `WM_PAINT` 処理（GDIによるテキスト描画、カーソル描画）を `TerminalRenderer::render` メソッドへ移管する。
- 描画に必要なリソース（フォントハンドル等）の管理も `TerminalRenderer` の責務とする。

### 2.3 モジュール結合の修正
- `lib.rs` および各階層の `mod.rs` を調整し、新しいパスでのビルドを通す。
- 既存のグローバル変数 (`TERMINAL_DATA` 等) へのアクセスパスを修正する。

## 3. Non-Functional Requirements
- **ビルドの維持**: リファクタリング前後で機能（ターミナル表示・入力）が一切損なわれないこと。
- **パフォーマンス**: 描画ロジックの分離による顕著なパフォーマンス低下がないこと。

## 4. Acceptance Criteria
- [ ] `cargo build` が正常に終了する。
- [ ] EmEditor上でターミナルが以前と同様に起動、表示、入力できる。
- [ ] `src/配下のディレクトリ構造が計画通りになっている。`
- [ ] `custom_bar.rs` から GDI 関連の低レベルな呼び出しが消え、`renderer` 経由になっている。

## 5. Out of Scope
- 新機能の追加（スクロールバッファ、マルチタブ等）。
- 非同期処理の根本的な見直し。
