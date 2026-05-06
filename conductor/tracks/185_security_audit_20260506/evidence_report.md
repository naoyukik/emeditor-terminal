# Security Audit Evidence Report (Track 185)

## 1. 調査概要 (Discovery Summary)
Gemini CLI (`gemini-cli-security`) および手動の静的解析を用いて、プロジェクトの依存関係とソースコードのセキュリティ監査を実施した。

## 2. 脆弱性診断結果 (Findings)

### 2.1 依存関係 (pnpm-lock.yaml)
| ID | 重要度 | パッケージ | 内容 | 修正案 |
| :--- | :--- | :--- | :--- | :--- |
| GHSA-869p-cjfg-cm3x | High (7.5) | `jws@3.2.2` | HMAC署名の不適切な検証 | `3.2.3` 以上に更新 |
| GHSA-c2c7-rcm5-vvqj | High (7.5) | `picomatch@2.3.1` | ReDoS脆弱性 | `2.3.2` 以上に更新 |
| GHSA-6rw7-vpxm-498p | Medium (6.3) | `qs@6.14.0` | メモリ枯渇による DoS | `6.14.1` 以上に更新 |
| GHSA-3v7f-55p6-f55p | Medium (5.3) | `picomatch@2.3.1` | メソッドインジェクション | `2.3.2` 以上に更新 |
| GHSA-w7fw-mjwx-w883 | Low (3.7) | `qs@6.14.0` | DoS (カンマパース) | `6.14.2` 以上に更新 |

### 2.2 ソースコード (SAST/Manual Review)
- **Safety Comment の欠落**: 全 101 箇所の `unsafe` ブロックのうち、約 35 箇所で規約違反のコメント欠落を確認。
  - 主な場所: `src/gui/driver/terminal_gui_driver.rs`, `src/gui/window/mod.rs` 等。
- **Win32 API ハンドル管理**: `WindowGuiDriver` により概ね隔離されているが、`unsafe` ブロック内でのハンドル有効性チェックの徹底が必要。

## 3. 修正方針 (Remediation Plan)

### フェーズ 2: 修正タスク
1.  **依存関係の更新**: `pnpm` を用いて脆弱性が修正されたバージョンにアップグレードする。
2.  **Safety Comment の補完**: コメントが欠落している `unsafe` ブロックに対し、安全性根拠（Safety Comment）を追記する。
3.  **unsafe 境界の再検証**: 特に文字列バッファやハンドル操作を伴う `unsafe` ブロックにおいて、オーバーフローや NULL 参照のガードが適切か再点検する。

## 4. 期待される挙動 (Expected Behavior)
- `/security:analyze` (OSV-Scanner) の再実行時に脆弱性が検出されない。
- すべての `unsafe` ブロックに規約に沿った Safety Comment が存在する。
- 修正後も `install.ps1` による実機ビルド・動作に支障がない。
