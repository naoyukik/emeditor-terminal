# Product Guidelines - emeditor-terminal

## Design Principles
- **EmEditor Native Integration**: プラグインの UI は EmEditor 本体のスタイル（配色、フォント、コンポーネント）に完全に準拠し、エディタの一機能としてシームレスに統合される。
- **Performance First**: 入出力의 レスポンス速度を最優先する。大量のログ出力時もエディタの操作性を損なわない非同期処理を徹底する。

## Behavioral Guidelines
- **Robust Isolation**: ターミナルプロセスをエディタ本体から分離し、子プロセスの異常終了がエディタに影響を与えない設計とする。
- **State Persistence**: セッションの中断や再起動が発生しても、カレントディレクトリや実行コンテキストを可能な限り復元し、作業の継続性を確保する。

## AI Interaction Guidelines
- **Change Visualization**: AI エージェントによるファイル操作が発生した際、ターミナル上の出力とエディタ上の変更箇所を視覚的に結びつけ、ユーザーが即座に変更を把握できるようにする。
