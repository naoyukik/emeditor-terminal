# **AcePilot版：Rust/Win32 レイヤードアーキテクチャの掟 (Strict Rigid版)**

AIエージェントに「ノリ」でコードを書かせず、物理的なファイル名と配置によって役割と境界を統制するための、最高レベルに厳格な設計ガイドです。

## **1. コンセプト：物理的隔離による統制**

ファイル名に特定の接尾辞（サフィックス）を義務付け、そのファイルが「知ってよい情報」と「依存してよい方向」を物理的に固定します。

| レイヤー | 接尾辞 | 役割とAIへの指示の掟 |
| :---- | :---- | :---- |
| **1. Presentation (GUI)** | `_resolver.rs` | OSメッセージ（WndProc等）やユーザー入力の解釈。Application層へのディスパッチ。 |
| **1.1. GUI Driver** | `_gui_driver.rs` | **描画・IME等のWin32操作**。Presentation層から呼び出される「手足」。 |
| **1.5. DTO (GUI用)** | `_request.rs` / `_response.rs` | **Presentation層専用**。Windowsメッセージ構造や外部入力と1:1で対応させる。 |
| **2. Application** | `_workflow.rs` | 処理のシナリオ（ユースケース）。**「翻訳者」**としてDTOとEntityを変換し、ドメインを動かす。 |
| **2.5. DTO (App用)** | `_input.rs` / `_result.rs` | **Application層の境界型**。Entityを直接外部に漏らさないための詰め替え型。 |
| **3. Domain** | `_entity.rs` / `_value.rs` | アプリの核心。**外部のDTO（Request/Input）を一切参照してはならない。** |
| **3.1. Domain Service** | `_domain_service.rs` | Entityに収まらない、ドメイン概念に基づく計算やロジック。 |
| **3.5. Repository (IF)** | `_repository.rs` | **Trait定義**。外部へのデータ要求を定義する。Domain層に属する。 |
| **4. Infrastructure** | `_repository_impl.rs` | Repository Traitの具象実装。IO Driverを使用して外部リソースへアクセスする。 |
| **4.1. IO Driver** | `_io_driver.rs` | **ConPTY・Editor SDK等の外部操作**。Infrastructure層から呼び出される「手足」。 |

## **2. Windows API (windows-rs) の隔離命令**

Windows APIの型は汚染力が強いため、以下の隔離を徹底せよ。

* **`_gui_driver.rs` / `_io_driver.rs`**: `windows` クレートの型（`HWND`, `RECT`, `COLORREF`等）を扱ってよい唯一の場所。
* **`_resolver.rs`**: メッセージループからWin32型を受け取るが、即座に内部型（Input DTO等）へ変換せよ。
* **重罪**: `Domain` 層および `Application` 層において、`windows` クレートを直接 `use` することを固く禁ずる。必要な定数（VKコード等）はDomain層でPure Rustな定義として再定義せよ。

## **3. ファイル分離とDTOの掟**

AIのコンテキスト理解を助け、編集の正確性を高めるために以下の分離を徹底せよ。

* **1ファイル1責務**: `DTO`, `Input`, `Result`, `Entity` はすべて別ファイルに切り出す。
* **AI最適化**: ファイルを小さく保ち、シンボルジャンプに頼らずともファイル構成だけで依存関係を理解できるようにせよ。

## **4. Repositoryの実装パターン**

* **定義 (Domain)**: `pub trait FooRepository { ... }`
* **実装 (Infrastructure)**: `pub struct FooRepositoryImpl { driver: FooIoDriver }`
* **DI (Application)**: `_workflow.rs` はコンストラクタで `Box<dyn FooRepository>` などの抽象を受け取り、具象クラス（`Impl`）に直接依存してはならない。

## **5. 依存方向とデータフローの絶対則**

依存の矢印は常に「内側（Domain/Application）」へ向かい、Win32 APIは両端の「Driver」に封印する。

```
【入力境界】                                              【出力・操作境界】
Presentation ───→ Application ───→ Domain ←─── Infrastructure
    (Resolver)          (Workflow)        (Entity)      (RepositoryImpl)
      │                                                   │
      └─→ [_gui_driver]                                  [_io_driver] ←┘
          (描画・IME)                                   (ConPTY・Editor)
```
※ `_gui_driver` と `_io_driver` は実装詳細であり、各境界の「最外周」に位置する。

## **6. ユビキタス言語辞典 (Ubiquitous Language Dictionary)**

特定の文脈において、以下の命名を「正しいドメイン用語」として強制する。

| 概念 | 推奨される名称 | 除外すべき汎用名 |
| :--- | :--- | :--- |
| ウィンドウ共有データ | `window_data` | `data` |
| 入力用バイト列 | `input_bytes` | `data`, `buf` |
| 文字属性（太字等） | `is_bold`, `is_italic` 等 | `bold`, `italic` (形容詞単体) |
| 可視性状態 | `is_visible` | `visible` |
| キー押下状態 | `is_ctrl_pressed` 等 | `ctrl`, `shift` (単一名) |
| スクロール位置 | `viewport_offset`, `scroll_pos` | `pos`, `offset` |
| 履歴・バックバッファ | `history`, `back_buffer` | `old_data`, `cache` |
