# Implementation Plan: Refactor Layered Architecture Boundaries

## Phase 1: Domain Layer Refactor
- [ ] Task: `src/domain/input.rs` の意図をドキュメント化する
    - [ ] Action: VKコード定数に、なぜ手動で定義しているか（`windows` クレート依存を避けるため）の説明コメントを追加する。
- [ ] Task: Domainモジュールの可視性をレビューする
    - [ ] Action: `src/domain/terminal.rs`、`src/domain/parser.rs` などのモジュールを確認する。
    - [ ] Action: クレート内でのみ必要な可視性を `pub` から `pub(crate)` に変更する。
- [ ] Task: ビルドを確認する
    - [ ] Command: `cargo check`
- [ ] Task: Phase 1 の変更をコミットする
    - [ ] Command: `git add src/domain/`
    - [ ] Command: `git commit -m "refactor(domain): Enhance visibility control and document intent"`
- [ ] Task: Conductor - User Manual Verification 'Domain Layer Refactor' (Protocol in workflow.md)

## Phase 2: Application Layer Refactor
- [ ] Task: `src/application/service.rs` の可視性をレビューする
    - [ ] Action: `TerminalService` 構造体とそのメソッドを確認する。
    - [ ] Action: パブリックインターフェースで `windows` クレートの型が公開されていないことを確認する。
    - [ ] Action: 不要な `pub` を `pub(crate)` に変更する。
- [ ] Task: ビルドを確認する
    - [ ] Command: `cargo check`
- [ ] Task: Phase 2 の変更をコミットする
    - [ ] Command: `git add src/application/`
    - [ ] Command: `git commit -m "refactor(app): Enforce strict visibility in TerminalService"`
- [ ] Task: Conductor - User Manual Verification 'Application Layer Refactor' (Protocol in workflow.md)

## Phase 3: Final Verification
- [ ] Task: ユニットテストを実行する
    - [ ] Command: `cargo test`
- [ ] Task: Linterを実行する
    - [ ] Command: `cargo clippy`
- [ ] Task: 最終確認をコミットする（変更がある場合）
    - [ ] Command: `git add .` (clippy修正がある場合のみ)
    - [ ] Command: `git commit -m "chore: Apply clippy fixes"` (optional)
- [ ] Task: Conductor - User Manual Verification 'Final Verification' (Protocol in workflow.md)
