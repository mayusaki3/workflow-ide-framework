<!--
HLDocS:LLM-MANAGED
doc_id: doc-20261126-000000Z-4E7B
lang: ja-JP
canonical_title: SAVE-RECOVERY 結合テスト仕様
document_type: testspec
canonical_document: true
transport: [true_out]
-->

[目次](../目次.md) > テスト仕様 > SAVE-RECOVERY 結合テスト仕様

# SAVE-RECOVERY 結合テスト仕様

実際の領域間連携を検証する。ファイル操作は隔離した一時ディレクトリで行い、競合条件は制御可能な検証器と同期点で再現する。各列挙条件は独立に検証し、実描画の確認結果は別途記録する。

## SAVE-RECOVERY-IT-SPEC-001 監視異常と状態保持
<!-- hldocs:sec_id=sec_60721e4ee3df6e9d6685745e -->
<!-- hldocs:ref=doc-20261124-020300Z-SR02#sec_e157eb7f35e664379a4b9553 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_36d084a1dc4de0b1503d1a38 -->

### 前提条件

編集中のProjectを監視する。

### 検証内容

一時ディレクトリのルートを消失させ、再出現後に明示的に監視を再設定する。

### 期待結果

RootMissingを通常変更と区別し、Projectと編集状態を保持する。再出現のみでは監視再開せず、明示操作で再設定できる。

## SAVE-RECOVERY-IT-SPEC-002 Save Asによる復旧
<!-- hldocs:sec_id=sec_07660ba1b8e8cd75dd124485 -->
<!-- hldocs:ref=doc-20261124-020300Z-SR02#sec_f8ef31fd51c296a4f223131b -->
<!-- hldocs:ref=doc-20261124-020300Z-SR02#sec_325de8e019361e4562e94098 -->
<!-- hldocs:ref=doc-20261124-020300Z-SR02#sec_fb9237d862d88c0681d0d45a -->

### 前提条件

メモリ編集、取得可能・不能なResourceを含むProject。

### 検証内容

ルート消失後、完全保存可能な場合と欠落への明示同意がある場合を別々にSave Asし、新ルートを再Openする。

### 期待結果

合意対象を保存し、世代整合性とID・論理参照を維持する。確定後のみ新ルートへ切替し、両保存成立をSave As成功として既存SavedAsを通知する。完全保存と欠落保存を保存結果で区別し、欠落参照を保持する。MissingのみではDirtyを維持せず、未保存のメモリ編集に対応するDirtyは維持する。

## SAVE-RECOVERY-IT-SPEC-003 欠落Resourceの再出現
<!-- hldocs:sec_id=sec_f64ab56637a769b3521a84f5 -->
<!-- hldocs:ref=doc-20261124-020300Z-SR02#sec_b45ec7593713bcc2c4590443 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_e283e27df679a175849e7347 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_0d2d5c75f41a39f64b3a6bca -->
<!-- hldocs:ref=doc-20261125-103800Z-SR03#sec_a06577628274dbec3bf1d6a2 -->

### 前提条件

欠落保存後のProject。

### 検証内容

新ルートの同一相対パスにファイルを配置し、必須検証失敗・Skipped・成功、任意警告を個別に発生させる。

### 期待結果

旧ルートに依存せず、ID維持。必須検証成功までMissingとし、任意警告のみなら利用可能。Panelは最新状態を反映する。

## SAVE-RECOVERY-IT-SPEC-004 Registry・検証・Panel連携
<!-- hldocs:sec_id=sec_146d3a2e5d37704116020873 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_925d45edc497a099bf3ac558 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_bd005d88f3d0d2cf0dce0421 -->
<!-- hldocs:ref=doc-20261125-103800Z-SR03#sec_789dbce32534508ad80d10ee -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_eaa877437c9d025cbb1c77eb -->

### 前提条件

複数Resourceと階層ノード、制御可能な検証器。

### 検証内容

OS通知と利用要求を混在させ、検証を開始・完了させる。検証中の変更と兄弟ノードの異常も発生させる。

### 期待結果

優先度・公平性を維持し、Registry確定後に整合した状態を通知する。旧結果を表示せず、親は最高優先状態を反映する。

## SAVE-RECOVERY-IT-SPEC-005 セッション切替競合
<!-- hldocs:sec_id=sec_c8ed0183ab3d88d919a6025a -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_4ae1105c8ab0f7fb09085d79 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_925d45edc497a099bf3ac558 -->
<!-- hldocs:ref=doc-20261125-103800Z-SR03#sec_3181fc1be6108411821f5380 -->

### 前提条件

旧セッションに待機・実行要求があり、キャンセル非対応検証器を含む。

### 検証内容

検証を制御点で停止し、別Projectへ切替または同一Projectを再Openしてから旧結果を返す。

### 期待結果

終了待ちなしで切替でき、旧待機を無効化し、全体実行数2以下。旧結果・診断を新RegistryやPanelに反映しない。

## SAVE-RECOVERY-IT-SPEC-006 非表示中の検証と再表示
<!-- hldocs:sec_id=sec_f84fa74121815efd5d6e0811 -->
<!-- hldocs:ref=doc-20261125-103800Z-SR03#sec_2500060a250655fa87951bd7 -->
<!-- hldocs:ref=doc-20261125-103800Z-SR03#sec_4f4835f994a4a8311596e8c1 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_bd005d88f3d0d2cf0dce0421 -->

### 前提条件

検証中のPanelと複数コンポーネント。

### 検証内容

非表示中に複数変更と検証完了を発生させ、再表示更新中にも変更を発生させる。

### 期待結果

表示専用処理停止、検証継続。再表示で最新状態を反映し、更新中の変更を失わず、Project保存用Dirtyと混同しない。

## SAVE-RECOVERY-IT-SPEC-007 保存失敗・取消
<!-- hldocs:sec_id=sec_060e9fbb2104f949d1400b9f -->
<!-- hldocs:ref=doc-20261124-020300Z-SR02#sec_4e4603724a0a2d49b3209a73 -->
<!-- hldocs:ref=doc-20261124-020300Z-SR02#sec_325de8e019361e4562e94098 -->
<!-- hldocs:ref=doc-20261124-020300Z-SR02#sec_fb9237d862d88c0681d0d45a -->

### 前提条件

欠落と未保存変更を含むProject。

### 検証内容

同意なし、取消、同意外欠落、同意後の保存失敗、世代不整合を別々に発生させる。

### 期待結果

未確定のまま旧ルート・メモリ・Dirtyを保持し、SavedAsを通知しない。未確定先を正常Projectと扱わない。

## SAVE-RECOVERY-IT-SPEC-008 実描画と診断表示
<!-- hldocs:sec_id=sec_f3518f8ea27bb11be482182a -->
<!-- hldocs:ref=doc-20261125-103800Z-SR03#sec_3f3b2419889615f304361598 -->
<!-- hldocs:ref=doc-20261125-103800Z-SR03#sec_2e5bdc4397043b5638892b33 -->
<!-- hldocs:ref=doc-20261123-110000Z-SR01#sec_af3dda9a59d04a6f6aaf4899 -->

### 前提条件

実際のPanelを起動し、複数の検証対象と診断を用意する。

### 検証内容

Theme・言語を変更し、Pendingと複数Validatingの描画、診断確認操作、検証完了を観察する。

### 期待結果

通常の状態表示はアイコンのみで状態を識別でき、Pendingは静的、Validatingは共通位相。診断コードと翻訳可能な表示を区別し、現在診断と履歴を混同しない。

---

[目次](../目次.md) > テスト仕様 > SAVE-RECOVERY 結合テスト仕様
