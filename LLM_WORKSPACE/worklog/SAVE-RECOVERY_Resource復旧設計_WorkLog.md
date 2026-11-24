目次 > LLM_WORKSPACE > worklog > SAVE-RECOVERY Resource復旧設計 WorkLog

# SAVE-RECOVERY Resource復旧設計 WorkLog

## 現在状態

- SAVE-RECOVERYの仕様策定段階。実装・テストは未着手。
- 仕様配置はA案で確定。第5章Resourceの中心仕様を先行し、第2章Application / Project、第4章Panel、テスト仕様へ展開する。
- 既存REG-001等は維持し、正式仕様移行時に重複を整理する。
- HLDocSはmainのv0.6.1系を参照。共通仕様14要素とspec用テンプレート・生成プロンプトの取得を確認した。
- SAVE-RECOVERYのWorklogが未作成であったため、本書を新規作成する。

## 決定事項

### 復旧とSave As

- プロジェクトルート消失後もメモリ上のプロジェクト、Registry、アプリ編集状態を保持する。
- Save Asで救出可能な内容を保存する。アクセスできないファイルのみの内容は復元不能として報告する。
- 不完全なSave Asには利用者の明示的な同意が必要。取消・失敗時はメモリ状態を維持し、部分保存を正常完了と扱わない。
- MissingリソースはResource IDと相対パスを保持し、旧ルート絶対パスに依存しない。新ルートに同じ相対パスで再出現した場合、検証成功後に復旧する。

### Resource検証

- Framework共通検証は存在・アクセス・種別。アプリ固有検証は任意に提供でき、必須・任意の区別を持つ。
- 必須検証の失敗またはスキップは復旧を阻止する。任意検証の警告は復旧を阻止しない。
- 非同期ワーカープール上限2。Resource ID単位で重複要求を統合し、旧generationの結果を破棄する。ポーリングしない。
- 同一ファイルが検証中に変更された場合、通常は実行中検証を強制中止せず、旧結果を破棄して最新状態で再検証する。
- 優先度は利用者操作が高、OS通知が通常。同優先度内FIFO。高優先度3件開始後、通常優先度1件の公平性を設ける。
- 通常優先度の保留要求に利用者操作が来た場合、高優先度キュー末尾へ昇格し、降格しない。
- 固定キュー上限は設けず、Resource IDあたり保留要求は最大1件。pending/running/high/normal/maxWait/coalesced/promotedを計測する。
- プロジェクト切替で旧セッション保留要求を無効化し、実行中は協調キャンセルする。切替時に完了待ちしない。再オープンは新セッション。ワーカー上限2は共有。
- 協調キャンセルはセッション切替、Resource削除、対象無効化時。アプリ検証器のキャンセル対応は任意。
- Registry反映はセッション有効・Resource登録・対象有効・generation最新・キャンセルなし・対象同一を確認した同期境界で行い、その後Panelへ通知する。

### 状態・診断・UI

- Resource StatusとValidation Statusは分離する。Missingは検証成功まで維持する。Validation Statusはメモリのみ保持する。
- 再オープン時、Missingは通常優先度で自動検証。それ以外はアクセスまたは変更時に検証する。永続化された状態を検証なしに信頼しない。
- 未検証Resourceを必要とする操作は高優先度検証の成功を待ち、失敗時はエラー。無関係な操作は継続できる。
- 診断情報はresourceId/sessionId/validationGeneration/errorCategory/errorCode/message/source/occurredAtを持つ。安定したコードとローカライズ可能なメッセージを用いる。
- 現在の診断とログを分離する。再検証中は旧診断を内部保持し、成功で消去、失敗で置換、切替・削除で消去する。
- 独立した複数エラーを収集する。実施できない検証は原因ID付きSkippedとして扱う。
- アイコン優先順位はValidating > Error > Warning > Pending > Normal。子孫の状態を親へ伝播する。Cancelは表示しない。
- Pendingは静的、Validatingは共通位相のアニメーション。非表示中は表示処理を停止し、再表示時に最新状態を反映する。
- UI更新はコンポーネント単位Dirty Flagを用い、更新成功時に同時変更を失わないようクリアする。中央集約ルーティングは新設しない。

## 作業記録（2026-11-24）

- 第5章Resource中心仕様を作成済み（93436c58）。
- 仕様目次の第5章へリンクを追加済み（c57c76e）。
- GitHub直接更新時はtrue_outを使用するという利用者指示を確認済み。
- 新規仕様のLLM-MANAGED、先頭・末尾ナビゲーション、相対リンクの存在、目次からの参照を個別確認済み。HLDocS全規約の機械検証は未実施。
- 第2章Application / ProjectのSave As復旧仕様を作成済み（aef9c5c1）。仕様目次へ未実装と明記して追加済み（3685ada7）。
- 次工程は第4章Panelの検証状態表示・UI更新仕様。実装・テストは未着手。

## 未確認事項

- 現行developのResource Registry実体、状態型、Panel構造と設計内容の対応。
- SAVE-RECOVERY仕様の生成前に必要なHLDocS入力、生成条件の最終確認。
- 上記決定事項の既存仕様との重複・矛盾の確認。

## 次工程

1. developの最新仕様・実装関連ファイルを取得し、既存仕様との差分を確認する。
2. 第5章ResourceのSAVE-RECOVERY中心仕様を作成する。（完了）
3. 第2章・第4章の関連仕様、番号付きテスト仕様を順次作成する。
4. HLDocS構造・参照・Traceabilityを検証する。

---

目次 > LLM_WORKSPACE > worklog > SAVE-RECOVERY Resource復旧設計 WorkLog
