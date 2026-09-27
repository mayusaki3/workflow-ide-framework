# Project / Resource / Layout / Notification 仕様

## 1. 目的

v0.1.0でConsumer ApplicationとFrameworkの境界に影響するProject、Resource、Panel Layout、Notificationの基本契約を定義する。UIの細かな調整値は本仕様の互換性契約に含めない。

## 2. Project

ProjectはFrameworkのproject-specific state、Application固有data、関連Resourceをまとめる単位とする。

```text
<Project Root>/
├─ project.toml
├─ framework/
│  └─ framework_settings.toml
├─ resources/
└─ application/
```

`project.toml` は保存済みProjectでは必須とする。他のdirectoryは必要時に作成する。

Frameworkは `project.toml`、`framework/`、Project lifecycleを所有する。Applicationは `application/` 以下を所有し、Frameworkはその内容を解釈しない。

Application IDはApplicationが提供するstable opaque IDとし、Project互換性判定のApplication識別に使用する。Application name / descriptionは表示用であり互換性判定には使用しない。

Project metadataとApplication metadataはそれぞれlanguageを持てる。自動翻訳は行わない。

## 3. Save

Project DirtyはFramework metadata dirty、Framework state dirty、Application dirtyの論理和とする。

Saveは同一のSave IDを使用し、概念上次の順序で行う。

1. Application dataを保存
2. Applicationから実際に保存したdata_versionを受領
3. framework/framework_settings.tomlをatomic save
4. project.tomlを最後にatomic save
5. 全処理成功後にDirtyをclear

Project全体のtransaction/rollbackは要求しない。途中失敗時はDirtyを維持する。

Save Asは新しいProject Rootへ複製保存し、全処理成功後だけCurrent Project Rootを切り替える。Application dataはFrameworkがblind copyせず、Application Save As境界を使用する。

## 4. Application data compatibility

stored application.data_versionはhintとしてApplicationへ渡す。Applicationは実データを確認し、Compatible / Converted / Incompatibleを返す。

version差だけではConvertedとしない。実変換を行った場合だけConvertedとし、Projectをdirtyにする。FrameworkはApplication dataのmigrationを所有しない。

## 5. ResourceReference

Resourceは次のScopeを持つ。

- Application
- Project
- External

Application / Projectはroot-relative path、Externalはabsolute pathを使用する。Application Resource RootはApplicationが提供し、Project Resource Rootは `<Project Root>/resources/` とする。

Application RootとProject Resource Rootは同一または包含関係にしてはならない。

## 6. Resource Selector

SelectorはSingle / Multipleをcallerが指定する。File typeはv0.1.0ではextension filterとし、confirm時にも再検証する。

Application / Project Scopeはroot外へ移動できず、confirm時にcanonical pathによるroot confinementを検証する。Externalはこのroot制約を持たない。

選択結果はSelected / Cancelled / Errorを区別する。Cancelは通常操作として扱う。

## 7. Resource Registry

Project Panelはfilesystem treeではなくResource Registry viewとする。

Registry entryはstable resource_idとResourceReferenceを持つ。重複判定はnormalized Scope + Pathで行う。

Project PanelはProject Resources / Application Resources / External Resourcesに分類する。Project配下に存在するだけのfileは登録されるまで通常のProject Resourcesには表示しない。

Resource Selectorで選択したResourceはRegistryへ登録する。

## 8. Unreferenced / Unregistered

Unreferencedは登録済みProject ResourceのうちFramework/Application双方から使用されていないものとする。Applicationがusageを判定できない場合はUnreferencedへ分類しない。

Unregisteredは `resources/` に実在するがRegistryに存在しないfileとする。

両検索結果ともtemporary viewでありProjectへ永続化しない。

## 9. Resource operation / Journal

filesystemとFramework/Application persistent stateをまたぐ操作にはResource Operation Journalを使用する。単独のunregistered filesystem operationにはJournalを要求しない。

JournalはFrameworkがformat/storage/lifecycleを所有し、Application固有payloadはopaque dataとして保持する。Journal saveに失敗した場合、filesystem operationを開始しない。

一つのProjectではjournaled resource operationを同時に一つだけ実行する。

Journalはrollback logではない。異常終了後はJournalと現在のfilesystem / Registry / Framework refs / Application stateを比較し、consistent stateへ解決する。

Pending Journalだけを理由にProject Openを禁止しない。

## 10. Missing / Locate-Replace

Resourceが消失してもResourceReferenceを自動削除せずMissingとして保持する。

Locate / Replaceは新しいResourceReferenceを選択し、Framework内の同一旧Referenceをまとめて置換する。新Resourceが未登録ならRegistryへ登録する。旧Registry entryは自動Removeしない。

元pathへfileが戻った場合はWatcher/RefreshによりRestoredとして扱う。

## 11. Watcher / Root state

Watcherはcurrent resource stateを補助する機構でありdurable event historyではない。

Resource stateとWatcher status、Resource Root statusを分離する。Watcher failureをResource Missingとして扱わない。Root unavailable時は配下ResourceをMissingと断定しない。

Framework initiated operationのeventはPending Operationと相関させ、単純な時間windowだけで無視しない。

## 12. Panel Layout

LayoutはPanelとPanelContainerの再帰treeとする。v0.1.0のContainerはHorizontal / Vertical / Tabsを扱う。

Panel instanceの一意keyは `(panel_id, instance_id)` とする。同じpairをLayout内に複数配置しない。

Containerはstable container_idを持ち、Default Container roleを一つ確保する。Applicationが初期配置を指定しないPanelはDefault ContainerへTabとして追加する。

Panel close behaviorはHide / Destroyを区別する。HideはLayout位置を保持し、Destroyはinstanceを除去する。Destroy後の冗長Containerは正規化する。

Split比率や最小サイズ等の細かな既定値はv0.1.0で強い互換性契約にしない。

## 13. Notification

Notification dataとNotification Panelを分離する。PanelがHiddenでもNotificationは存在できる。

NotificationはInformation / Warning / Critical severityとOnViewed / OnAcknowledged / OnResolved dismiss policyを独立して持つ。

Notification Panelは通常のFramework Panelとして扱い、標準instanceは `framework.notifications/default`、close behaviorはHideとする。

同一notification_idの再発行は新規追加ではなく既存通知の更新とする。Notification object自体をProjectの永続状態とせず、Journal、Root、Watcher等のpersistent/current source stateから必要な通知を再構築する。

## 14. v0.1.0調整方針

Project format、Application API、ResourceReference、Journal、永続化ownership等のConsumer契約を優先して固定する。

Split比率、Panel最小サイズ、未読badge等の細かなUI挙動は仮決めで実装し、明確な不具合がない限りv0.1.0で追加調整しない。
