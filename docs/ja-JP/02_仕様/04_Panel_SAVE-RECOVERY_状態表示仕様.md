<!--
HLDocS:LLM-MANAGED
doc_id: doc-20261125-103800Z-SR03
lang: ja-JP
canonical_title: Panel SAVE-RECOVERY 状態表示仕様
document_type: spec
canonical_document: true
transport: [true_out]
-->

[目次](../目次.md) > [仕様目次](仕様目次.md) > Panel > SAVE-RECOVERY 状態表示仕様

# Panel SAVE-RECOVERY 状態表示仕様

## 1. Purpose（存在理由）

Resourceの検証状態と診断情報をProject Panelに一貫して表示し、利用者が検証の進行、異常および復旧を識別できる契約を定義する。非表示のUIに対する不要な表示処理を抑制する。

## 2. Non-goals（対象外）

Resourceの利用可能判定、検証実行制御、保存処理および監視機構は本仕様の対象外とする。具体的なアイコン図柄、詳細表示UIの形式、公開APIの型および実装方式は本仕様で固定しない。

## 3. 責務境界
<!-- hldocs:sec_id=sec_3181fc1be6108411821f5380 -->

- Resource Registryおよび検証状態の正本はResource管理側に属する。
- Project Panelは現在のセッションに対応する状態を表示する。表示上の状態によってResource Statusを変更してはならない。
- Resource IDとProject Panel Node IDを区別する。同一Resourceを参照する複数ノードは同じResource状態を反映する。
- 汎用Tree Viewerはアプリ固有のResource検証規則を所有しない。
- Resourceを参照しない論理ノードも子孫の表示状態を反映できなければならない。

## 4. 状態表示

### 4.1 表示状態と優先順位
<!-- hldocs:sec_id=sec_2e5bdc4397043b5638892b33 -->

各ノードの検証・診断状態を表すアイコンは最大1個とする。通常のノード種別アイコンとは区別する。

複数の表示状態が成立する場合、次の優先順位を適用する。

| 優先順位 | 表示状態 | 意味 |
|---|---|---|
| 1 | Validating | 検証実行中 |
| 2 | Error | 現在有効なエラー |
| 3 | Warning | 現在有効な警告 |
| 4 | Pending | 検証待機中 |
| 5 | Normal | 上記の表示対象なし |

NormalはResourceの検証成功を保証する表示ではない。Cancelledは内部状態であり、独立した状態アイコンを表示しない。

### 4.2 再検証
<!-- hldocs:sec_id=sec_a06577628274dbec3bf1d6a2 -->

- 同一Resourceの再検証待機中はPending、実行中はValidatingを表示対象とする。
- 再検証中に内部保持されている旧診断を、当該Resourceの現在のエラーまたは警告として表示してはならない。
- 検証中に対象が変更され、最新世代の再検証が必要となった場合は、再検証が継続する間のValidating表示を維持する。
- 最新の検証結果が確定した場合は、その結果に対応する表示状態へ更新する。
- 古いセッションまたは世代の検証結果を表示へ反映してはならない。

### 4.3 親ノードへの伝播
<!-- hldocs:sec_id=sec_789dbce32534508ad80d10ee -->

親ノードの表示状態は自身と子孫の表示対象状態のうち、最も優先順位の高い状態とする。

子孫にValidatingが存在する間は、他の子孫にErrorが存在しても親の表示はValidatingとなる。検証状態の変更または子孫の追加・削除後は、現在の状態に基づく優先順位を反映しなければならない。

親への伝播は表示上の集約であり、親Resource自身の利用可能性や診断情報を変更してはならない。

## 5. 表示形式と診断
<!-- hldocs:sec_id=sec_3f3b2419889615f304361598 -->

- Framework標準の状態アイコンを用いる。通常の状態表示はアイコンのみとし、状態説明の文字列を常時追加しない。
- アイコンは現在のThemeに適合し、状態を識別できなければならない。
- Pendingは静的表示とする。
- Validatingはアニメーション表示とし、表示中の対象間で共通の位相を使用する。
- 利用者の明示操作によって診断内容を確認できなければならない。具体的な操作形式は本仕様で固定しない。
- 診断メッセージは既存Localization機構に従う。安定した診断コードと利用者向け表示文字列を区別する。
- 現在の診断と履歴ログを混同してはならない。

## 6. Visibilityと表示処理
<!-- hldocs:sec_id=sec_2500060a250655fa87951bd7 -->

- 非表示のPanelではアニメーション、表示専用の再構築等の表示専用処理を停止する。
- Panelの非表示を理由としてResource検証やRegistryの状態更新を停止してはならない。
- 再表示時は現在の正本状態を反映する。非表示期間中の表示変更を順番に再生する必要はない。

## 7. UI更新契約
<!-- hldocs:sec_id=sec_4f4835f994a4a8311596e8c1 -->

- 表示更新の必要性はコンポーネント単位のDirty Flagとして管理する。Project保存用Dirtyとは区別する。
- 当該コンポーネントの表示に関係する状態変更を更新要求として保持する。
- 非表示中は更新要求を保持し、表示専用の再構築を行わない。
- Dirty Flagは対象の表示更新が成功した場合にのみ解除できる。
- 更新処理と並行して発生した新たな変更をDirty Flagの解除によって失ってはならない。
- 既存の状態通知・更新要求機構を利用し、中央集約型の通知ルーティング機構を新設しない。

## 8. 関連仕様

- [Resource SAVE-RECOVERY 復旧検証仕様](05_Resource_SAVE-RECOVERY_復旧検証仕様.md)
- [Application / Project SAVE-RECOVERY 仕様](02_Application_Project_SAVE-RECOVERY_仕様.md)
- [REG-001 Framework管理Resource Registry 再設計仕様](REG-001_Framework管理ResourceRegistry_再設計.md)
- [Step 5 Standard Panels 仕様](10_Step5_Standard_Panels_仕様.md)

---

[目次](../目次.md) > [仕様目次](仕様目次.md) > Panel > SAVE-RECOVERY 状態表示仕様
