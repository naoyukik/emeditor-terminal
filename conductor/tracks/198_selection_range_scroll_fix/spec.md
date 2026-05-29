# 仕様書 (Specification): selection_range の選択開始点がスクロールで移動する不具合修正 (Issue 198)

## 概要 (Overview)
選択範囲を保持したままスクロールすると、選択開始点が画面上で移動して見える不具合を修正する。原因は `selection_range` を画面座標（visual row）で保持している点にあるため、scrollback history を含む論理座標（logical row）で保持する設計へ変更する。

## 機能要件 (Functional Requirements)
1. **選択範囲の論理座標化**
   - `selection_range` は画面座標ではなく論理座標（history を含む行座標）で保持する。
2. **スクロール耐性の保証**
   - `scroll_lines` / `scroll_to` / マウスホイール操作後も、選択開始点・終了点の論理位置が不変であること。
3. **選択描画の整合**
   - 描画時は visual row から logical row へ解決して選択判定を行い、表示上の選択範囲を一貫させる。
4. **コピー結果の整合**
   - 選択テキスト取得は論理座標モデルに合わせ、history-screen 跨ぎでも期待どおりの文字列を返す。

## 非機能要件 (Non-Functional Requirements)
- 既存の Win32 イベント経路（`WM_MOUSEWHEEL` / `WM_VSCROLL`）は変更しない。
- EmEditor custom bar の統合仕様に影響を与えない。
- 既存アーキテクチャ（Strict Rigid）を維持し、責務境界を崩さない。

## 受け入れ条件 (Acceptance Criteria)
- [ ] 選択後に上下スクロールしても選択開始点が移動しない。
- [ ] 逆方向スクロールでも選択範囲が崩れない。
- [ ] history-screen 跨ぎの選択でコピー結果が表示と一致する。
- [ ] 逆方向ドラッグを含む選択でも開始/終了正規化が正しく動作する。
- [ ] 関連ユニットテストが追加・更新され、回帰がない。

## Out of Scope
- 矩形選択や複数選択など新機能追加。
- マウス入力モデル全体の再設計。
- #187 相当の広範なバッファ責務再分割。
