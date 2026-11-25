<!--
HLDocS:LLM-MANAGED
doc_id: doc-20261125-104500Z-8F2C
lang: ja-JP
canonical_title: Resource SAVE-RECOVERY テスト仕様
document_type: testspec
canonical_document: true
transport: [true_out]
-->

[目次](../目次.md) > テスト仕様 > Resource SAVE-RECOVERY テスト仕様

# Resource SAVE-RECOVERY テスト仕様

単体テストのカバレッジ100%を合格条件とする。実装固有の追加検証は実装設計確定後に補完する。各ケース内の列挙条件は個別に検証する。

## SAVE-RECOVERY-UT-SPEC-01-001 状態分離
<!-- hldocs:sec_id=sec_e4390c271897978f0bfd5181 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_e4390c271897978f0bfd5181 -->

### 前提条件

MissingのResource。

### 検証内容

待機・検証開始・成功・失敗をそれぞれ観測。

### 期待結果

成功確定前はMissingを維持し、成功時のみAvailableとなる。

## SAVE-RECOVERY-UT-SPEC-01-002 検証状態と永続化
<!-- hldocs:sec_id=sec_5486c895f4a8fe10934fdf0b -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_5486c895f4a8fe10934fdf0b -->

### 前提条件

検証対象が登録済み。

### 検証内容

未検証・待機・実行・成功・失敗・取消の各状態を観測し保存データを確認。

### 期待結果

各状態を区別でき、Validation Statusは永続化されず、取消だけでは消失・失敗と判定しない。

## SAVE-RECOVERY-UT-SPEC-01-003 識別情報の分離
<!-- hldocs:sec_id=sec_6f54e0f13d044fb7a564ed72 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_6f54e0f13d044fb7a564ed72 -->

### 前提条件

複数Resourceと複数セッション。

### 検証内容

対象変更と再Openを比較。

### 期待結果

Resource ID、Session ID、世代、対象情報を区別できる。

## SAVE-RECOVERY-UT-SPEC-01-004 参照保持
<!-- hldocs:sec_id=sec_36d084a1dc4de0b1503d1a38 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_36d084a1dc4de0b1503d1a38 -->

### 前提条件

登録済みProject Resource。

### 検証内容

外部削除、ルート変更、同一相対パス再出現、ルート消失、再Openを個別に評価。

### 期待結果

IDと参照を維持し旧絶対パスに依存せず、検証前の利用を許可せず、監視異常と個別結果を区別する。

## SAVE-RECOVERY-UT-SPEC-01-005 共通検証
<!-- hldocs:sec_id=sec_f2b0b09e44dc08e0ff99d415 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_f2b0b09e44dc08e0ff99d415 -->

### 前提条件

期待種別が定義済み。

### 検証内容

存在・アクセス・種別の全成功と各単独失敗を評価。

### 期待結果

全成功のみ共通検証成功となる。

## SAVE-RECOVERY-UT-SPEC-01-006 必須・任意検証
<!-- hldocs:sec_id=sec_5008d87cdac12a1c65f674a9 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_5008d87cdac12a1c65f674a9 -->

### 前提条件

共通検証成功または依存検証失敗。

### 検証内容

固有検証なし、Required成功・失敗・Skipped、Optional警告、独立した複数失敗を評価。

### 期待結果

Required失敗・Skippedは利用不可、Optional警告のみは利用可。独立診断とSkipped原因IDを保持する。

## SAVE-RECOVERY-UT-SPEC-01-007 利用要求の制御
<!-- hldocs:sec_id=sec_e283e27df679a175849e7347 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_e283e27df679a175849e7347 -->

### 前提条件

未検証Resourceと無関係なResource。

### 検証内容

対象利用要求を出し検証成功・失敗を個別に確定。

### 期待結果

成功前に対象利用を許可せず、失敗時は利用不可、無関係な操作は継続できる。

## SAVE-RECOVERY-UT-SPEC-01-008 実行上限と統合
<!-- hldocs:sec_id=sec_952fc34ec9c82d85d7be6968 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_952fc34ec9c82d85d7be6968 -->

