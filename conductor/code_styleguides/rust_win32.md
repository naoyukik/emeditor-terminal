# EmEditor Terminal Coding Conventions & Guidelines

## 0. ドキュメントの構成
本ドキュメントは、上位規約である `general.md`（汎用的な設計原則）を前提とし、Rust および Win32 API（EmEditor プラグイン）という特殊な開発環境における固有の実行規則を定めるものである。

## 1. 全般原則 (General Principles)

### 1.1 The Rust Way
- **Rustfmt / Clippy 準拠**: `cargo fmt` と `cargo clippy` は絶対的な基準とする。Clippy の警告 (Warnings) は全て修正する。
- **命名規則**:
    - 変数・関数・モジュール: `snake_case`
    - 型 (Struct, Enum, Trait): `PascalCase`
    - 定数: `SCREAMING_SNAKE_CASE`
    - **例外**: Win32 API の FFI 境界 (`DllMain`, `OnCommand` 等) のみ、EmEditor SDK に合わせて `PascalCase` または `CamelCase` を許容する。ただし、それらは `lib.rs` 内に限定し、内部ロジックには持ち込まない。
    - **ファイル名と構造体**: 主要な構造体 (Struct) を定義するファイルは、その構造体名を `snake_case` に変換したものをファイル名とすること。（例: `struct TerminalService` → `terminal_service.rs`）
- **DDD / ユビキタス言語の推奨**:
    - 単なる `x`, `tmp`, `data` などの無機質な名前は避け、ドメインにおける意味（例: `cursor_position_x`, `temporary_input_buffer`）を反映させること。
    - **bool型**: `is_visible`, `has_focus`, `can_scroll` のように、状態（State）や能力（Capability）を明確にする述語形式とする。
    - プロジェクト内で用語を統一する（例: `Viewport` と `Window` を混同しない）。

#### 各種命名規約

##### ファイル命名規約

| 種類 | 命名パターン | 例 |
|------|------------|-----|
| モデル（エンティティ） | `{entity_name}.rs` (単数形・スネークケース) | `access_token.rs`, `facet.rs`, `login_credential.rs` |
| サービス | `{feature}_service.rs` | `message_service.rs`, `facet_service.rs` |
| リポジトリトレイト | `{feature}_repository.rs` | `login_repository.rs`, `message_repository.rs` |
| リポジトリ実装 | `{feature}_repository_impl.rs` | `login_repository_impl.rs` |
| DTO | `{entity}_dto.rs` | `facet_dto.rs`, `access_token_dto.rs` |
| ワークフロー | `{action}_workflow.rs` | `send_message_workflow.rs`, `authentication_workflow.rs` |
| プレゼンテーション層 | `{feature}_resolver.rs` | `message_resolver.rs` |
| モジュール定義 | `mod.rs` | ─ |

---

##### 構造体・列挙型・トレイト命名規約

| 種類 | 命名パターン | 例 |
|------|------------|-----|
| 構造体（モデル） | PascalCase | `AccessToken`, `Facet`, `LoginCredential` |
| 構造体（DTO） | PascalCase + `Dto`サフィックス | `FacetDto`, `AccessTokenDto` |
| 構造体（実装） | PascalCase + `Impl`サフィックス | `LoginRepositoryImpl`, `MessageServiceImpl` |
| 列挙型（enum） | PascalCase | `FeatureMode`, `Receivers`, `Command` |
| トレイト | PascalCase | `LoginRepository`, `MessageService`, `HttpService` |

---

##### 関数命名規約

| 用途 | 命名パターン | 例 |
|------|------------|-----|
| 一般関数 | snake_case（動詞ベース） | `create_facets()`, `set_post_message()` |
| コンストラクタ | `new` または `create` | `AccessToken::new()`, `Facet::create()` |
| ゲッター | `get_` プレフィックス | `get_identifier()`, `get_value()` |
| 変換関数 | `to_` プレフィックス | `to_facet_dto()`, `to_facet_index()` |
| ヘルパー関数（private） | snake_case | `get_url_string()` |

---

##### 変数・フィールド命名規約

- **変数名・フィールド名**: snake_case
  ```rust
  pub fn example_function() {
    let access_token = AccessToken::new();
  }
  ```

- **構造体フィールド**: snake_case（JSONシリアライズ時は`#[serde(rename = "camelCase")]`で変換）
  ```rust
  #[derive(Serialize, Deserialize)]
  pub struct FacetIndex {
      #[serde(rename = "byteStart")]
      byte_start: u16,
      #[serde(rename = "byteEnd")]
      byte_end: u16,
  }
  ```

### 1.2 単一責任の原則 (SRP) & ファイルサイズ
- **1ファイル 300行制限**: 1つのファイルが300行を超えたら、設計の見直しを検討せよ。
- **責務の分離**: UI描画、イベント処理、ドメインロジック、Infrastructure ラッパーは、それぞれ別のモジュール・構造体に分割する。

## 2. アーキテクチャ・レイヤー構造

レイヤードアーキテクチャを採用する。依存の方向は常に **外側 → 内側** でなければならない。

