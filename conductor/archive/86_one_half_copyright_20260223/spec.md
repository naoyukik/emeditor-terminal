# Specification - Track 86: feat: one halfの著作権表記を追加

## 1. Overview
本トラックでは、プロジェクトで使用されているカラーテーマ「One Half」の著作権表記を `README.md` に追加します。これにより、ライセンスコンプライアンスを遵守し、作者への適切なクレジットを提供します。

## 2. Functional Requirements
- `README.md` の末尾に「One Half」テーマの著作権およびライセンス情報を追加する。
- 追加するテキストは、Issue #86 で指定された以下の3行形式とする：
  ```
  Copyright (c) 2019 Son A. Pham 
  Released under the MIT license
  https://opensource.org/licenses/mit-license.php
  ```

## 3. Non-Functional Requirements
- 既存の `README.md` の内容を損なわないこと。
- 日本語ドキュメントとしての整合性を保つこと。

## 4. Acceptance Criteria
- [ ] `README.md` に指定された著作権表記が含まれている。
- [ ] 表記の場所が `README.md` の末尾である。
- [ ] テキストの内容が Issue #86 の指示と完全に一致している。

## 5. Out of Scope
- `LICENSE` ファイルへの追記（ユーザー指示により対象外）。
- プログラムコード（起動メッセージやダイアログ）への表示。
- 他のカラーテーマの著作権表記追加。
