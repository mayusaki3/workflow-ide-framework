<!--
HLDocS:LLM-MANAGED
doc_id: doc-20261123-110000Z-SR01
lang: ja-JP
canonical_title: Resource SAVE-RECOVERY 復旧検証仕様
document_type: spec
canonical_document: true
transport: [true_out]
-->

[目次](../目次.md) > [仕様](仕様目次.md) > Resource > Resource SAVE-RECOVERY 復旧検証仕様

# Resource SAVE-RECOVERY 復旧検証仕様

本仕様は、KairiのResource Registryに登録されたResourceの消失・再出現・検証・復旧に関する状態、制約および一貫性を定義する。

## 1. Purpose（存在理由）

- Resourceの一時的な消失を、登録情報や参照の削除と区別する。
- 再出現したResourceの利用可能性を検証結果に基づいて確定する。
- 非同期検証とプロジェクト切替が競合しても、異なるResourceやセッションへ検証結果を誤適用しない。
- Project PanelとResource利用操作に一貫した状態を提供する。

## 2. Non-goals（対象外）

- ProjectのSave Asによる保存世代確定、欠落ファイルの扱い、部分保存への同意契約は第2章Application / Projectの仕様に属する。
- Project Panelの具体的な表示部品・アニメーション・更新機構は第4章Panelの仕様に属する。
- アプリ固有ファイル形式の解釈はFrameworkの責務ではない。
- 本書は実装方式、操作手順、テスト条件を定義しない。

## 3. 用語・状態

### 3.1 Resource Status
<!-- hldocs:sec_id=sec_e4390c271897978f0bfd5181 -->

Resource StatusはResource実体の直近の確定した利用可能性を表す。少なくとも以下の概念を区別する。

- Available：対象Resourceが直近の確定した検証結果に基づき利用可能と判定された状態。
- Missing：対象Resourceが利用可能と確定できず、参照情報を保持した状態。

Resource StatusはValidation Statusと独立して管理しなければならない。Missingは検証開始のみを理由にAvailableへ変化してはならない。Availableの対象変更・再検証待機・実行中は直近のResource Statusを維持する。ただし実体の消失が確認された場合は検証完了を待たずMissingへ変更する。

### 3.2 Validation Status
<!-- hldocs:sec_id=sec_5486c895f4a8fe10934fdf0b -->

Validation Statusは検証の進行と結果を表し、少なくとも以下の状態を区別する。

- NotValidated：現在の対象に対する有効な検証結果がない。
- Pending：検証要求が待機している。
- Validating：検証が進行中である。
- Valid：現在の対象に対する必須検証が成功した。
- Invalid：現在の対象に対する必須検証が失敗または未成立である。
- Cancelled：検証要求が無効化または取り消された。

Validation Statusはセッション内の一時状態であり、プロジェクト保存データに永続化してはならない。CancelledのみをResource実体の消失や検証失敗の証拠として扱ってはならない。

### 3.3 識別情報
<!-- hldocs:sec_id=sec_6f54e0f13d044fb7a564ed72 -->

- Resource ID：Registry内でResource参照を識別する安定ID。
- Session ID：プロジェクトのOpenから切替・Closeまでの有効期間を区別する識別情報。
- Validation Generation：同一Resourceに対する検証対象の更新世代。
- Validation Target：現在のプロジェクトルート、ResourceReferenceおよび検証に影響する対象情報の組。

## 4. Resource参照の不変条件
<!-- hldocs:sec_id=sec_36d084a1dc4de0b1503d1a38 -->

1. 外部操作によるResource消失はRegistry登録解除を意味しない。Resource IDとResourceReferenceを維持しなければならない。
2. Project内のResource参照は相対パスの意味を維持し、旧プロジェクトルートの絶対パスを復旧先として固定してはならない。
3. 同一相対パスにResourceが再出現しても、検証成功前にAvailableとして扱ってはならない。
4. プロジェクトルートが消失した場合、ルート監視の異常と個別Resourceの検証結果を混同してはならない。
5. 永続化されたResource Statusのみを根拠として、再Open後のResource利用を許可してはならない。

Registryの所有権、ID維持、Missing時の参照保持および保存整合性は[REG-001 Framework管理Resource Registry 再設計仕様](REG-001_Framework管理ResourceRegistry_再設計.md)と整合しなければならない。

## 5. 検証契約

### 5.1 Framework共通検証
<!-- hldocs:sec_id=sec_f2b0b09e44dc08e0ff99d415 -->

Framework共通検証は、対象の存在、アクセス可能性および期待される実体種別を判定対象としなければならない。

### 5.2 アプリ固有検証
<!-- hldocs:sec_id=sec_5008d87cdac12a1c65f674a9 -->

アプリは固有の検証を提供してよい。各検証の必須性をRequiredまたはOptionalとして区別できなければならない。

- Requiredの失敗またはSkippedはResourceの利用可能判定を阻止する。
- Optionalの警告のみではResourceの利用可能判定を阻止しない。
- Requiredの実施不能を成功として扱ってはならない。
- 独立して判定可能な複数の不具合は、それぞれ診断情報として保持できなければならない。
- 実施不能な検証はSkippedとし、原因となる診断識別情報を関連付けなければならない。

### 5.3 利用可能性
<!-- hldocs:sec_id=sec_e283e27df679a175849e7347 -->

Resource StatusをAvailableへ変更できるのは、現在のValidation TargetについてFramework共通検証とすべてのRequired検証が成功した場合に限る。現在の必須検証の失敗または未成立が確定した場合はMissingへ変更または維持する。

