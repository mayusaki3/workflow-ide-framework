# SAVE-CONS-001 プロジェクト保存世代の整合性・実装前テスト仕様

状態：実装前テスト仕様（新経路未実装）。対象：REG-001 第8章。既存の `save_project()` は互換経路として維持し、この仕様の合格対象としない。

## 1. 目的と範囲
Framework Registry、アプリ所有データ（`application/` 配下）、Projectメタデータを同一の確定保存世代として復元する。アプリの論理IDとFramework Resource IDを維持する。ファイル削除等のResource Operationジャーナルは別機構とし、本仕様の保存世代と衝突しないことを確認する。

## 2. 保存世代とディレクトリ契約（設計確定事項）
- 新経路はプロジェクト直下の `framework/save-generations/` をFramework管理領域とする。世代ごとに不変の `<generation-id>/application/`、`framework_settings.toml`、`project.toml` を格納する。世代IDは衝突しない不透明文字列で、ファイル名として安全な文字のみ許す。
- `framework/save-generations/committed.toml` は確定世代IDを指す小さなコミット記録とする。更新は同一ディレクトリ内の一時ファイル作成・同期・原子的置換を用いる。確定記録の更新成功が唯一のコミット点。
- 準備中の世代は `framework/save-generations/staging/<generation-id>/` に構築する。アプリにはその中の `application/` のみを書き込み先として渡し、現行の `application/` や既存確定世代への書き込みを禁止する。
- 新経路でOpenする場合、`committed.toml` が指す世代の3要素のみを同時に読み込む。従来のプロジェクト直下の保存ファイルを混在させない。確定記録が存在しない従来プロジェクトは従来Open経路で読み込む。
- 旧来の `project.toml` と `framework/framework_settings.toml` を利用する既存APIとの共存期間中、新経路のデータを従来経路で上書きしない。保存入口の切替とOpen入口の切替は同時に行う。
- 保存候補の書き込み・検証完了後に世代ディレクトリを確定位置へ移動し、最後に `committed.toml` を更新する。記録更新前の世代は未確定であり、Openは採用しない。
- 保存世代の各ファイルには同一 `save_id` を記録する。アプリデータの形式はアプリが所有するため、アプリには世代IDと照合できるメタデータをステージング内に含める契約を課す。
- 完了済みの前世代は、少なくとも次の確定世代が検証されるまで削除しない。未確定世代の清掃は確定記録を変更せず行う。

## 3. 最小アプリAPI案（新経路専用）
```rust
fn stage_project_data(
    &mut self,
    destination_application_dir: &std::path::Path,
    save_id: &str,
) -> Result<ApplicationSaveResult, Self::Error>;
```
- アプリは自身のメモリ上の状態を指定ディレクトリへ全量出力し、既存保存データを直接変更しない。
- Frameworkはコールバック失敗時にコミット記録を変更しない。
- Save Asは元プロジェクトの確定世代をコピーするのではなく、現在のメモリ上のRegistryとアプリ状態からコピー先へ新しい世代を構築する。IDは再生成しない。
- 実装時には `ApplicationProjectAdapter` へ互換性を壊さない別トレイトまたは明示的な新APIを追加する。既存の `save_project_data()` の暗黙フォールバックは認めない。

## 4. 失敗注入・期待結果
| ID | 注入点・入力 | 期待結果 |
|---|---|---|
| SAVE-CONS-001 | Registryとアプリ参照を更新し正常Save→Close→Open | 両者が同一世代に復元され、ID不変、Dirtyクリア |
| SAVE-CONS-002 | アプリのステージング処理がエラー | コミット記録と旧世代不変、Dirty維持、再試行可能 |
| SAVE-CONS-003 | Framework設定のシリアライズ／ステージング書き込みが失敗 | 旧世代不変、Dirty維持、アプリ候補は未確定 |
| SAVE-CONS-004A | 世代ディレクトリ移動前に中断 | Openは旧世代のみ採用、未確定候補は無視 |
| SAVE-CONS-004B | 世代ディレクトリ移動後・確定記録更新前に中断 | Openは旧世代のみ採用、新世代は未確定として隔離 |
| SAVE-CONS-004C | 確定記録更新直後に中断 | Openは新世代のみ採用、旧世代は復旧用に保持 |
| SAVE-CONS-004D | 確定記録が存在するが参照先世代が欠落・破損 | 旧世代への黙示フォールバック禁止、明示的な復旧待ち |
| SAVE-CONS-005 | Save As成功 | 元プロジェクト不変、コピー先のRegistry IDとアプリ論理ID維持 |
| SAVE-CONS-006 | Save As途中失敗 | 元プロジェクト不変、コピー先は通常Open不可、未確定候補を識別 |
| SAVE-CONS-007 | 未保存のRegistryとアプリ状態を破棄しClose→Open | 最後の確定世代のみ復元 |
| SAVE-CONS-008 | アプリ参照IDが確定Registryに存在しない | アプリ検査フックが不整合を報告し、黙って正常Openしない |
| SAVE-CONS-009 | 新経路と従来経路が同じプロジェクトへ保存を試みる | 混在保存を拒否し、確定世代を保持 |
| SAVE-CONS-010 | 同時保存要求、世代ID重複、または不正世代ID | 競合・不正入力を拒否し、既存確定世代を保持 |

注入はテスト専用の明示的なフェーズ境界で行う。OSの偶発的なI/O失敗に依存しない。各ケースで確定記録、世代内3要素、Dirty、メモリ上Registry、アプリ状態、再Open結果を検査する。

## 5. OSと復旧の条件
- Windows / Linux / macOSを対象とする。世代ディレクトリ移動とコミット記録置換は同一ファイルシステム上で実施し、クロスデバイス移動に依存しない。
- Windowsでは開いているファイルやアンチウイルス等による置換失敗があり得る。コミット記録の置換に失敗した場合は保存失敗として旧記録を維持し、Dirtyを維持する。
- ファイル・ディレクトリの同期可能範囲はOSごとに検証する。クラッシュ耐性を未検証のOSについて保証済みと表現しない。
- 復旧時に破損した確定世代を自動で旧世代へ戻さない。利用者に異常を通知し、明示的な復旧操作を要求する。

## 6. 既存回帰試験・導入ゲート
- 新経路専用テストと既存テストを分離する。既存 `project_save`、`project_open`、`project_persistence`、`project_lifecycle`、`resource_executor`、`resource_delete` の期待値を変更せず通す。
- 保存・Open・Save Asの新経路が全件PASSするまでController/UIの既定経路は変更しない。
- REG-DELの確認付き削除テスト19件を維持し、削除操作後の保存世代再Openを追加検証する。
- Windowsの実測検証に加え、Linux/macOSで未実施の項目は「未検証」と明記する。
- `cargo fmt --all -- --check`、`cargo clippy -j 1 --all-targets --all-features -- -D warnings`、`cargo test -j 1 --all-targets --all-features` を実施する。

## 7. 未確定・実装前に決める事項
1. 世代ディレクトリの命名、コミット記録の正確なTOMLスキーマ、破損検出のチェックサム形式。
2. 新経路のOpenでアプリが世代内ディレクトリから読み込むためのAdapter契約。
3. Resource Operationジャーナルの位置と新世代への反映境界。未完了のファイル操作が保存世代と衝突する場合の拒否条件。
4. 従来プロジェクトを初めて新経路で保存する際の移行・ロールバック契約。
5. 複数プロセスから同一プロジェクトへ保存する際の排他制御。