### 前提条件

複数の検証要求。

### 検証内容

3件以上の異なる要求、同一IDの反復要求、多数の異なる待機要求を与える。

### 期待結果

実行数は2以下、同一セッション・IDの待機は1以下、最新世代を保持し固定件数による拒否や周期ポーリングを必要としない。

## SAVE-RECOVERY-UT-SPEC-01-009 優先度・FIFO・昇格
<!-- hldocs:sec_id=sec_eaa877437c9d025cbb1c77eb -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_eaa877437c9d025cbb1c77eb -->

### 前提条件

HighとNormalの待機要求。

### 検証内容

両キューを継続して非空にし開始順を観測。Normal待機中のIDに利用者操作とOS通知を追加。

### 期待結果

同優先度FIFO、High 3件後Normal 1件、昇格はHigh末尾で降格しない。

## SAVE-RECOVERY-UT-SPEC-01-010 旧世代結果の排除
<!-- hldocs:sec_id=sec_6652bb1067ba1d0e173e3e9e -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_6652bb1067ba1d0e173e3e9e -->

### 前提条件

検証実行中。

### 検証内容

対象を複数回変更し旧結果を成功・失敗それぞれで返す。

### 期待結果

旧結果を反映せず最新世代を再検証でき、通常変更で強制中止を必須としない。

## SAVE-RECOVERY-UT-SPEC-01-011 キャンセルと共有上限
<!-- hldocs:sec_id=sec_4ae1105c8ab0f7fb09085d79 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_4ae1105c8ab0f7fb09085d79 -->

### 前提条件

旧セッションに待機・実行要求あり。

### 検証内容

切替・Close・再Open、Resource削除、対象無効化を個別評価し、キャンセル非対応検証器も使用。

### 期待結果

旧待機無効化、協調キャンセル、切替は終了待ちなし、新Session ID、全体実行数2以下、旧結果反映なし。

## SAVE-RECOVERY-UT-SPEC-01-012 確定条件と競合
<!-- hldocs:sec_id=sec_925d45edc497a099bf3ac558 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_925d45edc497a099bf3ac558 -->

### 前提条件

検証結果が返却可能。

### 検証内容

6条件すべて成立する場合と各条件だけ不成立の6通り、確定と状態変更の競合を評価。

### 期待結果

全成立時のみ反映し、競合による誤反映なし。通知は反映後の状態を参照する。

## SAVE-RECOVERY-UT-SPEC-01-013 再Openと再検証
<!-- hldocs:sec_id=sec_0d2d5c75f41a39f64b3a6bca -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_0d2d5c75f41a39f64b3a6bca -->

### 前提条件

Missingとその他のResourceが存在。

### 検証内容

再Open、利用要求、変更通知、再検証結果を観測。

### 期待結果

MissingはNormalで自動検証、その他は利用・変更契機。成功までMissing、旧診断を新結果と混同しない。

## SAVE-RECOVERY-UT-SPEC-01-014 診断の構造と寿命
<!-- hldocs:sec_id=sec_af3dda9a59d04a6f6aaf4899 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_af3dda9a59d04a6f6aaf4899 -->

### 前提条件

Framework・アプリ由来の診断あり。

### 検証内容

診断項目、言語変更、再検証成功・失敗、切替・削除を評価。

### 期待結果

7項目を識別可能、コード安定、メッセージローカライズ可能、由来区別、成功で旧診断消去、失敗で置換、切替・削除で現行表示なし、ログと分離。

## SAVE-RECOVERY-UT-SPEC-01-015 状態スナップショットと計測
<!-- hldocs:sec_id=sec_bd005d88f3d0d2cf0dce0421 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_bd005d88f3d0d2cf0dce0421 -->

### 前提条件

待機・実行・統合・昇格が発生。

### 検証内容

状態更新と参照の競合および各観測項目を評価。

### 期待結果

状態・診断・世代の整合した組を取得でき、pending/running/high/normal/maxWait/coalesced/promotedを観測できる。

---

[目次](../目次.md) > テスト仕様 > Resource SAVE-RECOVERY テスト仕様
