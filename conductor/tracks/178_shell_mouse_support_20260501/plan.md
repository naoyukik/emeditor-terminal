# 実装計画: シェル上でのマウス操作のサポート (Issue #178)

## フェーズ 1: 問題の把握と詳細設計 (Discovery & Detailed Design)
- [ ] Task: `autonomous-researcher` による詳細調査と `evidence_report.md` の作成
    - [ ] Sub-task: TUI以外のシェル環境におけるマウス入力イベント（WM_LBUTTONDOWN, WM_RBUTTONDOWN 等）の処理経路を特定する。
    - [ ] Sub-task: 現在のカーソル位置からクリックされた位置までの相対距離を計算し、エスケープシーケンスに変換するロジックの設計方針を `evidence_report.md` にまとめる。
- [ ] Task: 調査結果に基づいた `plan.md` の以降のタスクの具体化。
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 1' (Protocol in workflow.md)

## フェーズ 2: 右クリックによる貼り付け機能の実装 (Right-Click Paste)
- [ ] Task: マウスイベント処理の拡張
    - [ ] Sub-task: `src/gui/` 以下の関連する入力処理モジュールにて、SGR 1006 が無効な状態での `WM_RBUTTONDOWN`（または `WM_RBUTTONUP`）イベントを捕捉する。
    - [ ] Sub-task: クリップボードの内容を取得し、ターミナルへ送信する処理を追加する。
- [ ] Task: Shift修飾キーの送信対応
    - [ ] Sub-task: `WM_RBUTTONDOWN` / `WM_LBUTTONDOWN` 時に Shift キーの押下状態を確認し、EmEditor ネイティブ機能ではなくターミナルへイベントを送信する条件分岐を実装する。
- [ ] Task: コード品質の確保
    - [ ] Sub-task: 追加したロジックのユニットテストを実装する。
    - [ ] Sub-task: `cargo clippy` の実行と警告の解消。
    - [ ] Sub-task: `cargo fmt` の実行。
- [ ] Task: フェーズ 2 の変更をコミット
    - [ ] Sub-task: `git add` と `git commit -m "feat: 右クリックによる無条件貼り付けとShiftバイパス機能を実装"`
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 2' (Protocol in workflow.md)

## フェーズ 3: 左クリックによるカーソル移動機能の実装 (Left-Click Cursor Movement)
- [ ] Task: マウス座標から文字位置への変換と距離計算
    - [ ] Sub-task: 左クリックされたピクセル座標から、ターミナルバッファ上の列位置を計算する。
    - [ ] Sub-task: 現在の論理カーソル位置との差分（水平方向のみ）を算出する。
- [ ] Task: 矢印キーエスケープシーケンスの生成と送信
    - [ ] Sub-task: 算出された差分に基づき、必要な数の Left (`\x1b[D`) または Right (`\x1b[C`) シーケンスを生成して送信する。
- [ ] Task: コード品質の確保とテスト
    - [ ] Sub-task: 追加したロジックに対するユニットテストの実装。
    - [ ] Sub-task: `cargo clippy` の実行と警告の解消。
    - [ ] Sub-task: `cargo fmt` の実行。
- [ ] Task: フェーズ 3 の変更をコミット
    - [ ] Sub-task: `git add` と `git commit -m "feat: 左クリックによる水平方向のカーソル移動機能を追加"`
- [ ] Task: Conductor - ユーザー手動検証 'フェーズ 3' (Protocol in workflow.md)