Resourceを必要とする新規操作は、現在のValidation Targetに対する有効な検証成功を前提としなければならない。Availableのみを根拠として利用を許可してはならず、有効な検証結果がない場合は検証成功を待つ。無関係なResourceの操作を一律に停止してはならない。

## 6. 非同期検証の制約

### 6.1 実行数・重複
<!-- hldocs:sec_id=sec_952fc34ec9c82d85d7be6968 -->

- 全セッションを通じた同時検証実行数は最大2とする。
- 同一Resource IDに対する待機要求は、同一セッション内で最大1件とする。
- 同一Resource IDの重複要求は統合し、最新のValidation Generationを有効とする。
- 固定の待機件数上限を設けない。
- 検証の進行に周期的ポーリングを必須としてはならない。

### 6.2 優先度と公平性
<!-- hldocs:sec_id=sec_eaa877437c9d025cbb1c77eb -->

- 利用者操作に起因する検証要求はHigh、OS通知に起因する要求はNormalとする。
- 同一優先度内の開始順序はFIFOとする。
- 両優先度に待機要求が存在する場合、Highを3件開始した後にNormalを1件開始する公平性を維持する。
- Normal待機中のResourceに利用者操作が発生した場合、その要求をHighの末尾へ昇格させる。昇格した要求をNormalへ降格させてはならない。

### 6.3 検証対象変更
<!-- hldocs:sec_id=sec_6652bb1067ba1d0e173e3e9e -->

検証中に対象が変更された場合、旧世代の結果をRegistryへ反映してはならない。通常のファイル変更のみを理由として実行中検証の強制中止を必須としてはならず、最新世代の再検証が成立しなければならない。

### 6.4 セッション切替とキャンセル
<!-- hldocs:sec_id=sec_4ae1105c8ab0f7fb09085d79 -->

- プロジェクト切替・Closeは旧セッションの待機要求を無効化する。
- 旧セッションの実行中要求には協調キャンセルを通知するが、切替完了をその終了待ちに依存させてはならない。
- 同一プロジェクトの再Openは新しいSession IDとして扱う。
- 旧セッションの検証が終了していなくても、全セッション合計の同時実行数2を超えてはならない。
- Resource削除または検証対象の無効化も協調キャンセルの対象とする。
- アプリ固有検証器による協調キャンセル対応は任意とする。非対応でも旧結果の適用禁止を維持しなければならない。

## 7. 検証結果の確定境界
<!-- hldocs:sec_id=sec_925d45edc497a099bf3ac558 -->

検証結果をRegistryへ確定する時点で、次の条件がすべて成立しなければならない。

1. Session IDが現在有効である。
2. Resource IDが現在のRegistryに登録されている。
3. 検証対象が有効である。
4. Validation Generationが現在の世代と一致する。
5. キャンセルが要求されていない。
6. Validation Targetが検証時と一致する。

これらの判定とRegistryへの結果反映は、他の状態変更による割込みで不整合が生じない一貫した確定境界を持たなければならない。Project Panelへの状態変更通知はRegistry反映後の状態を参照しなければならない。

## 8. 再Open・再出現時の状態
<!-- hldocs:sec_id=sec_0d2d5c75f41a39f64b3a6bca -->

- 再Open時にMissingとして保持されているResourceはNormal優先度の検証対象とする。
- その他のResourceは利用要求または変更通知を契機として検証対象とする。
- 検証成功までMissingを維持する。
- 再検証が開始された場合、以前のInvalid診断を新たな検証結果と混同してはならない。

## 9. 診断情報の契約
<!-- hldocs:sec_id=sec_af3dda9a59d04a6f6aaf4899 -->

診断情報は少なくとも以下の項目を識別できなければならない。

- resourceId
- sessionId
- validationGeneration
- errorCategory
- errorCode
- message
- source
- occurredAt

errorCodeは安定した識別子とし、利用者向けmessageはローカライズ可能とする。sourceはFramework由来とアプリ由来を区別する。

現在有効な診断と履歴ログは独立した情報とする。再検証中の旧診断は内部的に保持してよいが、最新検証の確定成功時に消去し、失敗時に新結果へ置換する。セッション切替またはResource削除後、旧診断を現行Resourceの診断として表示してはならない。

## 10. 状態公開と観測情報
<!-- hldocs:sec_id=sec_bd005d88f3d0d2cf0dce0421 -->

Resourceの状態を参照する利用者は、Resource Status、Validation Status、現在有効な診断およびValidation Generationの整合した組を取得できなければならない。

検証処理は少なくともpending、running、high、normal、maxWait、coalesced、promotedの観測情報を提供できなければならない。

Project Panelに公開する状態は現在のResourceとセッションに対応しなければならない。表示上の優先順位および更新方式は第4章Panelの仕様で定義する。

## 11. 関連仕様との境界

- [REG-001 Framework管理Resource Registry 再設計仕様](REG-001_Framework管理ResourceRegistry_再設計.md)：Registry所有権、Resource ID、参照、Missing、保存整合性。
- 第2章Application / Project：Save As復旧、部分保存同意、保存世代の一貫性。
- 第4章Panel：診断アイコン、親ノードへの状態伝播、表示更新とDirty Flag。
- 第5章Resourceの監視仕様：ProjectDirectoryWatcherのRootMissing、監視再設定、OS通知の扱い。

---

[目次](../目次.md) > [仕様](仕様目次.md) > Resource > Resource SAVE-RECOVERY 復旧検証仕様
