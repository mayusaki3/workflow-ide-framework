# REG-001 Framework管理Resource Registry 再設計仕様（草案）

## 1. 目的・状態
状態: 設計草案（実装未着手）。一般公開前のため旧APIとの後方互換は要求しない。Meridianは連携検証対象の一例であり、仕様はアプリ非依存とする。

## 2. 所有権と境界
- FrameworkHostが現在のProjectに対応するResourceRegistryを所有する。Newで初期化、Openで復元、Saveで永続化、Save AsでIDを維持、Closeで破棄。
- Registryはstable Resource IDとResourceReference（Application / Project / External）を管理する。
- Project PanelのNode IDとResource IDは別。論理ノードはRegistry登録を必須とせず、未表示の登録リソースも許す。同一Resource IDを複数ノードが参照してよい。
- アプリは自身の論理データと参照関係を所有する。Frameworkはアプリ固有の意味を推測しない。

## 3. 登録・更新
- FrameworkがResource IDを生成し、一意性を保証する。既存のnormalized Scope + Pathを登録要求された場合は既存IDを返す。
- ResourceReferenceの変更（移動・改名）でもResource IDを維持する。
- Registry単独の「登録解除」をユーザー操作として公開しない。登録情報はリソースの作成・削除・移動等に連動して更新する。
- ファイル削除はユーザー操作を起点とする。Registryから消すだけで実ファイルを残す独立操作は通常フローに設けない。
- アプリの論理項目削除では、アプリが関連する論理データを削除する。共有ファイルを暗黙削除しない。削除対象の物理ファイルがあればユーザーへ対象と影響を示す。

## 4. 削除と参照確認
- Frameworkは削除前にApplicationから対象Resource IDの参照状況を取得できる任意のフックを用意する。
- 参照数は永続カウンタではなく、その時点のアプリデータから計算する。フック未提供・取得失敗は「不明」とし、0とは扱わない。
- 参照数 > 0 の場合は、利用中の可能性、参照数、削除対象を明示して続行確認する。0でも通常の削除確認を省略する根拠にはしない。
- 論理項目の削除で関連データをどう処理するかはアプリが決める。アプリ処理が拒否・失敗した場合、Frameworkは成功扱いにしない。
- 実ファイル削除とRegistry更新の不整合は既存Resource Operationジャーナル／復旧機構と連携して扱う。部分失敗を隠蔽しない。
- プロジェクト外部でファイルが消失した場合はMissing状態として参照を維持し、勝手にRegistryから消さない。

## 5. 多言語
- ResourceEntryは任意のdisplay_name_key、description_keyを持てる。翻訳文字列はRegistryに保持せず、既存localization機構が解決する。
- ResourceReferenceのパスには任意で `{locale}` を含められる。Resource IDはLocale変更後も同一。
- Localeは検証したBCP 47言語タグのみ受け付け、パス区切り・親ディレクトリ遷移を許さない。
- 単純なテンプレート展開と、ファイル存在確認＋既定Localeフォールバックを別操作とする。全候補がない場合は未解決。
- Registry保存値は展開前のパステンプレートとする。既定Localeの選択元・設定項目は実装前に確定する。

## 6. 保存・API変更
- FrameworkSettings.resourcesにRegistry内容と任意の言語キーを保存する。Open時は検証してRegistryを復元する。
- application_project_panel(panel_id, provider, registry_source)をFramework所有Registry利用の2引数APIへ変更する。旧3引数API維持は要求しない。
- アプリ向けに登録・ID検索・参照更新・読み取りSnapshot・Resource IDからの解決を提供する。削除はユーザー操作を受けたResource Operation経由を基本とする。
- Registry変更はFramework state dirtyへ反映する。Save/Save As失敗時はdirtyを維持する。

## 7. 未決事項
1. 論理項目削除でアプリデータ更新と物理ファイル削除をまたぐ失敗時の復旧契約。
2. 参照確認フックの具体的な戻り値（件数、参照元一覧、未知、拒否）。
3. Locale別ファイルの既定Locale設定元と、対象がExternal scopeの場合の扱い。
4. Registryに存在するが実体がMissingの項目をユーザーが明示的に整理する復旧UIの位置付け。

## 8. 保存整合性の設計検討（2026-10-19）

### 8.1 現行実装の確認
- `ProjectController::save()` はメモリ上のRegistryを `FrameworkSettings.resources` に反映し、`save_project()` に渡す。
- `save_project()` はアプリの `save_project_data()`、Framework設定、Projectメタデータの順でそれぞれ書き込む。複数ファイルを一括コミットする機構はない。
- 成功時のみ `ProjectSession::complete_save()` がDirtyをクリアする。失敗時はメモリ上のRegistryとDirtyを維持するが、書き込み済みファイルは自動的に戻らない。
- `save_project_as()` はアプリデータ保存、resourcesディレクトリのコピー、Framework/Project保存の順。途中失敗時のコピー先のクリーンアップは保証されない。
- `save_id` はFramework設定とProjectメタデータに記録され、Open時に不一致を検出できる。アプリデータの `save_id` 一致はFramework側では検証しない。

