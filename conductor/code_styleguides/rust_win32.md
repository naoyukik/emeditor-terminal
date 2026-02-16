# **EmEditor Terminal Coding Conventions & Guidelines (汎用規約版)**

本ドキュメントは、Rust および Win32 API（EmEditor プラグイン）という特殊な開発環境における、コード品質と保守性を担保するための汎用的な実行規則を定めるものである。物理的な構成については `architecture_rules.md` を参照せよ。

## **1. 全般原則 (General Principles)**

### **1.1 The Rust Way**
- **標準ツール準拠**: `cargo fmt` と `cargo clippy` は絶対的な基準とする。Clippy の警告 (Warnings) は全て修正せよ。
- **基本命名規則**:
    - 変数・関数・モジュール: `snake_case`
    - 型 (Struct, Enum, Trait): `PascalCase`
    - 定数: `SCREAMING_SNAKE_CASE`
    - **例外**: Win32 API の FFI 境界 (`DllMain` 等) のみ `lib.rs` 内限定で `PascalCase` を許容する。
- **bool型の掟**: 必ず `is_visible`, `has_focus`, `can_scroll` のように、状態や能力を明確にする述語形式（Predicate）とせよ。

### **1.2 関数とメソッドの設計**
- **ドメイン指向の引数**: プリミティブ型（`u16`, `bool` 等）の羅列を避け、意味のある単位で構造体や列挙型を定義して渡すこと。
    - **Bad**: `fn handle_key(vk_code: u16, ctrl: bool, shift: bool)`
    - **Good**: `fn handle_key(key: InputKey)`
- **コンストラクタ**: 原則として `new` メソッドを提供せよ。

---

## **2. Win32 API ハンドリング**

### **2.1 型安全とカプセル化**
- **ハンドル管理 (RAII)**: `HWND`, `HDC`, `HFONT` などのハンドルは、必ず `Drop` トレイトによる自動解放（RAIIパターン）を行え。
- **New Type Pattern**: 生のハンドルをそのまま持ち回さず、可能な限り `Send` / `Sync` を考慮したラッパー構造体を通して扱え。
- **FFI境界**: Win32 API から受け取った生ポインタは、即座に安全な Rust の型に変換するか、ラップせよ。

---

## **3. 安全性と信頼性**

### **3.1 unsafe の局所化**
- `unsafe` ブロックは最小限に留め、なぜその操作が安全であると言えるのかを記述する **Safety Comment** を必須とする。

### **3.2 エラーハンドリング**
- **Panic禁止**: DLL として動作するため、`unwrap()` や `expect()` によるパニックは避け、`Result` を返してトップレベルで適切にログ出力・復帰を行え。
- **ロギング**: エラー発生時は必ず `log::error!` を出力せよ。ログは `$env:TEMP\emeditor_terminal.log` を確認。

---

## **4. ファイルサイズ制限**
- **300行制限**: 1ファイルが300行を超えたら、設計の見直し（ファイル分割）を検討せよ。

---

## **5. テスト戦略**

| レイヤー | 目標と方針 |
| :--- | :--- |
| **Domain** | **カバレッジ90%以上**。ANSIパース、バッファ制御、文字幅計算等はすべて `cargo test` で検証可能にせよ。 |
| **Application** | ドメインとの協調をテスト。必要に応じてインターフェース（Repository）をモック化せよ。 |
| **Infra / GUI** | Win32 API 依存部からロジックを極限まで分離し、純粋な計算部分のみをテスト対象とせよ。 |
