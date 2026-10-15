# Project Panel Hybrid 実装前テスト仕様

対象: Meridian合意済み Project Panel Public API（`docs/ja-JP/09_検討資料/ProjectPanel_Public_API_Meridian確認案.md`）。本実装前にケースを確定する。

| ID | 入力・操作 | 期待結果 |
|---|---|---|
| PP-HYB-001 | アプリ提供カテゴリ→Robot | 指定階層で表示 |
| PP-HYB-002 | Robotに登録済みURDFとMJCFを紐づける | 両ファイルが子として表示 |
| PP-HYB-003 | ファイルを持たないSysID | 論理リソースとして表示 |
| PP-HYB-004 | 1つのRegistry IDを複数ノードで参照 | ノードIDは別、Registry IDは共通 |
| PP-HYB-005 | 同じ表示名の論理ノードとファイル | 種別とIDが衝突しない |
| PP-HYB-006 | Missing / RootUnavailable | IDと参照を保持し、状態を区別 |
| PP-HYB-007 | Robotの表示名を変更 | 論理リソースIDとノードIDは不変 |
| PP-HYB-008 | Save/Open後にProvider再取得 | 永続IDと関連ファイルを復元 |
| PP-HYB-009 | 未登録ファイルがプロジェクト内に存在 | 自動的に登録済みノードへ変換しない |
| PP-HYB-010 | アプリ提供ツリーとFW設定 | FW設定は独立ルート |
| PP-HYB-011 | 同一Projectで連続invalidate | 取得要求を集約 |
| PP-HYB-012 | Project Close/Open/Save As | 古いProjectの通知・選択状態を引き継がない |
| PP-HYB-013 | Selected / Activated / ContextAction | ノードIDをアプリに通知、複数選択を保持 |
| PP-HYB-014 | ActionResult::NotHandled | FW対応の既定操作にフォールバック |
| PP-HYB-015 | 存在しないRegistry IDの子ノード | 別ファイルに付け替えず診断可能 |
| PP-HYB-016 | 論理リソース削除 | ファイル本体を暗黙削除しない |
| PP-HYB-017 | 既存ProjectPanelModel利用 | 既存のTreeModel変換結果を維持 |

## 検証レベル

- モデル単体: PP-HYB-001〜005, 007, 009〜011, 015, 017
- UI操作: PP-HYB-013, 014
- Registry / Resource Operation連携: PP-HYB-006, 015, 016
- Project lifecycle統合: PP-HYB-008, 012

## 受け入れ条件

Windows/Linux/macOSで`cargo test --all-targets --all-features`を成功させる。GUI操作は自動テストと別に実機検証の状態を記録する。未実装の項目を合格扱いしない。
