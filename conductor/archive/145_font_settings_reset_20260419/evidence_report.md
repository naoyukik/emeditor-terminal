# Evidence Report: Font Settings Reset to 0 (Issue #145)

## 1. Discovery Summary
- **課題**: ターミナルプラグインの起動時およびリサイズ時（`on_size`）に、フォントサイズとウェイトが `0` にリセットされる現象が確認された。
- **制約**: EmEditor プラグイン API (`EE_REG_QUERY_VALUE`) を通じて設定を取得しているが、ウィンドウハンドルの状態やタイミングにより、正しい値が取得できないケースがある。
- **成功条件**: リサイズや再起動後も、常に有効なフォント設定（サイズ > 0）が適用・維持されること。

## 2. Codebase Findings
### 2.1 現状の処理フロー
1. `src/gui/driver/window_gui_driver.rs` の `handle_resize` が `ConfigurationRepository::load` を呼び出す。
2. `src/gui/window/mod.rs` の `ensure_conpty_started` が `TerminalWorkflow::new` を呼び出し、その中で再度 `load` が走る。
3. `load` の実体である `EmEditorConfigRepositoryImpl::load` (`src/infra/repository/emeditor_config_repository_impl.rs`) は `emeditor_query_u32` を使用する。

### 2.2 脆弱な箇所
- **`emeditor_query_u32` (`src/infra/driver/emeditor_io_driver.rs:132`)**:
  - `let mut data: u32 = 0;` で初期化されており、`SendMessageW` が成功を返しつつも値を更新しなかった場合、そのまま `0` を返してしまう。
- **`EmEditorConfigRepositoryImpl::load`**:
  - 取得した値に対するバリデーションがなく、`0` という不正な値をそのまま `TerminalConfig` として返している。
- **`handle_resize`**:
  - `GetParent(hwnd)` で取得したハンドルを使用して設定をロードしているが、リサイズ処理の最中等でこのハンドルが一時的に無効、あるいは意図しない値を返す可能性がある。

## 3. Root Cause Identification
- **根本原因 1**: `emeditor_query_u32` の初期値設計ミス。
- **根本原因 2**: リポジトリ層およびドメイン層でのバリデーション欠如。

## 4. Expected Behavior
- 設定取得に失敗、あるいは取得した値が `0` の場合、`TerminalConfig::default()` の値（`10pt`, `400`）を維持すること。
- ロードされた設定値が有効であることを保証してから、描画メトリクスの更新 (`update_metrics`) を行うこと。

## 5. Architecture Design Options
### 案 A: インフラ層での初期値修正 (Surgical Fix)
- `emeditor_query_u32` で `data` を `default` で初期化する。
- メリット: 変更箇所が最小限。
- デメリット: `0` が明示的に保存されている場合に意図を無視する可能性がある（が、フォントサイズ `0` はそもそも不正なので問題ない）。

### 案 B: リポジトリ層でのバリデーション強化 (Robust Fix)
- `EmEditorConfigRepositoryImpl::load` で取得後に `if size == 0 { default }` のようなガードを入れる。
- メリット: ビジネスロジックとしての正当性を担保できる。

### 推奨案: A と B の併用
- インフラ層で安全な初期値を保証し、リポジトリ層でビジネスルール（サイズ > 0）を適用する。

## 6. Evidence
- **ログ**: `Loaded terminal config: ... font_size=0, weight=0` (Issue #145)
- **ソースコード**:
  - `src/infra/driver/emeditor_io_driver.rs` L134
  - `src/infra/repository/emeditor_config_repository_impl.rs` L44-45
