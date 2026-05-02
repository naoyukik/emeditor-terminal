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

### 3. テキスト選択の描画・イベント捕捉
- **描画**: `src/gui/driver/terminal_gui_driver.rs` の `render_internal` は現在、セル属性のみで色を決定している。選択範囲を外部から与える必要がある。
- **イベント**: `src/gui/resolver/window_message_resolver.rs` の `dispatch_mouse_event` は `is_drag` フラグを生成しており、ドラッグ中（ボタン押下＋移動）のイベントを Application 層へ届ける準備は整っている。
- **状態保持**: `src/domain/model/terminal_buffer_entity.rs` に選択範囲（開始座標・終了座標）を保持するフィールドを追加し、描画時に参照する。

## Clarifying Questions
1. **Shell モード時の Shift+Click**: トラッキング無効時（シェル環境）で Shift を押しながらクリックした場合も、ターミナル側で処理（ペースト等）を行うべきか、それともこの場合は EmEditor のネイティブ選択を優先（バイパス）すべきか？
    - **回答**: ターミナルにフォーカスがある場合は常にターミナルに送る。
2. **選択範囲のクリア**: クリック（ドラッグを伴わない）が行われた際、既存の選択範囲をクリアすべきか？
    - **推測**: 一般的なターミナルの挙動（クリックで解除、ドラッグで再選択）に従う。

## Architecture Options
| 案 | メリット | デメリット |
| :--- | :--- | :--- |
| **案1: Minimal (Parameter Passing)** | 実装が極めて単純。 | インターフェースが汚れる。 |
| **案2: Clean (Repository Pattern)** | 責務が明確。テスト時にモック可能。 | 構築に工数がかかる。 |

### 推奨案: 案2 (Clean Architecture)
プロジェクトの原則に従い、Infrastructure 層に `ClipboardRepositoryImpl` を実装し Application 層へ注入した。テキスト選択についても、`TerminalBufferEntity` が選択状態を保持し、Renderer がそれを参照する形式を採用する。

## Evidence
- `src/gui/resolver/window_message_resolver.rs:256` (Shift バイパス処理)
- `src/application/terminal_workflow.rs:141` (トラッキングモード判定)
- Windows Terminal の挙動: デフォルトで右クリック貼り付けが有効。