### 8.2 目標と不変条件
1. アプリの論理データが参照するRegistry Resource IDとRegistry本体は、**同じコミット世代**として復元される。
2. Save失敗時、メモリ上のRegistryとアプリの未保存変更は保持し、Dirtyをクリアしない。
3. Save Asで論理IDとRegistry Resource IDを変更せず、失敗時に元プロジェクトを変更しない。
4. Open時に不完全な保存世代を検出したら、正常な最新確定世代へ復旧するか、利用者に明示して操作を止める。**不整合な世代を黙って採用しない**。
5. `ApplicationProjectAdapter` はアプリデータの保存形式を所有し、Frameworkはその意味を解釈しない。

### 8.3 提案する最小API（未実装）
- 既存の `save_project_data()` / `save_project_data_as()` を即時書き込み型として残す間は、原子的保存を保証しない。
- 新たにアプリが **ステージング先** にデータを書き出すコールバックを追加する。Frameworkはアプリ・Registry・Projectメタデータを同じ保存世代の候補として準備する。
- Frameworkはコミット記録を管理し、完了済み世代だけをOpenで採用する。複数ファイルの置換中断に備え、前世代を復元できるようにする。
- アプリデータが複数ファイルの場合も扱えるよう、コールバックの対象は単一ファイルではなくディレクトリとする。
- 新APIに移行するアプリはステージング領域外へ書き込まない。従来API利用アプリの保証水準は別途明示する。

### 8.4 実装前テスト候補
| ID | 条件 | 期待結果 |
|---|---|---|
| SAVE-CONS-001 | Registryとアプリデータの同時変更後にSave | 両者の同じ保存世代がOpenで復元される |
| SAVE-CONS-002 | アプリのステージング失敗 | 既存保存世代不変、Dirty維持 |
| SAVE-CONS-003 | Framework設定の準備失敗 | 既存保存世代不変、Dirty維持 |
| SAVE-CONS-004 | コミット途中で中断 | Openで前世代へ復旧、または明示的な復旧待ち |
| SAVE-CONS-005 | Save As成功 | 元プロジェクト不変、Resource ID維持 |
| SAVE-CONS-006 | Save As失敗 | 元プロジェクト不変、コピー先を未確定として扱う |
| SAVE-CONS-007 | 保存せずClose/Open | 未保存Registryとアプリデータを破棄 |
| SAVE-CONS-008 | アプリの参照IDがRegistryに存在しない | 不整合を通知し、黙って正常扱いしない |

### 8.5 作業順序と既存テストへの影響
1. **先に保存整合性の仕様と失敗注入テストを確定する**。既存の `project_save` / `project_persistence` / `project_open` / `project_lifecycle` / `resource_executor` の現行期待値を記録する。
2. 新しいステージング保存経路を**別APIとして実装**し、既存保存APIは直ちに変更しない。既存テストへの破壊的変更を避ける。
3. 新経路の成功・失敗・再オープンテストを通した後、`ProjectController` とUIの保存入口を切り替える。
4. REG-DELの直接削除経路制限と確認UIを、新しい保存契約の下で完成させる。
5. Project Panelの追加ノード自動展開を実装する。

注：本章は設計検討であり、ステージング・世代復旧・アプリ参照整合性検査は**現時点で未実装**。

### 8.6 既存テストの保護範囲と切替条件（2026-10-19）

既存の `save_project()` はアプリ→Framework設定→Projectメタデータの順序を検証しており、`tests/project_save.rs` の「Projectメタデータを最後に書く」「アプリ保存失敗時は後続を書かない」「失敗時のメモリ上メタデータとDirtyを維持する」を維持する。これらは**従来経路の互換テスト**であり、原子的保存の証明ではない。

新経路の導入前に次を固定する。

- 保存世代の確定点は、全候補の準備と永続化が成功した後のコミット記録更新とする。Openは確定記録の指す世代のみ採用する。
- 保存候補の作成に失敗した場合、既存の確定世代と元プロジェクトの保存ファイルを変更しない。失敗注入点はアプリステージング、Frameworkシリアライズ、Framework書き込み、Projectメタデータ書き込み、コミット記録更新。
- アプリは指定されたステージングディレクトリ内に全プロジェクトデータを書き出す。保存中に現行の `application/` を変更する方式は新経路では禁止する。
- 新経路のSave Asはコピー先の未確定世代を通常Open対象にしない。元プロジェクトのメモリ状態・保存世代・Resource IDを変更しない。
- 従来経路と新経路を同時に切り替えず、まず新経路の専用テストを実装する。新経路がOpen/Save As/復旧を通過してからController/UIの入口を切り替える。
- Registryとアプリ参照の意味的整合性は、世代の原子性とは別にアプリの検査フックで判定する。世代一致だけで参照IDの存在を保証したと見なさない。

**既存回帰テスト（切替前後とも実行）**：`project_save`、`project_open`、`project_persistence`、`project_lifecycle`、`project_new`、`resource_executor`、`resource_delete`。Save Asの専用テストは実在するテストターゲット名を確認して追加する。

**次の実装前成果物**：SAVE-CONS-001〜008を独立したテスト仕様に展開し、失敗注入の方法、確定世代の保存形式、復旧ポリシー、Windows/Linux/macOSのファイル置換契約を確定する。
