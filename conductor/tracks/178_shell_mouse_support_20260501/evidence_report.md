# Evidence Report: Shell Mouse Support (Issue #178)

## Discovery Summary
- **課題**: 現状、SGR 1006 マウストラッキングが無効なシェル環境では、マウスイベントが全て無視されるか EmEditor のデフォルト挙動にフォールバックされる。また、Shift 押下時は常に EmEditor 側が優先（バイパス）される。
- **制約**: 「Strict Rigid Layered Architecture」を維持するため、Application 層 (`TerminalWorkflow`) から直接 Win32 API (Clipboard) を叩くことは禁止。
- **成功条件**:
  - 右クリック時にクリップボードの内容がペーストされる。
  - プロンプト行（カーソルと同一行）のクリックで、左右矢印キーが擬似送信されカーソルが移動する。
  - Shift 押下時の挙動がユーザーの期待通りに変更される。

## Codebase Findings
### 1. マウスイベントの流入経路
- **File**: `src/gui/resolver/window_message_resolver.rs`
- **Logic**: `dispatch_mouse_event` 関数内で Win32 メッセージを `MouseEvent` ドメインモデルに変換。
- **Point**: 
  - 現在、`is_shift_pressed` が true の場合、即座に `default_window_proc` (バイパス) を返している (L256-258)。
  - `window_data.service.handle_mouse_event(event)` を呼び出し、戻り値が `Ok(true)` でない場合はバイパスされる。

### 2. トラッキングモードの判定
- **File**: `src/application/terminal_workflow.rs`
- **Method**: `handle_mouse_event`
- **Point**: 
  - `self.buffer.get_mouse_tracking_mode() == MouseTrackingMode::None` の場合、即座に `Ok(false)` を返している。ここがシェル向け機能の実装ポイントとなる。

### 3. カーソル位置の取得
- **File**: `src/domain/model/terminal_buffer_entity.rs`
- **Method**: `get_cursor_pos()`
- **Point**: 現在の論理カーソル座標 `(x, y)` を取得可能。クリック位置 `event.y` と比較することで同一行判定が可能。

## Clarifying Questions
1. **Shell モード時の Shift+Click**: トラッキング無効時（シェル環境）で Shift を押しながらクリックした場合も、ターミナル側で処理（ペースト等）を行うべきか、それともこの場合は EmEditor のネイティブ選択を優先（バイパス）すべきか？
2. **バイパス手段の確保**: Shift+Click をターミナルに送るように変更した場合、EmEditor 本体のテキスト選択を行うための「代替バイパス手段」は必要か？（例：Ctrl+Shift+Click 等）

## Architecture Options
| 案 | メリット | デメリット |
| :--- | :--- | :--- |
| **案1: Minimal (Parameter Passing)** | 実装が極めて単純。 | `handle_mouse_event` の引数にクリップボード文字列を追加する必要があり、インターフェースが汚れる。 |
| **案2: Clean (Repository Pattern)** | 責務が明確。テスト時にモック可能。 | `ClipboardRepository` トレイトと、Win32 API を用いた実装クラスを新設する工数が必要。 |

### 推奨案: 案2 (Clean Architecture)
プロジェクトの「Strict Rigid」原則に従い、Infrastructure 層に `ClipboardRepositoryImpl` を実装し、Application 層へ注入する形式を推奨する。

## Evidence
- `src/gui/resolver/window_message_resolver.rs:256` (Shift バイパス処理)
- `src/application/terminal_workflow.rs:141` (トラッキングモード判定)
- Windows Terminal の挙動: デフォルトで右クリック貼り付けが有効。
