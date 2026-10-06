<!--
HLDocS:LLM-MANAGED
doc_id: doc-20261002-kairi-app-project
lang: ja-JP
canonical_title: Application / Project 仕様
document_type: spec
canonical_document: true
-->

[目次](../目次.md) > [仕様目次](./仕様目次.md) > 第2章 Application / Project

# 第2章 Application / Project

## 2.1 位置付け

本章は、`Application`（アプリケーション）と`Project`（プロジェクト）の基本契約を定義する。

利用アプリケーションはKairiを組み込み、アプリケーション固有機能とKairiの共通機能を組み合わせる。`Project`はKairiと利用アプリケーションが責務を分担して管理するデータ単位とする。

`Workspace`（ワークスペース）と`Panel`（パネル）は第3章以降で扱う。

## 2.2 Application

利用アプリケーションはKairiの`Application`を生成し、アプリケーション情報、必要な公開機能、`ApplicationProjectAdapter`を登録して`run()`を呼び出す。

`Application ID`は利用アプリケーションを識別するstable opaque IDとし、表示名とは分離する。Project互換性判定にも使用する。

Kairi内部のnative UI lifecycleはKairiが管理し、利用アプリケーションが直接管理することを基本契約としない。

## 2.3 Application metadata

利用アプリケーションは少なくとも次の情報をKairiへ設定できる。

| 項目 | 用途 |
|---|---|
| `Application ID` | 利用アプリケーションの識別、Project互換性判定 |
| `Application Name` | 表示名 |
| `Application Version` | 利用アプリケーションのバージョン |
| `Description` | 必要に応じてProject metadata等で使用 |
| `Language` | Application / Project metadataで使用 |

Kairi自身のバージョンは`FrameworkInfo`から利用アプリケーションが参照できる。

Kairiのバージョンをエンドユーザーへ表示・公開する方法は利用アプリケーションが所有する。Kairiは特定のAbout画面またはバージョン表示を必須としない。

## 2.4 Menu / Command

KairiはApplication levelの`Menu`（メニュー）と`Command`（コマンド）の基盤を提供する。

利用アプリケーション固有の操作は`Command`として登録し、Kairi標準Menuまたは利用アプリケーション独自Menuへ配置できる。

Kairi標準のProject lifecycle操作はKairiが提供する。

- New Project
- Open Project
- Save
- Save As
- Project Properties
- Close Project
- Exit

表示定義と処理を分離し、利用アプリケーション固有処理をKairi内部実装へ直接組み込まない。

## 2.5 Status

`Status`（ステータス）はApplication全体から確認できる軽量な状態表示とする。

利用アプリケーションはStatus itemを登録し、実行中にvalueとvisibilityを更新できる。

Statusは通知履歴またはログの代替としない。

## 2.6 Application lifecycle Event

利用アプリケーションへ次のApplication lifecycle Eventを通知する。

- `Starting`
- `Started`
- `CloseRequested`
- `Closing`

Eventは状態変化の通知とし、Kairiの終了判定を上書きするinterceptorとはしない。

`Closing`はProject dirty confirmation等を通過し、Application終了が確定した時点で通知する。

## 2.7 Projectの管理モデル

`Project`は、KairiのProject固有状態、利用アプリケーション固有データ、Project Resourceを一つのProject Rootで協調管理する単位とする。

保存済みProjectの基本構造は次のとおりとする。

```text
<Project Root>/
├─ project.toml
├─ framework/
│  └─ framework_settings.toml
├─ resources/
└─ application/
```

各領域の所有者は次のとおりとする。

| 領域 | 所有者 | 内容 |
|---|---|---|
| `project.toml` | Kairi | Project metadata、Application metadata、保存識別情報 |
| `framework/` | Kairi | Kairiが管理するProject固有状態 |
| `resources/` | Kairi / 利用アプリケーション | Project Resource領域 |
| `application/` | 利用アプリケーション | Application固有Project data |

KairiはProject RootとProject lifecycleを管理する。

利用アプリケーションは`application/`以下を自由に使用できる。Kairiはその内部file構造、データの意味、migrationを解釈または所有しない。

`resources/`は`application/`とは別のProject Resource領域とし、Resource仕様は第5章で定義する。

## 2.8 Project metadata

保存済みProjectでは`project.toml`を必須とする。

`project.toml`はProject metadataとApplication metadataを保持する。Project format versionがKairiの対応範囲より新しい場合は推測して読み込まず、Openを停止する。

Application IDが現在の利用アプリケーションと異なるProjectは、その利用アプリケーションのProjectとしてOpenしない。

Application data versionは利用アプリケーションによる互換性判定のための情報として保持する。version差だけを理由にKairiがApplication dataを変換しない。

## 2.9 ApplicationProjectAdapter

利用アプリケーション固有Project dataは`ApplicationProjectAdapter`境界を介してKairiと協調管理する。

