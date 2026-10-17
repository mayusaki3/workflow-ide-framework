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
