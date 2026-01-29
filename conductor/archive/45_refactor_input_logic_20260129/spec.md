# Track 45_refactor_input_logic Specification

## 1. Overview
`src/gui/custom_bar.rs` に集中しているキーボード入力処理（フックおよび変換ロジック）を、プロジェクトのレイヤードアーキテクチャ規約に従って分離・再構築する。これにより、「神クラス」の解消、責任の明確化、およびユニットテストの導入を実現する。

## 2. Functional Requirements
- **Domain層への抽出**: キーコード（Virtual-Key）から VT シーケンス（ANSI エスケープコード）への変換ロジックを `src/domain/input.rs` に抽出する。
- **Infra層への抽出**: Low-Level Keyboard Hook の管理（Set/Unhook）およびフックプロシージャを `src/infra/input.rs` に抽出する。
- **インターフェースの導入**: 入力処理に関する抽象化（Trait）を定義し、各層間の結合度を下げる。
- **メッセージパッシングの実装**: フックプロシージャからの通知は Windows メッセージを使用して `CustomBar` へ送信する。
- **ユニットテストの追加**: 抽出した変換ロジックに対し、網羅的なユニットテストを実装する。

## 3. Architecture Details
- **src/domain/input.rs**:
    - `KeyTranslator` トレイト: キー入力を抽象化する。
    - `VtSequenceTranslator`: `KeyTranslator` を実装し、VK からシーケンスを生成する。純粋なロジックとして実装し、Win32 API 依存を最小化する。
- **src/infra/input.rs**:
    - `KeyboardHook`: フックのライフサイクルを管理する構造体。
    - フックプロシージャ: キー入力を検知すると、指定されたウィンドウハンドルへメッセージを `PostMessage` する。
- **src/gui/custom_bar.rs**:
    - フックの初期化・解除のみを担当し、受信したメッセージを `TerminalService` 等へ橋渡しする。

## 4. Acceptance Criteria
- [ ] `custom_bar.rs` の行数が削減され、入力ロジックが排除されていること。
- [ ] キーボード入力機能（文字入力、特殊キー、Ctrl/Alt 組み合わせ等）が従来通り動作すること。
- [ ] `src/domain/input.rs` のテストが全てパスすること。
- [ ] ビルドエラーおよび Clippy の警告が発生しないこと。