```
┌───────────────────┐
│ GUI(Presentation) │
└──────┬────────────┘
       │ (依存)
       ↓
┌─────────────┐     ┌────────────────┐
│ Application │     │ Infrastructure │
└──────┬──────┘     └───────┬────────┘
       │ (依存)             │ (依存)
       ↓                    ↓
┌────────────────────────────────────┐
│               Domain               │
│      (Entities, Repository Traits) │
└────────────────────────────────────┘
```

### Layer 1: Domain (`src/domain/`)
- **純粋性**: **`windows` クレートへの依存を極力排除する。**
- **責務**: ターミナルの状態（バッファ、カーソル、履歴）、ANSI パース結果のデータ表現。
- **データモデリング**: プリミティブ型（`u16`, `bool` 等）の羅列を避け、意味のある単位で構造体（`struct`）や列挙型（`enum`）を定義すること。
    - **Bad**: `fn handle_key(vk_code: u16, ctrl: bool, shift: bool)`
    - **Good**: `fn handle_key(key: InputKey)` where `struct InputKey { code: KeyCode, modifiers: Modifiers }`
- **インターフェース定義**: 外部リソース（設定、永続化など）へのアクセスは、ここで `Trait`（Repository Interface）として定義する。実装には依存しない。
- **テスト**: ユニットテストでカバレッジ 90% 以上および十分な動作保証を目指す。ここに UI ロジックを持ち込んではならない。

### Layer 2: Application (`src/application/`)
- **責務**: Domain と Infrastructure/GUI の調整役（ユースケース）。
- **具体例**:
    - `handle_user_input(input: char)`: GUI 層から文字入力を受け取り、それを Domain 層のバッファに反映し、必要に応じて Infrastructure 層 (ConPTY) へ送信する。
    - `resize_terminal(cols, rows)`: ウィンドウのリサイズイベントを Domain 層（バッファ再構成）と Infrastructure 層 (ConPTY リサイズ) に伝播させる。
- **DI (Dependency Injection)**: 必要な Repository の実装をコンストラクタ等で受け取り、Domain ロジックに渡す。
- **ルール**: `TerminalService` は、具体的な描画方法 (GDI) を知ってはならない。

### Layer 3: Infrastructure (`src/infra/`)
- **責務**: OS (Windows API) や外部システム (ConPTY) との具体的な対話。
- **実装**: Domain 層で定義された Repository Trait を実装する。
    - 例: `Win32ClipboardRepository`, `ConptyInputRepository`
- **ルール**: Win32 API の生の操作はここに閉じ込める。Domain 層が使いやすい安全なラッパーを提供する。

### Layer 4: GUI(Presentation) (`src/gui/`)
- **責務**: 画面への描画 (GDI)、ユーザー入力の受け取り。
- **ルール**: `wnd_proc` のような巨大な関数を作らず、メッセージ処理ごとにハンドラ関数に委譲する。

## 3. 実装ガイドライン

### 3.1 安全性 (Safety & Unsafe)
- **`unsafe` の局所化**: `unsafe` ブロックは最小限にする。
- **コメント義務**: なぜ `unsafe` が必要なのか、なぜそれが安全だと言えるのか (Safety Comment) を必ず記述する。
- **FFI 境界**: Win32 API から受け取った生ポインタは、即座に安全な Rust の型に変換するか、ラップする。

### 3.2 Win32 API ハンドリング
- **型安全性**: `HWND`, `HDC`, `HFONT` などのハンドルは、可能な限り `Send` / `Sync` を実装したラッパー構造体を通して扱うか、New Type Pattern を用いて型安全性を確保する。
- **リソース管理**: `Drop` トレイトを使用し、RAII パターンで GDI オブジェクト (Pen, Brush, DC など) の解放漏れ (リーク) を絶対に防ぐ。

### 3.3 エラーハンドリング
- **Panic 禁止**: DLL として動作するため、`unwrap()` や `expect()` によるパニックは避け、`Result` を返し、トップレベルで適切にログ出力・復帰を行う。
- **ロギング**: エラー発生時は必ず `log::error!` を出力する。
- **ログファイル**: `$env:TEMP\emeditor_terminal.log` に出力されている

## 4. テスト戦略

### 4.1 ユニットテスト (Unit Tests)
- **Domain 層**: **カバレッジ 90% 以上**を目指す。残り 10% は、外部依存が絡む複雑なエラーハンドリングのエッジケースなど、テストコストが極端に高い部分を許容する。ビジネスロジック（ANSI パース、バッファ制御、文字幅計算など）はすべて `cargo test` で検証可能にする。
- **Application 層**: ドメインとの協調をテストする。Infrastructure 層への依存は、必要に応じてトレイトによるモック (Mocking) を検討する。

### 4.2 UI/Infrastructure のテスト
- Win32 API に依存する部分は自動テストが困難なため、ロジックを極限まで分離し、純粋な計算部分（例：キー入力から VT シーケンスへの変換テーブル）のみをユニットテストの対象とする。

