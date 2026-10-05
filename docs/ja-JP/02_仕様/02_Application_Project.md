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

## 2.1 この章の到達点

本章までの仕様を利用することで、Consumer Application は Kairi を組み込み、Application を起動し、基本 Menu、About / Version 表示および Project の New / Open / Save / Save As / Close を持つ基本的な IDE 型 Application を構築できることを目標とする。

Workspace と Application 固有 Panel の詳細は第3章以降で扱う。

## 2.2 Kairi の導入概念

Consumer Application は Kairi の `Application` を生成し、Application 固有設定と Project Adapter を登録した後、`run()` に Application の実行を委譲する。

現行実装での最小構成は次の形である。

```rust
Application::new("org.example.application", "Example Application")
    .version(env!("CARGO_PKG_VERSION"))
    .project_adapter(ApplicationProject)
    .run()
```

Application ID は Project compatibility 判定にも使用する stable opaque ID とし、表示名とは分離する。

## 2.3 Application から見た起動シーケンス

Application 開発者から見た基本 sequence は次の通りとする。

```text
Consumer main()
    │
    ├─ Application::new() / with_config()
    │
    ├─ Application metadata 設定
    │    └─ version()
    │
    ├─ 必要な Framework 機能を設定
    │
    ├─ project_adapter()
    │
    ├─ Workspace / Panel 設定      ← 第3・4章
    │
    └─ run()
         │
         ├─ Kairi localization 初期化
         ├─ Kairi logging 初期化
         ├─ Layout / Window 構築
         ├─ Theme / Font 適用
         ├─ ProjectController 構築
         └─ Application UI loop 開始
```

現行実装では `run()` 内で eframe native application を開始する。Consumer Application が Kairi 内部の eframe lifecycle を直接管理することは基本契約としない。

Application startup / started 等を Consumer に通知する独立 Public Event I/F は現行実装にはまだ定義されていない。必要な Event 名と callback timing は Event Reference の整備時に確定する。

## 2.4 Application metadata

Application は少なくとも次の情報を持つ。

| 項目 | 用途 |
|---|---|
| Application ID | stable identifier。Project compatibility 判定に使用 |
| Application Name | UI表示名 |
| Application Version | About / Version 表示 |
| Description | Project metadata等で必要に応じて使用 |
| Language | Application / Project metadataで使用 |

現行 `ApplicationConfig` は `id`、`name`、`version` を持つ。Project file の `ApplicationMetadata` は `id`、`name`、`description`、`language`、`data_version` を持つ。

## 2.5 Menu

Kairi は Application level の Menu 基盤を提供する。

現行実装では Project Adapter を登録した Application に対し、File Menu から Project lifecycle 操作を提供する。

- New Project
- Open Project
- Save
- Save As
- Project Properties
- Close Project
- Exit

Help Menu から About を表示する。

Menu item の有効/無効は Current Project の有無等の Framework state に応じて制御する。

Application 固有 Menu / Command の一般公開登録 I/F は、本章の最終仕様として今後整理する。現行実装に存在しない API 名を本仕様では仮定しない。

## 2.6 Status 表示

Status は Application level の要素として扱う。

Application / Project の状態、処理状態、通知等を Application 全体から確認できる領域を想定するが、現行 v0.1.0 develop には Consumer 向けの汎用 Status registration API はまだ確定していない。

Status の ownership、表示位置、Application 固有 status item の登録 I/F は Application level API として定義する。

## 2.7 About / Version

Kairi は標準 About 表示を提供する。

Application が `version()` で Version を設定した場合、標準 About は Application name / version と Kairi の Framework name / version を表示する。

Application が独自 About UI を必要とする場合は `about_renderer()` を使用できる。renderer には `FrameworkInfo` が渡されるため、独自画面でも Kairi の name / version を表示できる。

Application が独自 renderer を提供しない場合でも標準 About が利用可能であることを基本動作とする。

## 2.8 Project の位置づけ

Project は Application level の管理データ単位とする。

```text
<Project Root>/
├─ project.toml
├─ framework/
│  └─ framework_settings.toml
├─ resources/
└─ application/
```

Kairi は `project.toml`、`framework/` および Project lifecycle を所有する。

