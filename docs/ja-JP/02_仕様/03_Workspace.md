<!--
HLDocS:LLM-MANAGED
doc_id: doc-20261006-kairi-workspace
lang: ja-JP
canonical_title: Workspace 仕様
document_type: spec
canonical_document: true
-->

[目次](../目次.md) > 仕様 > Workspace

# Workspace（ワークスペース）

## 3.1 Workspace

`Workspace`（ワークスペース）は、利用アプリケーションで使用する`Panel`（パネル）の配置とレイアウトを管理する作業空間である。

Panel自体はWorkspaceが所有しない。Kairiまたは利用アプリケーションが所有するPanelをWorkspaceへ配置する。

KairiはApplicationにDefault Workspaceを一つ提供する。Default Workspaceは常に存在するため、Applicationは常に一つ以上のWorkspaceを持つ。

利用アプリケーションはDefault Workspaceのみを使用できる。複数の作業空間が必要な場合は、独自のWorkspaceを追加登録し、使用するWorkspaceを選択できる。

WorkspaceはPanelを配置するための作業空間として扱い、Workspace内のLayoutは利用アプリケーションが構成できる。

## 3.2 Workspace ID

各Workspaceは、表示名とは分離したstableな`Workspace ID`を持つ。

Workspace IDは、Workspaceの選択、保存済みLayoutとの対応付け、状態復元に使用する。

KairiのDefault Workspaceは次のIDで識別する。

```text
framework.workspace.default
```

`framework.workspace.default`はKairiが予約するWorkspace IDとする。

## 3.3 Default Workspace

`Default Workspace`は、KairiがApplicationに常に提供する標準Workspaceである。

利用アプリケーションはWorkspaceを独自に登録することなく、Default Workspaceを使用できる。

利用アプリケーションが独自Workspaceを登録した場合も、Default WorkspaceはApplicationのWorkspaceとして存在する。

Default WorkspaceはKairiが所有し、利用アプリケーションから削除または登録解除できない。

Default WorkspaceのLayoutは利用アプリケーションが構成できる。

## 3.4 Panel配置

Workspaceは、登録済みPanelを参照して配置する。

Panelの参照にはPanelを識別するIDを使用する。複数instanceを持つPanelでは、Panel定義とinstanceを識別できる組を使用する。

同一のPanel instanceを、同一WorkspaceのLayout内へ重複配置することはできない。

同一のPanel instanceを、異なるWorkspaceへ配置することはできる。

同一Panel instanceを複数Workspaceへ配置した場合、それぞれのWorkspaceはそのPanel instanceの配置状態を独立して管理する。

登録済みPanelがいずれのWorkspaceにも配置されていないことは許容する。

## 3.5 Layout

`Layout`（レイアウト）はPanelとPanel Containerの配置関係を表す。

Layoutは少なくとも次を表現できる。

- 水平方向の分割
- 垂直方向の分割
- 同一領域への複数Panel配置
- 選択中のPanel
- 分割比率
- Panelの表示状態

LayoutはKairi内部で使用するDock backend固有のTreeまたはNodeを、利用アプリケーションへ直接要求しない。

## 3.6 Default Container

各Workspaceは`Default Container`を一つ持つ。

KairiはWorkspaceの作成時にDefault Containerを作成する。

Default ContainerはWorkspaceそのものではなく、WorkspaceのLayout内に存在するPanel Containerである。

Default Containerは、Panelの初期配置先または再表示先を他の情報から特定できない場合の既定配置先として使用する。

Panelの配置先または再表示先が明示されている場合は、その指定をDefault Containerより優先する。

Default Containerをユーザー操作によって変更する機能は提供しない。

## 3.7 Layout設定

利用アプリケーションは、WorkspaceおよびPanel Containerを設定した後、対象を指定してLayoutを設定できる。

Workspaceを対象とするLayout設定では、Workspace内のPanel Containerの配置関係、分割方向、分割比率等を設定できる。

Panel Containerを対象として、Panelの配置および同一領域内でのPanelの配置関係を設定できる。

Layout設定はPanelの内部実装ではなく、Workspace、Panel ContainerおよびPanelの識別子を使用して行う。

Layout設定が存在しないWorkspace、Panel ContainerまたはPanelを参照した場合、Kairiは対象を暗黙に生成しない。

