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
| **3. Domain** | `_entity.rs` / `_value.rs` | アプリの核心。**外部DTO（Request/Input）や外部プロトコルTraitを一切参照してはならない。** |
| **3.1. Domain Service** | `_domain_service.rs` | Entityに収まらないドメインロジック。 |
| **3.2. Protocol Handler** | `_protocol_handler.rs` | **外部プロトコル（VTE等）の解釈とEntityへの操作命令**。Entityと外部ライブラリの絶縁層。 |
| **3.5. Repository (IF)** | `_repository.rs` | **Trait定義**。Domain層に属し、外部へのデータ要求を定義する。 |
| **4. Infrastructure** | `_repository_impl.rs` | Repository Traitの具象実装。IO Driverを使用する。 |
| **4.1. IO Driver** | `_io_driver.rs` | **ConptyIoDriver・Editor SDK等の外部操作**。最外周の「手足」。 |

**例外規定**: 
* `resource.rs`: `resource.h` から `build.rs` によって自動生成される Win32 リソース定数定義ファイル。ホワイトリストとして許可される。

## **2. Windows API (windows-rs) の隔離命令**

Windows APIの型は汚染力が強いため、以下の隔離を徹底せよ。

* **`_gui_driver.rs` / `_io_driver.rs`**: `windows` クレートの型（`HWND`, `RECT`, `COLORREF`等）を扱ってよい唯一の場所。
* **`_resolver.rs`**: Win32型を即座に内部型（Input DTO等）へ変換せよ。
* **重罪**: `Domain` 層および `Application` 層において、`windows` クレートを直接 `use` することを固く禁ずる。必要な定数（VKコード等）はDomain層でPure Rust定義として re-define せよ。

## **3. プロトコル解析 (vte等) の隔離命令**

ターミナル・シーケンス（ANSI/VT）の解釈ロジックは、外部仕様（プロトコル）と内部データ（Entity）の境界を跨ぐため、特に混同しやすい。以下の隔離を徹底せよ。

* **`_protocol_handler.rs`**: `vte::Perform` 等の外部ライブラリのトレイトを実装してよい唯一の場所。ここが Entity の操作メソッド（`move_cursor`, `put_char` 等）を呼び出す「指揮官」となる。
* **`_entity.rs`**: プロトコル解析用の巨大な `match` 文（`csi_dispatch` 等）をここに記述することを固く禁ずる。Entity は「言われた通りの座標に文字を置く」といった、純粋なバッファ操作に専念せよ。
* **利点**: 将来、別のパーサーライブラリに変更する場合や、テスト用のダミーパースを行う場合に、Entity 側を一切変更せずに済むようになる。

## **4. ファイル分離と行数制限の掟**

AIエージェントのコンテキスト理解を助け、編集の正確性を高めるために以下の分離を徹底せよ。

* **1ファイル1責務**: `DTO`, `Input`, `Result`, `Entity` はすべて別ファイルに切り出すこと。
* **300行制限の義務化**: 1ファイルが300行（コメント等を除く実質行数）を超えた場合、設計に瑕疵がある、あるいは責務が過剰であると判断し、即座にモジュール分割を行うこと。特に `_entity.rs` の肥大化は「神クラス」への予兆であり、絶対に許容しない。
* **AI最適化**: ファイルを小さく保ち、シンボルジャンプに頼らずともファイル構成だけで依存関係を理解できるようにせよ。

## **5. 依存方向とデータフローの絶対則**

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
