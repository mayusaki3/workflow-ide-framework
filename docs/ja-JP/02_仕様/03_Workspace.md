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

Applicationは常に一つ以上のWorkspaceを持つ。

利用アプリケーションは複数のWorkspaceを登録でき、使用するWorkspaceを選択できる。利用アプリケーションがWorkspaceを明示的に登録しない場合も、KairiがDefault Workspaceを提供する。

## 3.2 Workspace ID

各Workspaceは、表示名とは分離したstableな`Workspace ID`を持つ。

Workspace IDは、Workspaceの選択、保存済みレイアウトとの対応付け、状態復元に使用する。

KairiのDefault Workspaceは次のIDで識別する。

```text
framework.default
```

`framework.default`はKairiが予約するWorkspace IDとする。

## 3.3 Default Workspace

`Default Workspace`はApplicationの標準Workspaceである。

利用アプリケーションがWorkspaceを明示的に登録しない場合、Kairiは`framework.default`を使用する。

利用アプリケーションが独自Workspaceを登録する場合も、Applicationが使用するWorkspaceが一つ以上存在する状態を維持する。

Default WorkspaceはMain Windowの標準作業領域として使用できる。Dialog内で使用するWorkspaceとは識別子および状態を共有しない。

## 3.4 Panel配置

Workspaceは、登録済みPanelを参照して配置する。

Panelの参照にはPanelを識別するIDを使用する。複数instanceを持つPanelでは、Panel定義とinstanceを識別できる組を使用する。

同一のPanel instanceを同じWorkspaceのLayout内へ重複配置しない。

Workspaceに配置されていないPanelが存在することを許容する。

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

Workspaceは、配置先が指定されていないPanelを受け入れる既定のPanel Containerを一つ持つ。

利用アプリケーションがPanelの初期配置を指定しない場合、そのPanelは既定Containerへ配置できる。

## 3.7 Application default layout

利用アプリケーションはWorkspaceの初期Layoutを定義できる。

初期LayoutはPanelの内部実装ではなく、Panelの識別子と配置関係によって定義する。

Layoutが未登録のPanelを参照している場合は不正なLayoutとして扱い、Kairiが未知のPanelを暗黙に生成しない。

登録済みPanelが初期Layoutに含まれていないことは許容する。

## 3.8 保存済みLayout

WorkspaceのLayoutは永続化できる。

保存済みLayoutが存在する場合は、そのWorkspaceについて利用アプリケーションが定義した初期Layoutより保存済みLayoutを優先する。

保存済みLayoutが存在しない場合は利用アプリケーションの初期Layoutを使用し、それも存在しない場合はKairiの既定配置を使用する。

保存先がProject設定かUser設定かは、Workspaceまたは利用アプリケーションが選択する永続化範囲に従う。

## 3.9 LayoutとPanelの分離

WorkspaceのLayout変更はPanel自体のownershipを変更しない。

Panelの移動、表示・非表示、Dock状態等はWorkspaceの配置状態として扱い、Panel固有のデータや利用アプリケーション固有のdomain stateとは分離する。

Panelの表示状態変更によって、利用アプリケーション固有Runtimeのownershipまたはlifetimeを暗黙に変更しない。

## 3.10 Workspace切替

Workspaceを切り替えた場合、選択されたWorkspaceのLayoutを表示状態へ反映する。

Workspace切替によってPanel定義そのものを登録解除しない。

同じPanelを複数Workspaceから利用する場合も、各Workspaceはそれぞれの配置状態を管理する。

## 3.11 Dialog Workspace

Dialog内にもWorkspaceとPanel配置の共通機構を使用できる構造とする。

Dialog内のWorkspaceはMain WindowのDefault Workspaceとは別のWorkspace instanceとして扱い、LayoutおよびPanel配置状態を共有しない。

Kairi標準Dialogで使用する標準構成を`Default Dialog Workspace`として扱う。

Default Dialog Workspaceの公開API、Workspace ID、Layout capability、永続化範囲等の詳細契約は本仕様では固定しない。

---

[目次](../目次.md) > 仕様 > Workspace
