# 実装計画: ターミナル配色のテーマ対応および改善 (Issue #38)

## Phase 0: SDKを通じた構成保存方法の調査
- [x] Task: `sdk/` 以下の調査
    - [x] EmEditorプラグインとして設定（Themeや各種config等）を永続化するための標準的な手段（レジストリ、専用API、iniファイルなど）を調査する。
    - [x] EmEditorのワークスペース機能（`.code-workspace`）に対応するAPIが提供されているか調査する。
    - [x] `sdk/` 配下のヘッダファイルやサンプルコードを確認する。
    - [x] 調査結果をこのトラックのドキュメント（必要であれば新ファイル）にまとめる。
- [x] Task: 構成保存方針の決定
    - [x] 調査結果に基づき、本トラックで実装する「構成情報のロード処理（Phase 2）」の設計方針（ハードコード、独自ファイル、またはEmEditor標準設定APIの利用）を決定する。
- [x] Task: Conductor - ユーザー手動検証 'Phase 0: SDKを通じた構成保存方法の調査' (Protocol in workflow.md)
- [x] Task: Commit Phase 0 changes

## Phase 1: テーマモデルと設定の定義 (Domain Layer)
- [ ] Task: テーマ情報のドメインエンティティの作成
    - [ ] `src/domain/model/color_theme.rs`（または類似のDDDに基づく命名）を新規作成する。
    - [ ] `ColorTheme` 構造体を定義し、背景色、文字色、16色パレットを保持させる。
    - [ ] RGB値を表す `Color` バリューオブジェクトを定義、または既存のものを利用する。
- [ ] Task: Solarized テーマの実装
    - [ ] `ColorTheme::solarized_dark()` などのファクトリメソッドを実装し、パレットのRGB値を定義する。
    - [ ] `ColorTheme::solarized_light()` の実装を追加する。
    - [ ] `ColorTheme::default()` の実装を追加する（既存の配色または標準ANSIカラー）。
- [ ] Task: 構成読み込みのための設定インターフェース定義
    - [ ] ターミナルの設定を表すエンティティ（例: `TerminalConfig`）を定義し、選択されているテーマの種類を保持できるようにする。
- [ ] Task: Domain層のユニットテスト追加
    - [ ] 各テーマが期待する色情報を保持しているか検証するテストを追加する。
- [ ] Task: Conductor - ユーザー手動検証 'Phase 1: テーマモデルと設定の定義 (Domain Layer)' (Protocol in workflow.md)
- [ ] Task: Commit Phase 1 changes

## Phase 2: 設定のロード処理と Application 層の適応
- [ ] Task: 構成情報のロード処理 (Infrastructure/Application Layer)
    - [ ] 設定ファイルから `TerminalConfig` を読み込むロジックを実装する。今回は単純なハードコード設定、または簡易的な設定ファイルでの切り替えとする。
    - [ ] 起動時に構成情報を読み込み、適切な `ColorTheme` を生成・取得する処理を実装する。
- [ ] Task: Service/Workflow の改修
    - [ ] `TerminalService` 等の初期化時に `ColorTheme` または `TerminalConfig` の情報を受け渡せるようコンストラクタ等を修正する。
- [ ] Task: Application層のユニットテスト修正
    - [ ] 追加された引数に対するモック・スタブ対応を行う。
- [ ] Task: Conductor - ユーザー手動検証 'Phase 2: 設定のロード処理と Application 層の適応' (Protocol in workflow.md)
- [ ] Task: Commit Phase 2 changes

## Phase 3: GUI/描画処理へのテーマ適応
- [ ] Task: GDI描画ロジックの改修
    - [ ] `src/gui/driver/` 内の描画処理（背景クリア、文字描画）において、ハードコードされている色を `ColorTheme` の背景色・文字色に置き換える。
    - [ ] SGRシーケンス（30-37, 40-47等）からRGB値へのマッピング処理を、`ColorTheme` の16色パレットを参照するように改修する。
- [ ] Task: GUI層のユニットテスト修正
    - [ ] SGRシーケンスをパースした結果が、テーマのパレットと一致することを検証するテストを追加・修正する。
- [ ] Task: Conductor - ユーザー手動検証 'Phase 3: GUI/描画処理へのテーマ適応' (Protocol in workflow.md)
- [ ] Task: Commit Phase 3 changes

## Phase 4: 最終動作確認と調整
- [ ] Task: 実際のEmEditor上での動作検証
    - [ ] プラグインをビルドし、EmEditor上でターミナルを起動して、テーマ（Solarized Dark等）が正しく適用されているか目視確認する。
    - [ ] ターミナル上でlsコマンド（ディレクトリ色等）を実行し、ANSIカラーがパレット通りに描画されるか確認する。
- [ ] Task: テーマ切り替えのテスト
    - [ ] コード上の設定値を変更して再ビルドし、別のテーマ（Solarized Light等）が適用されることを確認する。
- [ ] Task: 不要なコードの削除とクリーンアップ
    - [ ] 古いハードコードされた色の定義や未使用の変数を削除する。
- [ ] Task: Conductor - ユーザー手動検証 'Phase 4: 最終動作確認と調整' (Protocol in workflow.md)
- [ ] Task: Commit Phase 4 changes