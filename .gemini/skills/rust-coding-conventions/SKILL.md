---
name: rust-coding-conventions
description: Provides coding conventions for Rust/Win32 projects. Includes naming rules, layered architecture, safety guidelines, and test strategy. Use when writing or reviewing code.
---

# Rust/Win32 コーディング規約

## 命名規則

| 対象 | 形式 | 例 |
|------|------|-----|
| 変数・関数・モジュール | `snake_case` | `cursor_position_x` |
| 型 (Struct, Enum, Trait) | `PascalCase` | `TerminalWorkflow` |
| 定数 | `SCREAMING_SNAKE_CASE` | `MAX_BUFFER_SIZE` |
| ファイル名 | 構造体名の`snake_case` | `terminal_service.rs` |
| bool型 | 述語形式 | `is_visible`, `has_focus`, `can_scroll` |

**例外**: Win32 API FFI境界（`DllMain`, `OnCommand`等）のみ`PascalCase`/`CamelCase`を許容。`lib.rs`内に限定。

## アーキテクチャ

依存方向: **外側 → 内側**

```
GUI → Application → Domain ← Infrastructure
```

### レイヤー責務

| レイヤー | 責務 | 制約 |
|----------|------|------|
| Domain (`src/domain/`) | 状態、データ表現、Repository Trait定義 | `windows`クレート依存禁止 |
| Application (`src/application/`) | ユースケース、レイヤー間調整 | 具体的な描画方法を知らない |
| Infrastructure (`src/infra/`) | OS/外部システムとの対話 | Win32 APIをここに閉じ込める |
| GUI (`src/gui/`) | 描画、ユーザー入力受け取り | 巨大関数を作らない |

## データモデリング

プリミティブ型の羅列を避け、意味のある構造体を定義する。

```rust
// Bad
fn handle_key(vk_code: u16, ctrl: bool, shift: bool)

// Good
fn handle_key(key: InputKey)
// where struct InputKey { code: KeyCode, modifiers: Modifiers }
```

## 安全性ガイドライン

- **`unsafe`の局所化**: 最小限に抑える
- **Safety Comment必須**: なぜ`unsafe`が必要か、なぜ安全かを記述
- **FFI境界**: 生ポインタは即座に安全なRust型に変換
- **リソース管理**: `Drop`トレイトでRAIIパターンを使用（GDIオブジェクトのリーク防止）
- **Panic禁止**: `unwrap()`/`expect()`を避け、`Result`を返す

## ファイルサイズ

**1ファイル300行制限**: 超えたら設計見直しを検討

## テスト戦略

| レイヤー | 目標 |
|----------|------|
| Domain | カバレッジ90%以上 |
| Application | ドメインとの協調をテスト、必要に応じてモック |
| Infrastructure/GUI | ロジックを分離し、純粋な計算部分のみテスト |

## 必須ツール

```bash
cargo fmt   # フォーマット
cargo clippy  # 警告は全て修正
```
