# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/)
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.6.0] - 2026-04-18
### Added
- 依存関係の最新化: `windows-rs` を 0.62 (Release 73) へアップデートし、基盤を刷新
- 新規クレート導入: `windows-core`, `windows-registry`, `windows-result`, `windows-strings`, `windows-link` を採用し、型安全性とリンクの標準化を向上
- 物理的隔離の完遂: Presentation層 (Resolver) から Win32 型を完全に排除し、`WindowId` による抽象化を実現
- プロトコル解釈の分離: `TerminalProtocolHandler` を新設し、Domain層から外部パーサー依存を隔離
- 大規模ファイル分割: 肥大化した `TerminalBufferEntity` を `TerminalGridEntity` と `TerminalScrollbackEntity` に分割し凝集度を向上
- 安全性ドキュメント: 全ての `unsafe` ブロックに対し、RAII やスレッド安全性に関する `// SAFETY:` コメントを網羅

### Changed
- レジストリ操作を `windows-registry` クレートによる Safe API 実装へ移行
- `DllMain` からの重い初期化処理を排除し、Loader Lock によるデッドロックのリスクを根絶
- ドメイン層のユニットテストをリファクタリング後の構造に合わせて復元・拡充

### Fixed
- TUI 描画崩れ: リファクタリング中に欠落した ANSI シーケンス（ECH, CHA, CNL等）を復元
- IME 候補位置: 仮想カーソル移動に伴う IME 候補ウィンドウの表示位置同期を修正
- リソース解放: `WM_DESTROY` ハンドラでのフック解除およびターミナルリソースの解放を確実に実行するよう修正
- 安定性向上: `wnd_proc` のメッセージ引数不整合によるスタック破壊およびクラッシュを修正

## [0.5.0] - 2026-04-17
### Added
- DECSCUSR（カーソル形状変更）サポートを追加
- 書記素クラスター対応により、結合文字やワイド文字の描画・カーソル移動を改善
- 設定基盤を実装し、設定ダイアログでの各種設定編集を追加
- フォント書体・サイズ・スタイル（太字/斜体）の外部化と描画反映を追加
- カラーテーマ設定の外部化とファイルロード、システムテーマ連動を追加
- プラグイン終了時の設定保存処理を追加
- 起動時にターミナルへ自動フォーカスする機能を追加

### Changed
- ThemeType の変換・表示ロジックをドメイン層へ集約
- WindowGuiDriver 導入など、GUI層の責務分離を強化
- システムキャレット同期を再設計し、IME連携と表示安定性を向上

### Fixed
- ターミナル二重起動時のクラッシュを修正
- 起動直後に右端・下部がはみ出すサイズ計算の不整合を修正
- IME候補ウィンドウと未確定文字の表示位置同期を改善

## [0.4.0] - 2026-02-26
### Added
- GDIダブルバッファリングを導入し、描画最適化とフリッカー防止を実装
- vteパースエンジンへ完全換装し、ワークスペースポリシーを導入

### Changed
- vte導入に向けてDomain層インターフェースを標準化し、パーサーを刷新

### Fixed
- `edit` 等のTUIアプリで発生していた表示ズレとクラッシュを修正

## [0.3.0] - 2026-02-23
### Added
- 起動ダイアログを廃止し `pwsh.exe` を直接起動する方式へ変更
- IME候補ウィンドウ位置の同期とインライン入力を実装
- 追加カーソル移動シーケンス（CHA, VPA, CNL, CPL）をサポート
- Microsoft EditでのTUI描画問題を修正
- AltキーによるTUIメニュー操作をサポート
- カーソル位置の保存/復元（DECSC/DECRC）を実装
- 拡張SGRシーケンスと属性描画（256色/TrueColor/スタイル）をサポート
- スクロールバック・バッファを実装
- ターミナル配色のテーマ対応と視認性改善を実装

### Changed
- レイヤードアーキテクチャの導入とディレクトリ再構築を実施
- キーボード入力ロジックをDomain/Infra層へ抽出し安全性を向上
- ドメインモデルの物理的分離とカレントディレクトリ修正を実施
- IMEハンドリングを `custom_bar.rs` から抽出
- スクロールロジックを `ScrollManager` へ抽出
- ウィンドウプロシージャの整理とディスパッチ効率化を実施
- RepositoryパターンとDIを導入
- レイヤードアーキテクチャ境界の強化と可視性の適正化を実施
- DDDに基づく命名規則を適用
- Suffix Ruleに基づく物理構成の再編とGit運用の強化を実施
- コーディング規約および運用ガイドラインを導入
- AIエージェント統制強化とドキュメント合理化を実施

### Fixed
- clippy警告（lints）を修正
- PSReadLineの予測入力（Prediction）表示を改善

## [0.2.0] - 2026-01-20
### Added
- EmEditorカスタムバーによるConPTYターミナル実装（Phase 1-4）
- ConPTYカーソル描画とグリッド整列の精度を改善

## [0.1.0] - 2026-01-17
### Added
- RustによるEmEditorプラグイン開発基盤を構築し、Hello World動作を確認
- 簡易ターミナル入出力機能を実装し、基本動作を検証

[Unreleased]: https://github.com/naoyukik/emeditor-terminal/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/naoyukik/emeditor-terminal/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/naoyukik/emeditor-terminal/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/naoyukik/emeditor-terminal/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/naoyukik/emeditor-terminal/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/naoyukik/emeditor-terminal/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/naoyukik/emeditor-terminal/releases/tag/v0.1.0
