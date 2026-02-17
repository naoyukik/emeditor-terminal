# **AcePilot版：Rust/Win32 レイヤードアーキテクチャの掟 (Strict Rigid版)**

AIエージェントに「ノリ」でコードを書かせず、物理的なファイル名と配置によって役割と境界を統制するための、最高レベルに厳格な設計ガイドである。

## **1. コンセプト：物理的隔離による統制 (The Suffix Rule)**

ファイル名に特定の接尾辞（サフィックス）を義務付け、そのファイルが「知ってよい情報」と「依存してよい方向」を物理的に固定する。

| レイヤー | 接尾辞 | 役割とAIへの指示の掟 |
| :---- | :---- | :---- |
| **1. Presentation** | `_resolver.rs` | OSメッセージ（WndProc等）の解釈。Application層へのディスパッチ。 |
| **1.1. GUI Driver** | `_gui_driver.rs` | **描画・IME等のWin32操作**。最外周の「手足」。 |
| **1.5. DTO (GUI用)** | `_request.rs` / `_response.rs` | Presentation層専用。Windowsメッセージ構造と1:1で対応。 |
| **2. Application** | `_workflow.rs` | 処理のシナリオ（ユースケース）。DTOとEntityを変換する。 |
| **2.5. DTO (App用)** | `_input.rs` / `_result.rs` | Application層の境界型。Entityを直接外部に漏らさない。 |
| **3. Domain** | `_entity.rs` / `_value.rs` | アプリの核心。**外部DTO（Request/Input）を一切参照してはならない。** |
| **3.1. Domain Service** | `_domain_service.rs` | Entityに収まらないドメインロジック。 |
| **3.5. Repository (IF)** | `_repository.rs` | **Trait定義**。Domain層に属し、外部へのデータ要求を定義する。 |
| **4. Infrastructure** | `_repository_impl.rs` | Repository Traitの具象実装。IO Driverを使用する。 |
| **4.1. IO Driver** | `_io_driver.rs` | **ConptyIoDriver・Editor SDK等の外部操作**。最外周の「手足」。 |

## **2. Windows API (windows-rs) の隔離命令**

Windows APIの型は汚染力が強いため、以下の隔離を徹底せよ。

* **`_gui_driver.rs` / `_io_driver.rs`**: `windows` クレートの型（`HWND`, `RECT`, `COLORREF`等）を扱ってよい唯一の場所。
* **`_resolver.rs`**: Win32型を即座に内部型（Input DTO等）へ変換せよ。
* **重罪**: `Domain` 層および `Application` 層において、`windows` クレートを直接 `use` することを固く禁ずる。必要な定数（VKコード等）はDomain層でPure Rust定義として再定義せよ。

## **3. ファイル分離とDTOの掟**

AIのコンテキスト理解を助け、編集の正確性を高めるために以下の分離を徹底せよ。

* **1ファイル1責務**: `DTO`, `Input`, `Result`, `Entity` はすべて別ファイルに切り出すこと。
* **AI最適化**: ファイルを小さく保ち、シンボルジャンプに頼らずともファイル構成だけで依存関係を理解できるようにせよ。

## **4. 依存方向とデータフローの絶対則**

依存の矢印は常に「内側（Domain/Application）」へ向かい、Win32 APIは両端の「Driver」に封印する。

```
【入力境界】                                              【出力・操作境界】
Presentation ───→ Application ───→ Domain ←─── Infrastructure
    (Resolver)          (Workflow)        (Entity)      (RepositoryImpl)
      │                                                   │
      └─→ [_gui_driver]                                  [_io_driver] ←┘
          (描画・IME)                                   (ConptyIoDriver・Editor)
```
※右側のレイヤーは自身より左側のレイヤー（外界に近い側）を知ってはならない。

## **5. ユビキタス言語辞典 (Ubiquitous Language Dictionary)**

特定の文脈において、以下の命名を「正しいドメイン用語」として強制する。

| 概念 | 推奨される名称 | 除外すべき汎用名 |
| :--- | :--- | :--- |
| ウィンドウ共有データ | `window_data` | `data` |
| 入力用バイト列 | `input_bytes` | `data`, `buf` |
| 受信テキスト | `output_text` | `output` |
| 文字属性（太字等） | `is_bold`, `is_italic` 等 | `bold`, `italic` (形容詞単体) |
| 可視性状態 | `is_visible` | `visible` |
| キー押下状態 | `is_ctrl_pressed` 等 | `ctrl`, `shift` (単一名) |
| スクロール位置 | `viewport_offset`, `scroll_pos` | `pos`, `offset` |
| 履歴・バックバッファ | `history`, `back_buffer` | `old_data`, `cache` |