Consumer Application は `application/` 以下の Application 固有 data を所有する。Kairi は Application data の意味、migration、内部保存形式を所有しない。

## 2.9 Project Adapter

Consumer Application は `ApplicationProjectAdapter` を実装して Kairi に登録する。

Adapter は次の責務境界を持つ。

- 新規 Project の Application data 初期化
- 保存済み Application data の compatibility 判定
- Application data の consistency 判定
- Save
- Save As

詳細な trait 定義は [Application / Project API Reference](Reference/01_Application_Project_API.md) を参照する。

## 2.10 New Project

New Project は、Project metadata と Framework state を新規作成し、Application Project Adapter に Application data 初期化を要求する。

保存場所は Application の方針により即時必須または後決めを許容できる。現行 UI は `NewProjectStoragePolicy::Deferred` を使用し、未保存 Project を作成できる。

新規 Project は metadata / framework / application の各 dirty state を持つ。

## 2.11 Open Project

Open は最初に `project.toml` を読み、Project format と Application ID を検証する。

Application ID が現在の Application と異なる Project は、その Application の Project として Open しない。

Kairi は Application Project Adapter に compatibility / consistency を問い合わせる。変換や recovery の判断を Framework が Application に代わって推測しない。

User decision が必要な場合は即時 Open 完了とせず、判断可能な状態を上位 UI に返す。

## 2.12 Save / Save As

Save は概念上、次の順序で行う。

1. Application data を保存
2. Application から保存済み data version を受領
3. Framework settings を保存
4. `project.toml` を最後に保存
5. 全処理成功後に dirty state を clear

途中失敗時は dirty state を維持する。

Save As は新しい Project Root に保存し、全処理成功後に Current Project Root を切り替える。Application data は Kairi が blind copy せず、Application Project Adapter の Save As 境界を使用する。

## 2.13 Close / Application 終了

Project が dirty の場合、Close は確認なしに Project state を破棄しない。

現行 `ProjectController::close()` は dirty confirmation の要否を結果として返す。

Application 終了時にも Current Project の dirty state を考慮する。Application Closing / Project Closing 等の Consumer 向け Public Event I/F は現行実装では未確定であり、Event Reference で確定する。

## 2.14 Project Properties

Kairi は Project name / description を編集する標準 Project Properties UI を提供する。

Application 固有設定を同じ画面で扱う必要がある場合、Application は `project_properties_renderer()` で画面全体を所有できる。

独自 renderer を指定しない場合は Kairi の標準 UI を使用する。

## 2.15 Application / Project の主要 I/F

本章で Application 開発者が直接扱う主要な実装済み I/F は次の通りである。

- `Application::new()`
- `Application::with_config()`
- `Application::version()`
- `Application::project_adapter()`
- `Application::command()`
- `Application::menu_item()`
- `Application::status_item()`
- `Application::status_handle()`
- `Application::on_application_event()`
- `Application::on_project_event()`
- `Application::about_renderer()`
- `Application::project_properties_renderer()`
- `Application::run()`
- `ApplicationProjectAdapter`
- `ProjectContext`
- `ProjectController`
- `ProjectCommandResult`
- `ProjectDataCompatibility`
- `ProjectDataConsistency`
- `NewProjectStoragePolicy`

型・method・戻り値の詳細は [Application / Project API Reference](Reference/01_Application_Project_API.md) に集約する。

## 2.16 第2章 Public I/F の確定範囲

Application level の次の I/F は develop で実装済みとする。

- Consumer Application 固有 Menu / Command
- Consumer Application Status
- Application lifecycle Event
- Project lifecycle Event

Project `Opened` は Open が実際に完了した場合だけ通知する。`NeedsOpenDecision` 中は通知せず、`accept_open()` 成功後に通知する。

Project Close は `CloseRequested` と `Closed` を分離し、dirty confirmation 中は `Closed` を通知しない。

Application `Closing` は Project dirty confirmation 等を通過して終了を確定した時点で通知する。

詳細は [Application / Project API Reference](Reference/01_Application_Project_API.md) を参照する。

---

[目次](../目次.md) > [仕様目次](./仕様目次.md) > 第2章 Application / Project
