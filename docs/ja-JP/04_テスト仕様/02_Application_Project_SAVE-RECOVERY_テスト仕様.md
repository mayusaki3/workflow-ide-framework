<!--
HLDocS:LLM-MANAGED
doc_id: doc-20261125-121400Z-9B4E
lang: ja-JP
canonical_title: Application / Project SAVE-RECOVERY テスト仕様
document_type: testspec
canonical_document: true
transport: [true_out]
-->

[目次](../目次.md) > テスト仕様 > Application / Project SAVE-RECOVERY テスト仕様

# Application / Project SAVE-RECOVERY テスト仕様

単体テストのカバレッジ100%を合格条件とする。各検証単位の列挙条件は個別に検証する。領域間の実連携は結合テストで検証する。

## SAVE-RECOVERY-UT-SPEC-02-001 ルート消失時の状態保持
<!-- hldocs:sec_id=sec_e157eb7f35e664379a4b9553 -->
<!-- hldocs:ref=doc-20261124-020300Z-SR02#sec_e157eb7f35e664379a4b9553 -->

### 前提条件

編集中のProjectが存在する。

### 検証内容

RootMissingを通知し、ルート再出現、明示的な監視再設定、Save As要求を個別に評価する。

### 期待結果

Closeせずmetadata、Framework状態、Registry、Application状態、未保存変更を保持する。通常Resource通知と区別し、再出現のみでは監視を再開せず、明示操作で再設定できる。RootMissingのみでSave Asを拒否しない。

## SAVE-RECOVERY-UT-SPEC-02-002 復旧対象の判定
<!-- hldocs:sec_id=sec_f8ef31fd51c296a4f223131b -->
<!-- hldocs:ref=doc-20261124-020300Z-SR02#sec_f8ef31fd51c296a4f223131b -->

### 前提条件

メモリ上の編集状態、アクセス可能なファイル、取得不能なファイルを含むProject。

### 検証内容

各対象の復旧可能性とApplicationProjectAdapterへの責務分担を評価する。

### 期待結果

メモリ状態とアクセス可能なファイルを保存対象とし、Resource IDと参照を維持する。Application固有データはAdapterが扱い、Frameworkは内部形式を解釈しない。取得不能内容を明示し、空データで暗黙に置換しない。

## SAVE-RECOVERY-UT-SPEC-02-003 欠落の通知と同意
<!-- hldocs:sec_id=sec_4e4603724a0a2d49b3209a73 -->
<!-- hldocs:ref=doc-20261124-020300Z-SR02#sec_4e4603724a0a2d49b3209a73 -->

### 前提条件

復旧不能な対象が存在する。

### 検証内容

欠落対象と影響の通知、明示同意、同意なし、取消、同意後の対象外欠落、保存処理失敗を個別に評価する。

### 期待結果

欠落と影響を識別でき、明示同意なしでは部分保存を確定しない。同意対象外の欠落を暗黙に許容せず、同意済み欠落と保存失敗を区別する。

## SAVE-RECOVERY-UT-SPEC-02-004 保存世代と確定境界
<!-- hldocs:sec_id=sec_325de8e019361e4562e94098 -->
<!-- hldocs:ref=doc-20261124-020300Z-SR02#sec_325de8e019361e4562e94098 -->

### 前提条件

新旧ルート、保存対象、Dirty状態が存在する。

### 検証内容

保存対象の全成功、個別保存失敗、世代不整合、確定前の状態参照、中止を個別に評価する。

### 期待結果

合意対象の保存成立と世代整合時のみ確定する。確定前にルートを切り替えず、元保存内容と論理参照を維持し、未確定先を正常Projectと扱わない。中止・失敗時はメモリ状態とDirtyを維持し、成功時は成立範囲のみDirty解除する。欠落を完全復旧と扱わない。MissingのみではDirtyを維持せず、未保存のメモリ編集に対応するDirtyは維持する。

## SAVE-RECOVERY-UT-SPEC-02-005 保存結果とlifecycle通知
<!-- hldocs:sec_id=sec_fb9237d862d88c0681d0d45a -->
<!-- hldocs:ref=doc-20261124-020300Z-SR02#sec_fb9237d862d88c0681d0d45a -->

### 前提条件

Save As要求が存在する。

### 検証内容

完全保存、同意済み欠落保存、同意なし、取消、保存失敗を個別に評価する。

### 期待結果

完全保存と欠落保存を区別して新ルートへ切り替える。同意なし・取消・失敗は未確定で旧ルートを維持する。両保存成立をSave As成功とし、復旧の完全性は保存結果で区別する。欠落参照を保持し、両保存成立で既存SavedAsを通知する。専用イベントは追加せず、同意なし・取消・失敗ではSavedAsを通知しない。

## SAVE-RECOVERY-UT-SPEC-02-006 新ルート参照とMissing維持
<!-- hldocs:sec_id=sec_b45ec7593713bcc2c4590443 -->
<!-- hldocs:ref=doc-20261124-020300Z-SR02#sec_b45ec7593713bcc2c4590443 -->

### 前提条件

Missing Resourceと相対パスを保持するProject。

### 検証内容

Save As成立後の参照解決と検証前の状態を評価する。

### 期待結果

新ルートの相対パスを基準とし旧絶対パスを固定しない。保存成立だけではAvailableに変更しない。

---

[目次](../目次.md) > テスト仕様 > Application / Project SAVE-RECOVERY テスト仕様
