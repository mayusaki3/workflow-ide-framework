<!--
HLDocS:LLM-MANAGED
doc_id: doc-20261125-232200Z-7C9F
lang: ja-JP
canonical_title: Panel SAVE-RECOVERY テスト仕様
document_type: testspec
canonical_document: true
transport: [true_out]
-->

[目次](../目次.md) > テスト仕様 > Panel SAVE-RECOVERY テスト仕様

# Panel SAVE-RECOVERY テスト仕様

単体テストのカバレッジ100%を合格条件とする。各検証単位の列挙条件は個別に検証する。実際の描画品質と領域間の実連携は単体テストのみで保証せず、結合検証で扱う。

## SAVE-RECOVERY-UT-SPEC-03-001 表示とResource管理の分離
<!-- hldocs:sec_id=sec_3181fc1be6108411821f5380 -->
<!-- hldocs:ref=doc-20261125-103800Z-SR03#sec_3181fc1be6108411821f5380 -->

### 前提条件

複数ノードが同一Resourceを参照し、Resourceを参照しない論理親も存在する。

### 検証内容

現行・旧セッションの状態を与え、表示更新と複数参照を評価する。

### 期待結果

現行状態のみを表示し、同一Resourceの状態を各参照ノードに反映する。Node IDとResource IDを混同せず、表示によってRegistryを変更しない。Tree Viewerは固有検証規則を所有せず、論理親にも状態を集約できる。

## SAVE-RECOVERY-UT-SPEC-03-002 アイコン選択
<!-- hldocs:sec_id=sec_2e5bdc4397043b5638892b33 -->
<!-- hldocs:ref=doc-20261125-103800Z-SR03#sec_2e5bdc4397043b5638892b33 -->

### 前提条件

Validating、Error、Warning、Pending、Normalの表示候補が存在する。

### 検証内容

各単独状態および全組合せを評価する。Cancelledのみ、未検証状態、通常ノード種別アイコンとの共存も評価する。

### 期待結果

Validating > Error > Warning > Pending > Normalの順に最大1個の検証・診断アイコンを選択する。種別アイコンとは区別し、Cancelled独自アイコンを表示せず、Normalを検証成功と解釈しない。

## SAVE-RECOVERY-UT-SPEC-03-003 再検証と旧診断の抑制
<!-- hldocs:sec_id=sec_a06577628274dbec3bf1d6a2 -->
<!-- hldocs:ref=doc-20261125-103800Z-SR03#sec_a06577628274dbec3bf1d6a2 -->

### 前提条件

以前のエラーまたは警告があるResourceが存在する。

### 検証内容

再検証待機、実行、実行中の複数回の対象変更、最新結果の成功・失敗・任意警告、旧世代・旧セッション結果を個別に評価する。Availableを維持した再検証とMissingからの再検証を比較する。

### 期待結果

待機中はPending、実行中はValidatingを表示し旧診断を現行診断として表示しない。対象変更後の継続検証中はValidatingを維持し、最新確定結果のみを反映する。

## SAVE-RECOVERY-UT-SPEC-03-004 階層集約
<!-- hldocs:sec_id=sec_789dbce32534508ad80d10ee -->
<!-- hldocs:ref=doc-20261125-103800Z-SR03#sec_789dbce32534508ad80d10ee -->

### 前提条件

自身の状態を持つ親、論理親、複数階層の子孫が存在する。

### 検証内容

自身と子孫の状態の全優先順位、別子孫のValidatingとErrorおよびPendingとErrorの競合、状態変更、子孫追加・削除を評価する。最高優先状態を持つ子孫の削除、最後の子孫の削除、複数階層への伝播を個別に評価する。

### 期待結果

自身と全子孫の最高優先状態を表示する。Validating終了後および最高優先状態の消失後は残存状態を再集約する。子孫がなくなった場合は自身の状態に従う。親自身のResource Statusと診断を変更しない。

## SAVE-RECOVERY-UT-SPEC-03-005 表示形式・診断・共通位相
<!-- hldocs:sec_id=sec_3f3b2419889615f304361598 -->
<!-- hldocs:ref=doc-20261125-103800Z-SR03#sec_3f3b2419889615f304361598 -->

### 前提条件

複数の表示対象、診断、Themeおよび言語設定が存在する。

### 検証内容

PendingとValidatingの時間変化、異なる時点で検証を開始した複数対象の同一時点の位相、Theme・言語変更、診断確認操作を評価する。現在診断と履歴ログが異なる場合も評価する。

### 期待結果

標準アイコンを使用しThemeに適合する。通常の状態表示はアイコンのみで、状態説明文字列を常時追加しない。Pendingは静的、Validatingは共通位相で動く。診断は明示操作で確認でき、既存Localizationに従いコードと表示文字列を区別する。現在診断と履歴ログを混同しない。

## SAVE-RECOVERY-UT-SPEC-03-006 非表示と再表示
<!-- hldocs:sec_id=sec_2500060a250655fa87951bd7 -->
<!-- hldocs:ref=doc-20261125-103800Z-SR03#sec_2500060a250655fa87951bd7 -->

### 前提条件

表示中のPanelで検証と状態更新が発生している。

### 検証内容

非表示へ切替後に複数の状態変更を発生させ、再表示する。非表示中に検証が開始・完了する場合、再表示時にも検証が継続中の場合、状態変更がない場合を個別に評価する。

### 期待結果

非表示中はアニメーションと表示専用再構築を停止するが、検証とRegistry更新は継続する。再表示時に最新正本状態を反映し、過去の表示変更の再生を必要としない。

## SAVE-RECOVERY-UT-SPEC-03-007 Dirty Flagと競合
<!-- hldocs:sec_id=sec_4f4835f994a4a8311596e8c1 -->
<!-- hldocs:ref=doc-20261125-103800Z-SR03#sec_4f4835f994a4a8311596e8c1 -->

### 前提条件

複数コンポーネントとProject保存用Dirtyが存在する。

### 検証内容

関連・無関係な状態変更、非表示中の複数変更、更新成功・失敗、失敗後の再更新、更新途中の新規変更を個別に評価する。複数コンポーネントのうち一部のみが更新成功する場合も評価する。

### 期待結果

関連コンポーネントの更新要求を保持し、Project保存用Dirtyと区別する。無関係なコンポーネントに不要な更新要求を発生させない。非表示中は再構築せず、更新失敗では解除しない。更新成功したコンポーネントのDirty Flag解除は他のコンポーネントの未処理要求に影響しない。成功時も途中の新規変更を失わない。既存通知機構で更新でき、中央集約型の通知ルーティング機構を新設しない。

---

[目次](../目次.md) > テスト仕様 > Panel SAVE-RECOVERY テスト仕様
