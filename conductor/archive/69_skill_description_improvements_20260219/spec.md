# Specification: Track 69 - AI Agent Control via Skill Descriptions

## 1. Overview
Gemini CLIにおいて、スキルの `description`（メタデータ）はエージェントが適切な知識や規約をロードするためのトリガーである。現在の記述は不十分または日本語主体であり、エージェントが設計指針やプロトコルを無視するリスクがある。本トラックでは、Claudeのベストプラクティスに基づき、これらの記述を英語主体かつ具体的で制約の強い内容に改善し、エージェントの「メタ認知」と統制を強化する。

## 2. Functional Requirements
以下のスキルの `description` を、英語主体かつ具体的なアクション・制約・状況（トリガーキーワード）を含む形式に改善する。

### 2.1 対象スキルと改善ポイント
- **`rust-coding-conventions`**
    - **Keywords:** "Win32 API boundary", "FFI", "unsafe", "Layered architecture", "Dependency direction", "Pure Rust domain", "Resource management", "RAII", "GDI object leakage".
    - **Constraint:** レイヤードアーキテクチャの厳守と、Win32 APIのInfrastructure層への隔離を強調する。
- **`sequential-thinking`**
    - **Keywords:** "Complex problem solving", "Multi-step planning", "Ambiguous requirements", "Before any tool execution", "Verification of hypothesis".
    - **Constraint:** 複雑なタスクの前に必ず「思考」を挟み、論理的なステップを踏むよう誘導する。
- **`naming-conventions`**
    - **Keywords:** "Refactoring", "New file/struct/function creation", "DDD terminology", "Boolean predicate naming", "Snake_case vs PascalCase".
    - **Constraint:** ファイル作成やリファクタリング時に命名規則（`is_xxx`, `snake_case`等）を強制する。
- **`conductor-protocol`**
    - **Keywords:** "Phase transition", "spec.md/plan.md creation", "Track initialization", "User manual verification", "Checkpointing".
    - **Constraint:** 開発フェーズの各ステップ、特に仕様策定と手動検証のプロトコルを厳格に実行させる。
- **`accessing-microsoft-learn-docs`**
    - **Keywords:** "Windows API reference", "Win32 system calls", "Official Microsoft documentation", "Investigation".
    - **Constraint:** Win32 API調査時に公式ドキュメントを参照することを強く推奨する。
- **`referencing-commit-convention` & `operating-git`**
    - **Keywords:** "Commit message format", "Conventional Commits", "Individual file staging", "NO git add .", "NO git add -A".
    - **Constraint:** `git add .` の厳禁と、プロジェクト固有のコミット形式の遵守を絶対条件として記述する。
- **`japanese-response`**
    - **Keywords:** "Japanese communication", "Conductor commands", "Professional tone".
    - **Constraint:** Conductorコマンド使用時や報告時の日本語応答を、英語の description で定義する。

## 3. Non-Functional Requirements
- **Consistency:** すべての `description` で三人称、動名詞（-ing）による能力記述を採用し、トーンを統一する。
- **Clarity:** 100単語程度に最適化し、曖昧さを排除する。
- **Visibility:** Gemini CLIが適切にスキルを選択できるよう、状況に応じたキーワードを配置する。

## 4. Acceptance Criteria
- [ ] 指定されたすべてのスキルの `SKILL.md` (または同等のメタデータファイル) が更新されている。
- [ ] 各スキルの `description` が英語主体であり、指定されたキーワードが含まれている。
- [ ] `README.md` の記述に矛盾がないか再確認されている。
- [ ] （エージェント自身による）更新後の各スキルの認識テストを行い、正しくロードされることを確認する。

## 5. Out of Scope
- README.md への大幅なセクション追加（今回は対象外）。
- スキル自体のロジック（実行コードや詳細な本文）の大規模な改変（`description` の改善が主眼）。
