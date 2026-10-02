# Application / Project API Reference

## 1. 目的

Kairi 機能仕様 第2章 Application / Project から使用する Public API の詳細参照先である。

本書では develop の現行実装に存在する名前を記載する。未実装の lifecycle event 等には仮の API 名を付けない。

## 2. Application

### Application::new

```rust
pub fn new(id: impl Into<String>, name: impl Into<String>) -> Application
```

Application ID と表示名から Application builder を生成する。

### Application::with_config

```rust
pub fn with_config(config: ApplicationConfig) -> Application
```

ApplicationConfig を明示して生成する。

### Application::version

```rust
pub fn version(self, version: impl Into<String>) -> Application
```

About 等で使用する Consumer Application version を設定する。

### Application::project_adapter

```rust
pub fn project_adapter<A>(self, adapter: A) -> Application
where
    A: ApplicationProjectAdapter + 'static
```

Application 固有 Project lifecycle 実装を登録する。

### Application::about_renderer

Application 独自 About UI を登録する。renderer には `FrameworkInfo` と open state が渡される。

### Application::project_properties_renderer

Application 独自 Project Properties UI を登録する。未指定時は Framework 標準 UI を使用する。

### Application::run

```rust
pub fn run(self) -> eframe::Result<()>
```

設定済み Application を Kairi host として起動する。

## 3. ApplicationConfig

現行公開 field:

- `id: String`
- `name: String`
- `version: Option<String>`
- `window: WindowConfig`
- `appearance: AppearanceConfig`
- `panels: Vec<PanelDefinition>`
- `logging: LoggingConfig`
- `layout: Option<LayoutConfig>`
- Framework standard panel enable flags
- `localization: LocalizationConfig`

Workspace導入に伴う構造変更は第3章仕様で扱う。

## 4. FrameworkInfo

`FrameworkInfo::current()` は現在のUI言語での Framework product name と `FRAMEWORK_VERSION` を返す。

## 5. ApplicationProjectAdapter

```rust
pub trait ApplicationProjectAdapter {
    type Error: std::fmt::Display;

    fn initialize_project(&mut self, context: &ProjectContext)
        -> Result<(), Self::Error>;

    fn inspect_project_data(
        &mut self,
        context: &ProjectContext,
        stored_data_version: Option<&str>,
    ) -> Result<ProjectDataCompatibility, Self::Error>;

    fn check_project_consistency(
        &mut self,
        context: &ProjectContext,
    ) -> Result<ProjectDataConsistency, Self::Error>;

    fn save_project_data(
        &mut self,
        context: &ProjectContext,
        save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error>;

    fn save_project_data_as(
        &mut self,
        source: Option<&ProjectContext>,
        destination: &ProjectContext,
        save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error>;
}
```

## 6. ProjectContext

Project Root を保持し、次の標準 path を提供する。

- `project_file_path()`
- `framework_directory()`
- `framework_settings_path()`
- `resource_root()`
- `application_directory()`
- `root()`

## 7. ProjectController

現行実装は次の Project lifecycle operation を持つ。

- `new_project()`
- `open()`
- `accept_open()`
- `save()`
- `save_as()`
- `close()`
- `is_open()`
- `project_name()`
- `project_description()`
- `update_project_metadata()`

通常の Consumer Application は Framework UI を利用する場合、これらを直接駆動する必要はない。Project lifecycle の Framework/Application境界は `ApplicationProjectAdapter` とする。

## 8. ProjectCommandResult

現行 result:

- `Completed`
- `NeedsSaveLocation`
- `NeedsOpenDecision(ProjectOpenResult)`
- `NeedsDirtyConfirmation`
- `Failed(String)`

## 9. Project file metadata

`ProjectFile` は `ProjectMetadata` と `ApplicationMetadata` を持つ。

Application ID は Project Open 時の compatibility gate として使用する。

## 10. Event

Application startup / started / closing、Project opened / closing / closed 等を Consumerへ通知する独立 Public Event API は現行 develop では未定義である。

Event名、payload、発火順序を確定する際は本Referenceへ追加し、第2章のsequenceから参照する。