登録済みPanelがいずれのWorkspaceにも配置されていないことは許容する。

利用アプリケーションは、WorkspaceごとにApplication既定Layoutを構成できる。

Application既定Layoutが構成されていない場合は、Workspace作成時にKairiが生成したDefault Containerを既定状態として使用する。

## 3.8 Layoutの永続化とリセット

WorkspaceのLayoutおよびPanel配置状態は、Project内のUser設定として永続化する。

保存済みLayoutが存在する場合は、そのWorkspaceについてApplicationが定義した既定Layoutより保存済みLayoutを優先して復元する。

保存済みLayoutが存在しない場合はApplication既定Layoutを使用し、それも存在しない場合はWorkspace作成時にKairiが生成したDefault Containerを使用する。

利用者は、対象Workspaceを指定してLayoutをApplication既定値へリセットできる。

現在表示中のWorkspaceを対象とするリセット操作では、他のWorkspaceのLayoutを変更しない。

表示されていないWorkspaceをリセットする場合は、対象Workspaceを明示的に指定する。

LayoutをApplication既定値へリセットした場合、対象Workspaceの保存済みLayoutを破棄し、その時点のApplicationが定義する既定Layoutから再構成する。Application既定Layoutが存在しない場合は、Workspace作成時のDefault Containerのみを持つ状態へ戻す。

Applicationの更新によって既定Layoutが変更された場合も、保存済みLayoutが存在するWorkspaceでは保存済みLayoutを優先する。

Application既定Layoutの変更を、既存の保存済みLayoutへ暗黙に適用またはマージしない。

## 3.9 LayoutとPanelの分離

WorkspaceのLayout変更はPanel自体のownershipまたはlifetimeを変更しない。

Panelの移動、表示・非表示、Dock状態等はWorkspaceごとの配置状態として扱い、Panel固有のデータや利用アプリケーション固有のdomain stateとは分離する。

同一Panel instanceを複数Workspaceへ配置した場合も、Panel instance自体は共有し、配置状態はWorkspaceごとに独立して管理する。

Panelの表示状態変更またはWorkspace切替によって、Panel instanceを暗黙に生成または破棄しない。

Panelの表示状態変更またはWorkspace切替によって、利用アプリケーション固有Runtimeのownershipまたはlifetimeを暗黙に変更しない。

## 3.10 Workspace切替

利用アプリケーションは、表示対象のWorkspaceを切り替えることができる。

Workspaceを切り替えた場合、選択されたWorkspaceが保持するLayoutおよびPanel配置状態を表示へ反映する。

Workspace切替によってPanel定義またはPanel instanceを登録解除、生成、破棄しない。

同一Panel instanceが切替前後のWorkspaceに配置されている場合、Panel instanceを維持したまま、切替先Workspaceが保持する配置状態を表示へ反映する。

各Workspaceは、それぞれのLayoutおよびPanel配置状態を独立して管理する。

## 3.11 DialogでのWorkspace利用

WorkspaceはMain WindowだけでなくDialog内でも使用できる。

Dialog内で使用するWorkspaceもMain Windowで使用するWorkspaceと同一のWorkspace機構を使用し、Workspace固有の型またはLayoutモデルを設けない。

Dialogで使用するWorkspaceも、作成時にDefault Containerを一つ持つ。

WorkspaceをMain WindowまたはDialogのどちらへ表示するかはUI上の表示形態として扱い、Workspace自体の機能またはデータモデルを変更しない。

Dialogで使用するWorkspaceは、他のWorkspaceと同様に独立したWorkspace ID、LayoutおよびPanel配置状態を持つ。

Dialogの終了はWorkspaceの表示終了として扱い、Workspaceの破棄を意味しない。

Workspaceの生成および破棄はWorkspaceの所有者が管理し、Main WindowまたはDialogの表示lifetimeとは分離する。

## 3.12 将来拡張

よく使用するWorkspace構成を再利用するためのWorkspace Templateは将来拡張として扱い、v0.1.0では仕様化しない。

Workspace Templateを導入する場合は、TemplateそのものをWorkspaceとして扱うのではなく、必要な構成をWorkspaceへコピーして利用する方式を想定する。

---

[目次](../目次.md) > 仕様 > Workspace
