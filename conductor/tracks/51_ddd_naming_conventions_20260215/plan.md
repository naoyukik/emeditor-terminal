# Implementation Plan - Track 51: DDDに基づく命名規則の適用

## Phase 1: 基盤整備と監査
- [ ] Task: 監査用リストの作成。全ファイルから `data`, `buf`, `tmp`, `res` 等の汎用変数および、述語形式になっていない boolean 変数を抽出する。
- [ ] Task: 命名対応表（Ubiquitous Language Dictionary）の作成。既存の不透明な名称と、新しく適用するドメイン用語の対照表を作成し、ユーザーの承認を得る。
- [ ] Task: 変更内容のコミット
- [ ] Task: Conductor - ユーザー手動検証 'Phase 1: 監査' (Protocol in workflow.md)

## Phase 2: Domain層のリファクタリング
- [ ] Task: `src/domain/terminal.rs` 内の属性フラグ（bold, italic等）を `is_xxx` 形式にリネームし、関連する全コードを修正する。
- [ ] Task: `src/domain/model/input.rs` 内のフラグ類を述語形式にリネームする。
- [ ] Task: Domain層の関数引数やローカル変数における汎用名（data, buf等）をドメイン用語へ変更する。
- [ ] Task: ユニットテストを実行し、ドメインロジックの完全性を確認する。
- [ ] Task: 変更内容のコミット
- [ ] Task: Conductor - ユーザー手動検証 'Phase 2: Domain層' (Protocol in workflow.md)

## Phase 3: Application層のリファクタリング
- [ ] Task: `src/application/service.rs` 内の変数名・引数名をドメイン用語へ変更する。
- [ ] Task: boolean型の戻り値やローカル変数を述語形式へ変更する。
- [ ] Task: ユニットテストを実行し、Application層の挙動を確認する。
- [ ] Task: 変更内容のコミット
- [ ] Task: Conductor - ユーザー手動検証 'Phase 3: Application層' (Protocol in workflow.md)

## Phase 4: Infrastructure層のリファクタリング
- [ ] Task: `src/infra/repository/` 内の実装クラスにおいて、汎用名をドメイン用語へ変更する。
- [ ] Task: `src/infra/conpty.rs`, `editor.rs`, `input.rs` 内の命名を修正する。
- [ ] Task: 統合テストまたは手動実行により、ConPTY/Editor操作の正常性を確認する。
- [ ] Task: 変更内容のコミット
- [ ] Task: Conductor - ユーザー手動検証 'Phase 4: Infrastructure層' (Protocol in workflow.md)

## Phase 5: GUI層のリファクタリング
- [ ] Task: `src/gui/window/` 配下（mod.rs, handlers.rs）で多用されている `data` 変数を `window_data` 等へリネームする。
- [ ] Task: `renderer.rs`, `ime.rs`, `scroll.rs` 内の汎用変数・非述語booleanを修正する。
- [ ] Task: ビルドし、EmEditor上での描画、IME入力、スクロールが正常に動作することを確認する。
- [ ] Task: 変更内容のコミット
- [ ] Task: Conductor - ユーザー手動検証 'Phase 5: GUI層' (Protocol in workflow.md)

## Phase 6: 最終確認とクリーンアップ
- [ ] Task: プロジェクト全体を `grep` し、汎用的な名称（data, buf, tmp）の「意図しない残り」がないか最終監査する。
- [ ] Task: `cargo clippy` を実行し、命名に関連する警告がないことを確認する。
- [ ] Task: 最終成果物のコミット
- [ ] Task: Conductor - ユーザー手動検証 'Phase 6: 最終確認' (Protocol in workflow.md)
