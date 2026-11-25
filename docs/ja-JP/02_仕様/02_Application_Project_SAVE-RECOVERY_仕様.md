<!--
HLDocS:LLM-MANAGED
doc_id: doc-20261124-020300Z-SR02
lang: ja-JP
canonical_title: Application / Project SAVE-RECOVERY 仕様
document_type: spec
canonical_document: true
transport: [true_out]
-->

[目次](../目次.md) > [仕様目次](仕様目次.md) > [Application / Project](02_Application_Project.md) > SAVE-RECOVERY

# Application / Project SAVE-RECOVERY 仕様

## 1. Purpose（存在理由）

Project Rootの消失後もCurrent Projectのメモリ状態を維持し、Save Asによって救出可能なデータを新しいProject Rootへ保存できる契約を定義する。

## 2. Non-goals（対象外）

Resource単位の検証、非同期検証の制御、診断状態の表示、監視機構の内部構造は本仕様の対象外とする。

## 3. 基本契約
<!-- hldocs:sec_id=sec_e157eb7f35e664379a4b9553 -->

- Project Rootの消失はProject Closeを意味しない。
- Project metadata、Framework管理状態、Resource Registry、Application管理状態および未保存変更を保持する。
- RootMissingは監視異常として扱い、通常のResource変更通知と区別する。
- ルートの再出現のみを理由として監視を自動再開してはならない。監視の再設定には明示的な操作を必要とする。
- RootMissingのみを理由としてSave Asを禁止してはならない。

## 4. 復旧対象と責務
<!-- hldocs:sec_id=sec_f8ef31fd51c296a4f223131b -->

| 対象 | 復旧契約 |
|---|---|
| Project metadata | メモリ上の状態を保存対象とする |
| Framework管理状態 | メモリ上の状態を保存対象とする |
| Resource Registry | Resource IDと参照情報を維持する |
| Application管理状態 | ApplicationProjectAdapterの責務として保存する |
| アクセス可能なProject Resource | 新しいProject Rootへの複製対象とする |
| 取得不能なファイル内容 | 復元不能として明示する |

FrameworkはApplication固有データの意味や内部保存形式を解釈してはならない。Application固有データの復旧可能性はApplicationProjectAdapterの責務とする。

取得不能な内容を空データ等で暗黙に置換し、完全に復旧したと扱ってはならない。

## 5. 欠落と利用者判断
<!-- hldocs:sec_id=sec_4e4603724a0a2d49b3209a73 -->

- 復旧できない内容がある場合、利用者が欠落対象と保存への影響を識別できなければならない。
- 欠落を伴うSave Asの確定には、利用者の明示的な同意を必要とする。
- 同意が得られない場合、部分保存を確定してはならない。
- 同意済みの欠落と、保存処理そのものの失敗を区別しなければならない。
- 同意の対象外の欠落を暗黙に許容してはならない。

## 6. 保存確定と状態保持
<!-- hldocs:sec_id=sec_325de8e019361e4562e94098 -->

Save Asの確定は、合意された保存対象について保存処理が成立し、保存世代の整合性が確保された場合に限る。

- 確定前にCurrent Project Rootを切り替えてはならない。
- Resource IDおよびApplicationが保持する論理参照を維持する。
- 元Projectの保存内容を変更してはならない。
- 未確定の保存先を正常な確定済みProjectとして扱ってはならない。
- 中止または失敗時はCurrent Project、メモリ上の状態およびDirtyを維持する。
- Dirtyの解除は、既存Application / Project仕様に従い、対応する保存が成立した範囲に限る。
- 欠落を伴う保存を完全復旧として扱ってはならない。

保存世代の整合性はREG-001の保存整合性契約に従う。本仕様は通常Saveに対する一括rollbackの保証を追加するものではない。

## 7. 結果の区別
<!-- hldocs:sec_id=sec_fb9237d862d88c0681d0d45a -->

| 結果 | Current Project Root | 保存結果の扱い |
|---|---|---|
| 完全保存成立 | 新ルートへ切替 | 完全保存 |
| 明示同意済みの欠落を伴う保存成立 | 新ルートへ切替 | 欠落を伴う保存 |
| 同意なし・取消 | 維持 | 未確定 |
| 保存処理失敗 | 維持 | 未確定 |

欠落を伴う保存が成立した場合も、欠落Resourceの参照を保持し、利用可能性を偽ってはならない。

Project lifecycle Eventは既存Application / Project仕様に従う。取消または失敗に対してSavedAsを通知してはならない。

## 8. Resource復旧との関係
<!-- hldocs:sec_id=sec_b45ec7593713bcc2c4590443 -->

新ルートにおけるProject Resourceの参照は、保持された相対パスを基準とする。旧ルートの絶対パスを復旧先として固定してはならない。

Missing Resourceの再出現と利用可能判定は、Resource SAVE-RECOVERY復旧検証仕様に従う。保存成立のみを理由にMissingをAvailableへ変更してはならない。

## 9. 関連仕様

- [Application / Project仕様](02_Application_Project.md)
- [Application-level Public I/F設計](02_Application_Public_IF_設計.md)
- [Resource SAVE-RECOVERY 復旧検証仕様](05_Resource_SAVE-RECOVERY_復旧検証仕様.md)
- [REG-001 Framework管理Resource Registry 再設計仕様](REG-001_Framework管理ResourceRegistry_再設計.md)

---

[目次](../目次.md) > [仕様目次](仕様目次.md) > [Application / Project](02_Application_Project.md) > SAVE-RECOVERY