利用アプリケーションは次を担当する。

- 新規ProjectのApplication data初期化
- 保存済みApplication dataの互換性判定
- Application dataの整合性判定
- Application dataのSave
- Application dataのSave As

KairiはApplication dataの意味、migration、内部保存形式を所有しない。

## 2.10 Project dirty state

Project全体のdirty stateは次の3つを分離して管理する。

- Project metadata
- Kairi管理状態
- Application管理状態

Project全体がdirtyかどうかは、この3状態の論理和とする。

各ownerの保存が成功するまで対応するdirty stateをclearしない。

## 2.11 New Project

New ProjectではProject metadataとKairi管理状態を生成し、`ApplicationProjectAdapter`へApplication dataの初期化を要求する。

新規Projectは保存場所を即時決定する方式と、未保存状態で開始して後から保存場所を決定する方式を扱える。

保存場所を選択する必要がある場合、v0.1.0ではKairiが標準Project dialogを提供する。

## 2.12 Open Project

Openでは`project.toml`を先に読み、Project formatとApplication IDを検証する。

その後、Kairi管理状態を読み込み、`ApplicationProjectAdapter`へApplication dataのcompatibilityとconsistencyを問い合わせる。

Kairi管理状態の欠損・不整合、Application dataの変換・非互換・不整合、未完了のResource operation等により利用者判断が必要な場合は、Openを即時完了せず判断可能な状態を上位UIへ返す。

利用者が継続を承認し、Open可能条件を満たした後にCurrent Projectとして確定する。

Open対象のProject Rootを選択する操作にはKairi標準Project dialogを使用する。

## 2.13 Save

SaveではKairi管理状態とApplication管理状態を同一Projectとして協調して保存する。

概念上、次の順序で保存する。

1. Application dataを保存する。
2. Applicationから実際に保存したdata versionを受領する。
3. Kairi管理状態を保存する。
4. `project.toml`を最後に保存する。
5. 全処理成功後にdirty stateをclearする。

Project全体を一括rollbackするtransactionは要求しない。途中失敗時はdirty stateを維持する。

未保存Projectで保存場所が必要な場合はKairi標準Project dialogで保存先を決定する。

## 2.14 Save As

Save Asでは新しいProject RootへProjectを保存し、全処理成功後にのみCurrent Project Rootを切り替える。

Application dataはKairiがblind copyせず、`ApplicationProjectAdapter`のSave As境界を使用する。

Project ResourceはKairiが新しいProject Rootへ複製する。

保存先Project Rootの選択にはKairi標準Project dialogを使用する。

## 2.15 Close Project

Projectがdirtyの場合、Closeは確認なしにProject stateを破棄しない。

dirty confirmationを通過し、Project sessionを実際に破棄した後にClose完了とする。

Application終了時にもCurrent Projectのdirty stateを同じ規則で扱う。

## 2.16 Project lifecycle Event

利用アプリケーションへ次のProject lifecycle Eventを通知する。

- `Created`
- `Opened`
- `Saved`
- `SavedAs`
- `CloseRequested`
- `Closed`

Project operationが失敗した場合、対応する成功Eventは通知しない。

Openで利用者判断が必要な間は`Opened`を通知せず、Openが実際に成立した後に通知する。

Closeでは`CloseRequested`と`Closed`を分離し、dirty confirmation中は`Closed`を通知しない。

## 2.17 Project dialog

v0.1.0では、Kairi自身のProject lifecycleに必要な標準Project dialogをKairiが提供する。

対象は少なくとも次の操作とする。

- New Projectで保存場所を必要とする場合
- Open Project
- 未保存ProjectのSave
- Save As

Project dialogはProject Rootを選択するためのKairi内部UIであり、利用アプリケーションへ汎用file/resource selectorとして公開するPublic I/Fとは区別する。

利用アプリケーションへ提供する汎用dialog Public I/Fはv0.1.0の必須範囲に含めず、v0.2.0以降の対象とする。

## 2.18 Project Properties

KairiはProject name / descriptionを編集する標準Project Properties UIを提供する。

利用アプリケーションが独自Project Properties UIを使用する場合は、KairiからProject情報を受け取り、画面全体を利用アプリケーション側で構成できる。

独自UIを使用する場合も、Project metadataの所有者はKairiとする。

## 2.19 Public I/F

本章の公開型、method、Event payload、戻り値および厳密な発火条件は[Application / Project API Reference](Reference/01_Application_Project_API.md)で定義する。

Application levelのMenu / Command、Status、Application lifecycle Event、Project lifecycle Eventの設計上の背景は[Application-level Public I/F設計](02_Application_Public_IF_設計.md)を参照する。

利用アプリケーションはKairi内部実装を直接変更せず、本章から参照されるPublic I/Fを通じてApplication / Project機能を利用する。

---

[目次](../目次.md) > [仕様目次](./仕様目次.md) > 第2章 Application / Project
