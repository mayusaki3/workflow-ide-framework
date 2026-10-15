use eframe::egui;
pub mod application_event;
pub mod command;
pub mod controller_panel;
pub mod file_resource_selector;
pub mod flow_editor;
pub mod framework_settings;
pub mod layout;
pub mod locale_font;
pub mod localization;
pub mod locate_replace;
pub mod log_viewer;
pub mod logging;
pub mod probe;
pub mod project;
pub mod project_adapter;
pub mod project_adapter_erased;
pub mod project_controller;
pub mod project_event;
pub mod project_io;
pub mod project_lifecycle;
pub mod project_open;
pub mod project_panel;
pub mod project_panel_api;
pub mod project_resource;
pub mod project_save;
pub mod project_save_as;
pub mod property_panel;
pub mod recent_projects;
pub mod resource_journal;
pub mod resource_operation;
pub mod resource_registry;
pub mod resource_selection_registry;
pub mod resource_state;
pub mod status;
pub mod table;
pub mod text_editor;
pub mod theme;
pub mod tree_viewer;
pub mod unsaved_notification;
pub mod workspace;
pub mod workspace_dock;
pub use layout::{LayoutConfig, LayoutSplit, SplitDirection};
pub use tracing;

pub const FRAMEWORK_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
pub struct FrameworkInfo {
    /// Localized Framework product name for the current UI language.
    pub name: String,
    pub version: &'static str,
}

impl FrameworkInfo {
    pub fn current() -> Self {
        Self {
            name: localization::text("framework.product_name"),
            version: FRAMEWORK_VERSION,
        }
    }
}

/// Consumer-owned About UI. The Consumer decides the window, layout, labels,
/// and close interaction. Set `open` to false when its About UI is closed.
pub type AboutRenderer = Box<dyn FnMut(&egui::Context, FrameworkInfo, &mut bool)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectPropertiesMode {
    Create,
    Edit,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectPropertiesData {
    pub name: String,
    pub description: String,
}

/// Consumer-owned Project Properties UI. When supplied, the Consumer owns the
/// entire screen, including Framework fields such as project name/description.
/// Set commit to true to accept the edited data. Set open to false to cancel.
pub type ProjectPropertiesRenderer = Box<
    dyn FnMut(
        &egui::Context,
        ProjectPropertiesMode,
        &mut ProjectPropertiesData,
        &mut bool,
        &mut bool,
    ),
>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelKind {
    StandardUi,
    GpuViewport,
    Browser,
}

#[derive(Debug, Clone)]
pub struct PanelDefinition {
    pub id: String,
    /// Localization resource key used for the panel title.
    pub title_key: String,
    pub kind: PanelKind,
    pub initially_visible: bool,
}

impl PanelDefinition {
    pub fn new(id: impl Into<String>, title_key: impl Into<String>, kind: PanelKind) -> Self {
        Self {
            id: id.into(),
            title_key: title_key.into(),
            kind,
            initially_visible: true,
        }
    }

    pub fn initially_visible(mut self, visible: bool) -> Self {
        self.initially_visible = visible;
        self
    }
}

fn project_window_title(application_title: &str, project_name: &str, dirty: bool) -> String {
    format!(
        "{application_title} - {project_name}{}",
        if dirty { " *" } else { "" }
    )
}

#[cfg(test)]
mod project_title_tests {
    use super::project_window_title;

    #[test]
    fn dirty_project_has_unsaved_marker() {
        assert_eq!(
            project_window_title("App", "Project", true),
            "App - Project *"
        );
    }

    #[test]
    fn saved_project_has_no_unsaved_marker() {
        assert_eq!(
            project_window_title("App", "Project", false),
            "App - Project"
        );
    }
}

#[derive(Debug, Clone)]
pub struct ApplicationConfig {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub user_config_directory_name: Option<String>,
    pub window: WindowConfig,
    pub appearance: AppearanceConfig,
    pub panels: Vec<PanelDefinition>,
    pub logging: logging::LoggingConfig,
    pub layout: Option<LayoutConfig>,
    pub probe_panel: bool,
    pub logging_settings_panel: bool,
    pub language_settings_panel: bool,
    pub theme_settings_panel: bool,
    pub localization: localization::LocalizationConfig,
}

impl ApplicationConfig {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: None,
            user_config_directory_name: None,
            window: WindowConfig::default(),
            appearance: AppearanceConfig::default(),
            panels: Vec::new(),
            logging: logging::LoggingConfig::default(),
            layout: None,
            probe_panel: false,
            logging_settings_panel: false,
            language_settings_panel: false,
            theme_settings_panel: false,
            localization: localization::LocalizationConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct WindowConfig {
    pub title: Option<String>,
    pub initial_size: Option<[f32; 2]>,
    pub min_size: Option<[f32; 2]>,
}

#[derive(Debug, Clone)]
pub struct AppearanceConfig {
    pub ui_scale: Option<f32>,
    /// Optional application font file. When set, WFIDE loads it into egui
    /// before the first frame and gives it priority for proportional text.
    pub font_path: Option<std::path::PathBuf>,
    pub theme: theme::Theme,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            ui_scale: None,
            font_path: None,
            theme: theme::Theme::System,
        }
    }
}

#[derive(Debug, Clone)]
struct FlowPropertyLink {
    flow_panel_id: String,
    property_panel_id: String,
}

#[derive(Debug, Clone)]
struct ControllerPropertyLink {
    controller_panel_id: String,
    property_panel_id: String,
}

pub struct Application {
    unsaved_notification: unsaved_notification::UnsavedNotification,
    config: ApplicationConfig,
    text_editors: std::collections::HashMap<String, text_editor::TextDocument>,
    text_editor_options: std::collections::HashMap<String, text_editor::TextEditorOptions>,
    log_viewers: std::collections::HashMap<String, log_viewer::LogViewerOptions>,
    tree_viewers: std::collections::HashMap<String, tree_viewer::TreeModel>,
    project_panel_provider: Option<Box<dyn project_panel_api::ApplicationProjectPanelProvider>>,
    project_panel_registry: Option<Box<dyn Fn(&project::ProjectContext) -> resource_registry::ResourceRegistry>>,
    project_panel_id: Option<String>,
    flow_editors: std::collections::HashMap<String, flow_editor::FlowModel>,
    property_panels: std::collections::HashMap<String, property_panel::PropertyModel>,
    flow_property_links: Vec<FlowPropertyLink>,
    controller_panels: std::collections::HashMap<String, controller_panel::ControllerModel>,
    controller_property_links: Vec<ControllerPropertyLink>,
    project_adapter: Option<Box<dyn project_adapter_erased::ErasedApplicationProjectAdapter>>,
    about_renderer: Option<AboutRenderer>,
    project_properties_renderer: Option<ProjectPropertiesRenderer>,
    commands: command::CommandRegistry,
    menu_items: Vec<command::MenuItemDefinition>,
    status: status::StatusHandle,
    application_events: application_event::ApplicationEventDispatcher,
    project_events: project_event::ProjectEventDispatcher,
}

impl Application {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::with_config(ApplicationConfig::new(id, name))
    }

    pub fn with_config(config: ApplicationConfig) -> Self {
        Self {
            unsaved_notification: unsaved_notification::UnsavedNotification::default(),
            config,
            text_editors: std::collections::HashMap::new(),
            text_editor_options: std::collections::HashMap::new(),
            log_viewers: std::collections::HashMap::new(),
            tree_viewers: std::collections::HashMap::new(),
            project_panel_provider: None,
            project_panel_registry: None,
            project_panel_id: None,
            flow_editors: std::collections::HashMap::new(),
            property_panels: std::collections::HashMap::new(),
            flow_property_links: Vec::new(),
            controller_panels: std::collections::HashMap::new(),
            controller_property_links: Vec::new(),
            project_adapter: None,
            about_renderer: None,
            project_properties_renderer: None,
            commands: command::CommandRegistry::default(),
            menu_items: Vec::new(),
            status: status::StatusHandle::new(),
            application_events: application_event::ApplicationEventDispatcher::default(),
            project_events: project_event::ProjectEventDispatcher::default(),
        }
    }

    /// Cloneable notification handle for consumer-owned editors and panels.
    pub fn unsaved_notification(&self) -> unsaved_notification::UnsavedNotification {
        self.unsaved_notification.clone()
    }

    pub fn text_editor_panel(
        mut self,
        panel_id: impl Into<String>,
        document: text_editor::TextDocument,
    ) -> Self {
        let panel_id = panel_id.into();
        self.text_editors.insert(panel_id.clone(), document);
        self.text_editor_options.entry(panel_id).or_default();
        self
    }

    pub fn text_editor_ime_debug(mut self, panel_id: impl Into<String>, enabled: bool) -> Self {
        self.text_editor_options
            .entry(panel_id.into())
            .or_default()
            .ime_debug = enabled;
        self
    }

    pub fn log_viewer_panel(mut self, panel_id: impl Into<String>) -> Self {
        self.log_viewers.entry(panel_id.into()).or_default();
        self
    }

    pub fn tree_viewer_panel(
        mut self,
        panel_id: impl Into<String>,
        model: tree_viewer::TreeModel,
    ) -> Self {
        self.tree_viewers.insert(panel_id.into(), model);
        self
    }

    /// Register an application-owned Project Panel and its registry snapshot source.
    pub fn application_project_panel(
        mut self,
        panel_id: impl Into<String>,
        provider: impl project_panel_api::ApplicationProjectPanelProvider + 'static,
        registry: impl Fn(&project::ProjectContext) -> resource_registry::ResourceRegistry + 'static,
    ) -> Self {
        let panel_id = panel_id.into();
        self.tree_viewers.entry(panel_id.clone()).or_default();
        self.project_panel_id = Some(panel_id);
        self.project_panel_provider = Some(Box::new(provider));
        self.project_panel_registry = Some(Box::new(registry));
        self
    }

    pub fn flow_editor_panel(
        mut self,
        panel_id: impl Into<String>,
        model: flow_editor::FlowModel,
    ) -> Self {
        self.flow_editors.insert(panel_id.into(), model);
        self
    }

    pub fn property_panel(
        mut self,
        panel_id: impl Into<String>,
        model: property_panel::PropertyModel,
    ) -> Self {
        self.property_panels.insert(panel_id.into(), model);
        self
    }

    pub fn controller_panel(
        mut self,
        panel_id: impl Into<String>,
        model: controller_panel::ControllerModel,
    ) -> Self {
        self.controller_panels.insert(panel_id.into(), model);
        self
    }

    /// Reference adapter linking Controller layout selection/editing to a Property Panel.
    pub fn link_controller_properties(
        mut self,
        controller_panel_id: impl Into<String>,
        property_panel_id: impl Into<String>,
    ) -> Self {
        self.controller_property_links.push(ControllerPropertyLink {
            controller_panel_id: controller_panel_id.into(),
            property_panel_id: property_panel_id.into(),
        });
        self
    }

    /// Reference adapter linking Flow selection/editing to a Property Panel.
    /// Consumer domain properties should use Consumer/Application state instead.
    pub fn link_flow_properties(
        mut self,
        flow_panel_id: impl Into<String>,
        property_panel_id: impl Into<String>,
    ) -> Self {
        self.flow_property_links.push(FlowPropertyLink {
            flow_panel_id: flow_panel_id.into(),
            property_panel_id: property_panel_id.into(),
        });
        self
    }

    /// Attach the Consumer Project lifecycle implementation to the Framework host.
    pub fn project_adapter<A>(mut self, adapter: A) -> Self
    where
        A: project_adapter::ApplicationProjectAdapter + 'static,
    {
        self.project_adapter = Some(Box::new(adapter));
        self
    }

    /// Set the Consumer/Application version shown by the Framework About dialog.
    /// Select the application's user settings folder, independent of its stable Application ID.
    /// Only ASCII letters, digits, hyphens and underscores are accepted.
    pub fn user_config_directory_name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        assert!(
            recent_projects::directory_name(&self.config.id, Some(&name)).is_some(),
            "invalid user config directory name"
        );
        self.config.user_config_directory_name = Some(name);
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.config.version = Some(version.into());
        self
    }

    /// Let the Consumer/Application render the About contents. Framework metadata is supplied on each render.
    pub fn about_renderer<F>(mut self, renderer: F) -> Self
    where
        F: FnMut(&egui::Context, FrameworkInfo, &mut bool) + 'static,
    {
        self.about_renderer = Some(Box::new(renderer));
        self
    }

    /// Let the Consumer/Application own the complete Project Properties UI.
    /// If omitted, Framework renders its standard name/description screen.
    pub fn project_properties_renderer<F>(mut self, renderer: F) -> Self
    where
        F: FnMut(
                &egui::Context,
                ProjectPropertiesMode,
                &mut ProjectPropertiesData,
                &mut bool,
                &mut bool,
            ) + 'static,
    {
        self.project_properties_renderer = Some(Box::new(renderer));
        self
    }

    /// Register a Consumer command. Duplicate IDs are rejected immediately.
    pub fn command(mut self, command: command::CommandDefinition) -> Self {
        self.commands
            .register(command)
            .expect("duplicate Kairi command id");
        self
    }

    /// Place a registered Consumer command in File, Help, or a custom top-level menu.
    pub fn menu_item(mut self, item: command::MenuItemDefinition) -> Self {
        self.menu_items.push(item);
        self
    }

    /// Return a cloneable status handle that remains usable while the application is running.
    pub fn status_handle(&self) -> status::StatusHandle {
        self.status.clone()
    }

    /// Register an initial application-level status item.
    pub fn status_item(self, item: status::StatusItem) -> Self {
        self.status
            .register(item)
            .expect("duplicate Kairi status id");
        self
    }

    /// Subscribe to Consumer-visible application lifecycle events.
    pub fn on_application_event<F>(mut self, handler: F) -> Self
    where
        F: FnMut(application_event::ApplicationEvent) + 'static,
    {
        self.application_events.subscribe(handler);
        self
    }

    /// Subscribe to Consumer-visible project lifecycle events.
    pub fn on_project_event<F>(mut self, handler: F) -> Self
    where
        F: FnMut(&project_event::ProjectEvent) + 'static,
    {
        self.project_events.subscribe(handler);
        self
    }

    pub fn panel(mut self, panel: PanelDefinition) -> Self {
        self.config.panels.push(panel);
        self
    }

    pub fn layout(mut self, layout: LayoutConfig) -> Self {
        self.config.layout = Some(layout);
        self
    }

    pub fn framework_probe_panel(mut self) -> Self {
        self.config.probe_panel = true;
        self
    }

    pub fn logging_settings_panel(mut self) -> Self {
        self.config.logging_settings_panel = true;
        self
    }

    /// Set the initial Framework/Consumer tracing level before logging is initialized.
    /// Add application-owned localization resources. Files use the same locale TOML format as Framework resources.
    pub fn localization_resources(mut self, directory: impl Into<std::path::PathBuf>) -> Self {
        self.config
            .localization
            .application_resource_directories
            .push(directory.into());
        self
    }

    pub fn log_level(mut self, level: logging::LogLevel) -> Self {
        self.config.logging.level = level;
        self
    }

    pub fn language_settings_panel(mut self) -> Self {
        self.config.language_settings_panel = true;
        self
    }

    pub fn theme_settings_panel(mut self) -> Self {
        self.config.theme_settings_panel = true;
        self
    }

    /// Load an application font before the first frame.
    /// The font is installed for both proportional and monospace egui text.
    pub fn font_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.config.appearance.font_path = Some(path.into());
        self
    }

    pub fn run(self) -> eframe::Result<()> {
        let mut config = self.config;
        let unsaved_notification = self.unsaved_notification;
        let text_editors = self.text_editors;
        let text_editor_options = self.text_editor_options;
        let log_viewers = self.log_viewers;
        let tree_viewers = self.tree_viewers;
        let project_panel_provider = self.project_panel_provider;
        let project_panel_registry = self.project_panel_registry;
        let project_panel_id = self.project_panel_id;
        let flow_editors = self.flow_editors;
        let property_panels = self.property_panels;
        let flow_property_links = self.flow_property_links;
        let controller_panels = self.controller_panels;
        let controller_property_links = self.controller_property_links;
        let project_adapter = self.project_adapter;
        let about_renderer = self.about_renderer;
        let project_properties_renderer = self.project_properties_renderer;
        let commands = self.commands;
        let menu_items = self.menu_items;
        let status = self.status;
        let mut application_events = self.application_events;
        let project_events = self.project_events;
        application_events.emit(application_event::ApplicationEvent::Starting);
        if config.probe_panel
            && !config
                .panels
                .iter()
                .any(|panel| panel.id == probe::PANEL_ID)
        {
            config.panels.push(PanelDefinition::new(
                probe::PANEL_ID,
                "WFIDE Probe",
                PanelKind::StandardUi,
            ));
            if let Some(layout) = &mut config.layout {
                layout.root_panel_ids.push(probe::PANEL_ID.to_owned());
            }
        }
        if config.logging_settings_panel
            && !config
                .panels
                .iter()
                .any(|panel| panel.id == "__wfide_logging_settings")
        {
            config.panels.push(PanelDefinition::new(
                "__wfide_logging_settings",
                "logging.title",
                PanelKind::StandardUi,
            ));
            if let Some(layout) = &mut config.layout {
                layout
                    .root_panel_ids
                    .push("__wfide_logging_settings".to_owned());
            }
        }
        if config.language_settings_panel
            && !config
                .panels
                .iter()
                .any(|panel| panel.id == "__wfide_language_settings")
        {
            config.panels.push(PanelDefinition::new(
                "__wfide_language_settings",
                "language.title",
                PanelKind::StandardUi,
            ));
            if let Some(layout) = &mut config.layout {
                layout
                    .root_panel_ids
                    .push("__wfide_language_settings".to_owned());
            }
        }
        if config.theme_settings_panel
            && !config
                .panels
                .iter()
                .any(|panel| panel.id == "__wfide_theme_settings")
        {
            config.panels.push(PanelDefinition::new(
                "__wfide_theme_settings",
                "theme.title",
                PanelKind::StandardUi,
            ));
            if let Some(layout) = &mut config.layout {
                layout
                    .root_panel_ids
                    .push("__wfide_theme_settings".to_owned());
            }
        }
        // A Consumer that only enables Framework-provided panels should still
        // get a usable dock. Without this fallback, declared panels were shown
        // only as diagnostic text because no DockState was constructed.
        if config.layout.is_none() {
            let visible = config
                .panels
                .iter()
                .filter(|panel| panel.initially_visible)
                .map(|panel| panel.id.clone())
                .collect::<Vec<_>>();
            if !visible.is_empty() {
                config.layout = Some(LayoutConfig::new(visible));
            }
        }
        localization::init(&config.localization);
        let _logging_guard =
            logging::init(&config.id, &config.logging).map_err(eframe::Error::AppCreation)?;
        tracing::info!(target: "wfide::application", application_id = %config.id, "WFIDE application starting");
        // Workspace is the authoritative placement model. Legacy layout roots
        // are imported as panel instances; dock trees are rendering projections.
        let mut workspace_registry = workspace::WorkspaceRegistry::new();
        let initial_panel_ids = config
            .layout
            .as_ref()
            .map(|layout| {
                layout
                    .root_panel_ids
                    .iter()
                    .cloned()
                    .chain(
                        layout
                            .splits
                            .iter()
                            .flat_map(|split| split.panel_ids.iter().cloned()),
                    )
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        workspace_dock::import_legacy_root_panels(&mut workspace_registry, &initial_panel_ids)
            .map_err(|error| {
                eframe::Error::AppCreation(Box::new(std::io::Error::other(format!("{error:?}"))))
            })?;
        let mut workspace_docks = workspace_dock::project_workspace(
            workspace_registry
                .get(workspace::DEFAULT_WORKSPACE_ID)
                .expect("default workspace"),
        )
        .map_err(|error| {
            eframe::Error::AppCreation(Box::new(std::io::Error::other(format!("{error:?}"))))
        })?;
        if let Some(layout) = &config.layout {
            workspace_dock::apply_initial_layout(&mut workspace_docks, layout).map_err(
                |error| {
                    eframe::Error::AppCreation(Box::new(std::io::Error::other(format!(
                        "{error:?}"
                    ))))
                },
            )?;
        }
        let window_title = config
            .window
            .title
            .clone()
            .unwrap_or_else(|| config.name.clone());

        let mut viewport = egui::ViewportBuilder::default();
        if let Some([width, height]) = config.window.initial_size {
            viewport = viewport.with_inner_size([width, height]);
        }
        if let Some([width, height]) = config.window.min_size {
            viewport = viewport.with_min_inner_size([width, height]);
        }

        let native_options = eframe::NativeOptions {
            viewport,
            ..Default::default()
        };

        eframe::run_native(
            &window_title,
            native_options,
            Box::new(move |cc| {
                apply_theme_mode(&cc.egui_ctx, config.appearance.theme);
                if let Some(scale) = config.appearance.ui_scale {
                    cc.egui_ctx.set_zoom_factor(scale);
                }
                let locale = localization::current_locale();
                match locale_font::install_for_locale(
                    &cc.egui_ctx,
                    &locale,
                    config.appearance.font_path.as_deref(),
                ) {
                    Ok(paths) if !paths.is_empty() => {
                        tracing::info!(target: "wfide::font", %locale, count = paths.len(), "fallback fonts loaded")
                    }
                    Ok(_) => {
                        tracing::warn!(target: "wfide::font", %locale, "no OS or locale fallback font found")
                    }
                    Err(error) => {
                        tracing::warn!(target: "wfide::font", %locale, %error, "failed to configure fonts")
                    }
                }

                let project_controller = project_adapter.as_ref().map(|_| {
                    project_controller::ProjectController::new(
                        config.id.clone(),
                        config.name.clone(),
                    )
                });

                let recent_projects = recent_projects::settings_path(
                    &config.id,
                    config.user_config_directory_name.as_deref(),
                )
                .map(|path| recent_projects::RecentProjects::load(&path))
                .unwrap_or_default();
                Ok(Box::new(FrameworkHost {
                    config,
                    unsaved_revision_seen: unsaved_notification.revision(),
                    unsaved_notification,
                    workspace_registry,
                    workspace_docks,
                    workspace_docks_by_id: std::collections::BTreeMap::new(),
                    active_workspace_dock_id: workspace::DEFAULT_WORKSPACE_ID.to_owned(),
                    text_editors,
                    text_editor_options,
                    log_viewers,
                    tree_viewers,
                    project_panel_provider,
                    project_panel_registry,
                    project_panel_id,
                    project_panel_lifecycle: project_panel_api::ProjectPanelLifecycle::default(),
                    flow_editors,
                    property_panels,
                    flow_property_links,
                    controller_panels,
                    controller_property_links,
                    project_controller,
                    project_adapter,
                    about_renderer,
                    project_properties_renderer,
                    commands,
                    menu_items,
                    status,
                    application_events,
                    project_events,
                    project_ui: ProjectUiState::default(),
                    recent_projects,
                    theme_editor: None,
                    fonts_initialized: false,
                    started_emitted: false,
                }))
            }),
        )
    }
}

fn apply_theme_mode(ctx: &egui::Context, selected: theme::Theme) {
    match selected.is_dark() {
        Some(true) => {
            ctx.set_theme(egui::ThemePreference::Dark);
            selected.apply_with_system_dark(ctx, true);
        }
        Some(false) => {
            ctx.set_theme(egui::ThemePreference::Light);
            selected.apply_with_system_dark(ctx, false);
        }
        None => {
            ctx.set_theme(egui::ThemePreference::System);
            selected.apply_with_system_dark(
                ctx,
                ctx.system_theme().unwrap_or(egui::Theme::Dark) == egui::Theme::Dark,
            );
        }
    }
}

struct FrameworkHost {
    unsaved_notification: unsaved_notification::UnsavedNotification,
    unsaved_revision_seen: u64,
    config: ApplicationConfig,
    workspace_registry: workspace::WorkspaceRegistry,
    workspace_docks: workspace_dock::WorkspaceDockProjection,
    workspace_docks_by_id:
        std::collections::BTreeMap<String, workspace_dock::WorkspaceDockProjection>,
    active_workspace_dock_id: String,
    text_editors: std::collections::HashMap<String, text_editor::TextDocument>,
    text_editor_options: std::collections::HashMap<String, text_editor::TextEditorOptions>,
    log_viewers: std::collections::HashMap<String, log_viewer::LogViewerOptions>,
    tree_viewers: std::collections::HashMap<String, tree_viewer::TreeModel>,
    project_panel_provider: Option<Box<dyn project_panel_api::ApplicationProjectPanelProvider>>,
    project_panel_registry: Option<Box<dyn Fn(&project::ProjectContext) -> resource_registry::ResourceRegistry>>,
    project_panel_id: Option<String>,
    project_panel_lifecycle: project_panel_api::ProjectPanelLifecycle,
    flow_editors: std::collections::HashMap<String, flow_editor::FlowModel>,
    property_panels: std::collections::HashMap<String, property_panel::PropertyModel>,
    flow_property_links: Vec<FlowPropertyLink>,
    controller_panels: std::collections::HashMap<String, controller_panel::ControllerModel>,
    controller_property_links: Vec<ControllerPropertyLink>,
    project_adapter: Option<Box<dyn project_adapter_erased::ErasedApplicationProjectAdapter>>,
    project_controller: Option<project_controller::ProjectController>,
    about_renderer: Option<AboutRenderer>,
    project_properties_renderer: Option<ProjectPropertiesRenderer>,
    commands: command::CommandRegistry,
    menu_items: Vec<command::MenuItemDefinition>,
    status: status::StatusHandle,
    application_events: application_event::ApplicationEventDispatcher,
    project_events: project_event::ProjectEventDispatcher,
    project_ui: ProjectUiState,
    recent_projects: recent_projects::RecentProjects,
    theme_editor: Option<theme::ThemeEditor>,
    fonts_initialized: bool,
    started_emitted: bool,
}

#[derive(Default)]
struct NewProjectDialogState {
    name: String,
    description: String,
}

struct ProjectPropertiesDialogState {
    mode: ProjectPropertiesMode,
    data: ProjectPropertiesData,
}

enum ProjectFolderDialogMode {
    Open,
    SaveAs,
}

struct ProjectFolderDialogState {
    mode: ProjectFolderDialogMode,
    directory: std::path::PathBuf,
    project_name: String,
    reveal_tree_selection: bool,
    selected_entry: Option<std::path::PathBuf>,
    new_folder_name: String,
    create_folder_parent: Option<std::path::PathBuf>,
    delete_folder_pending: Option<std::path::PathBuf>,
    delete_folder_error: Option<String>,
}

#[derive(Default)]
struct ProjectUiState {
    message: Option<String>,
    new_project: Option<NewProjectDialogState>,
    folder_dialog: Option<ProjectFolderDialogState>,
    pending_open: Option<(project::ProjectContext, project_open::ProjectOpenResult)>,
    confirm_close: bool,
    exit_after_close: bool,
    show_about: bool,
    project_properties: Option<ProjectPropertiesDialogState>,
}

fn project_save_stamp() -> (String, String) {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQUENCE: AtomicU64 = AtomicU64::new(1);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    (
        format!("save-{}-{}", now.as_nanos(), sequence),
        format!("unix:{}.{:09}", now.as_secs(), now.subsec_nanos()),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProjectFolderKind {
    Folder,
    Project,
    OtherApplicationProject(String),
    UnavailableProject,
}

fn project_folder_kind(path: &std::path::Path, application_id: &str) -> ProjectFolderKind {
    let file = path.join("project.toml");
    if !file.is_file() {
        return ProjectFolderKind::Folder;
    }
    let Some(content) = std::fs::read_to_string(file).ok() else {
        return ProjectFolderKind::UnavailableProject;
    };
    if project::ProjectFile::from_toml(&content, application_id).is_ok() {
        return ProjectFolderKind::Project;
    }
    match toml::from_str::<project::ProjectFile>(&content) {
        Ok(file) if file.application.id != application_id => {
            ProjectFolderKind::OtherApplicationProject(file.application.name)
        }
        _ => ProjectFolderKind::UnavailableProject,
    }
}

fn project_browser_visible_directory(_path: &std::path::Path) -> bool {
    // Leading dots are valid folder-name characters (e.g. ".A").
    // On Windows, hide only directories explicitly marked hidden/system.
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
        if let Ok(metadata) = std::fs::metadata(_path) {
            let attributes = metadata.file_attributes();
            if attributes & (FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM) != 0 {
                return false;
            }
        }
    }
    true
}

fn project_browser_directories(path: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut children = std::fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            entry
                .file_type()
                .ok()
                .filter(|kind| kind.is_dir())
                .map(|_| entry.path())
        })
        .filter(|path| project_browser_visible_directory(path))
        .collect::<Vec<_>>();
    children.sort_by_key(|child| {
        child
            .file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .unwrap_or_default()
    });
    children
}

fn project_browser_roots() -> Vec<std::path::PathBuf> {
    #[cfg(windows)]
    {
        (b'A'..=b'Z')
            .filter_map(|letter| {
                let path = std::path::PathBuf::from(format!(
                    "{}:{}",
                    letter as char,
                    std::path::MAIN_SEPARATOR
                ));
                path.is_dir().then_some(path)
            })
            .collect()
    }
    #[cfg(not(windows))]
    {
        vec![std::path::PathBuf::from("/")]
    }
}

// Recursive tree rendering passes independent navigation and context-menu outputs.
#[allow(clippy::too_many_arguments)]
fn show_project_folder_tree(
    ui: &mut egui::Ui,
    path: &std::path::Path,
    selected: &std::path::Path,
    application_id: &str,
    next: &mut Option<std::path::PathBuf>,
    open_project: &mut Option<std::path::PathBuf>,
    reveal_selection: bool,
    create_parent: &mut Option<std::path::PathBuf>,
    delete_target: &mut Option<std::path::PathBuf>,
) {
    let kind = project_folder_kind(path, application_id);
    let name = path
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    let suffix = match &kind {
        ProjectFolderKind::Folder => String::new(),
        ProjectFolderKind::Project => {
            format!(" [{}]", localization::text("project.folder.project"))
        }
        ProjectFolderKind::OtherApplicationProject(application_name) => format!(
            " [{application_name} {}]",
            localization::text("project.folder.project")
        ),
        ProjectFolderKind::UnavailableProject => format!(
            " [{}]",
            localization::text("project.folder.unavailable_project")
        ),
    };
    if kind != ProjectFolderKind::Folder {
        let response = ui.selectable_label(path == selected, format!("{name}{suffix}"));
        project_tree_context_menu(&response, path, false, false, create_parent, delete_target);
        if reveal_selection && path == selected {
            response.scroll_to_me(Some(egui::Align::Center));
        }
        if kind == ProjectFolderKind::Project && response.double_clicked() {
            *open_project = Some(path.to_path_buf());
        } else if response.clicked() {
            *next = Some(path.to_path_buf());
        }
        return;
    }
    let should_open = selected.starts_with(path);
    let heading = egui::RichText::new(format!("{name}{suffix}"));
    let heading = if path == selected {
        heading
            .strong()
            .background_color(ui.visuals().selection.bg_fill)
    } else {
        heading
    };
    let id = ui.make_persistent_id(("project_folder_tree", path));
    let response = egui::CollapsingHeader::new(heading)
        .id_salt(id)
        .default_open(should_open)
        .open(should_open.then_some(true))
        .show(ui, |ui| {
            for child in project_browser_directories(path) {
                show_project_folder_tree(
                    ui,
                    &child,
                    selected,
                    application_id,
                    next,
                    open_project,
                    reveal_selection,
                    create_parent,
                    delete_target,
                );
            }
        });
    let header = response.header_response;
    project_tree_context_menu(&header, path, true, true, create_parent, delete_target);
    if reveal_selection && path == selected {
        header.scroll_to_me(Some(egui::Align::Center));
    }
    if header.clicked() {
        *next = Some(path.to_path_buf());
    }
}

fn project_tree_context_menu(
    response: &egui::Response,
    path: &std::path::Path,
    is_folder: bool,
    allow_create: bool,
    create_parent: &mut Option<std::path::PathBuf>,
    delete_target: &mut Option<std::path::PathBuf>,
) {
    response.context_menu(|ui| {
        let parent = if is_folder {
            path
        } else {
            path.parent().unwrap_or(path)
        };
        if allow_create
            && ui
                .button(localization::text("project.folder.create_folder"))
                .clicked()
        {
            if create_parent.is_none() {
                *create_parent = Some(parent.to_path_buf());
            }
            ui.close();
        }
        if is_folder
            && ui
                .button(localization::text("project.folder.delete_folder"))
                .clicked()
        {
            if delete_target.is_none() {
                *delete_target = Some(path.to_path_buf());
            }
            ui.close();
        }
        if ui
            .button(localization::text("project.folder.show_in_file_manager"))
            .clicked()
        {
            // Project directories are folders too, even when not ordinary folders.
            let open_path = if path.is_dir() {
                path
            } else {
                path.parent().unwrap_or(path)
            };
            #[cfg(windows)]
            let result = {
                let normalized = std::path::PathBuf::from(
                    open_path.to_string_lossy().replacen(":\\\\", ":\\", 1),
                );
                let argument = normalized.as_os_str();
                std::process::Command::new("explorer.exe")
                    .arg(argument)
                    .spawn()
            };
            #[cfg(target_os = "macos")]
            let result = std::process::Command::new("open").arg(open_path).spawn();
            #[cfg(all(not(windows), not(target_os = "macos")))]
            let result = std::process::Command::new("xdg-open")
                .arg(open_path)
                .spawn();
            if let Err(error) = result {
                ui.label(error.to_string());
            }
            ui.close();
        }
    });
}

impl FrameworkHost {
    fn refresh_application_project_panel(&mut self) {
        let Some(project) = self.project_panel_lifecycle.take_refresh_request() else {
            return;
        };
        let (Some(panel_id), Some(provider), Some(registry_source)) = (
            self.project_panel_id.as_ref(),
            self.project_panel_provider.as_ref(),
            self.project_panel_registry.as_ref(),
        ) else {
            return;
        };
        let roots = provider.project_panel_roots(&project);
        let registry = registry_source(&project);
        match project_panel_api::build_application_project_tree(&roots, &registry) {
            Ok(mut model) => {
                if let Some(previous) = self.tree_viewers.get(panel_id) {
                    model.selected_id = previous.selected_id.clone();
                }
                self.tree_viewers.insert(panel_id.clone(), model);
            }
            Err(error) => {
                tracing::warn!(target: "wfide::project_panel", ?error, "project panel refresh rejected");
            }
        }
    }

    fn modal_dialog_open(&self) -> bool {
        self.project_ui.new_project.is_some()
            || self.project_ui.folder_dialog.is_some()
            || self.project_ui.pending_open.is_some()
            || self.project_ui.confirm_close
            || self.project_ui.show_about
            || self.project_ui.project_properties.is_some()
            || self.project_ui.message.is_some()
    }

    fn handle_project_result(
        &mut self,
        result: project_controller::ProjectCommandResult,
        open_context: Option<project::ProjectContext>,
    ) {
        use project_controller::ProjectCommandResult;
        match result {
            ProjectCommandResult::Completed => self.project_ui.message = None,
            ProjectCommandResult::NeedsSaveLocation => self.project_save_as(),
            ProjectCommandResult::NeedsOpenDecision(result) => {
                if let Some(context) = open_context {
                    self.project_ui.pending_open = Some((context, result));
                }
            }
            ProjectCommandResult::NeedsDirtyConfirmation => self.project_ui.confirm_close = true,
            ProjectCommandResult::Failed(error) => self.project_ui.message = Some(error),
        }
    }

    fn project_new(&mut self) {
        self.project_ui.new_project = Some(NewProjectDialogState {
            name: String::new(),
            description: String::new(),
        });
    }

    fn create_new_project(&mut self, name: String, description: String) {
        let language = localization::current_locale();
        let description = (!description.trim().is_empty()).then(|| description.trim().to_owned());
        let (Some(controller), Some(adapter)) = (
            self.project_controller.as_mut(),
            self.project_adapter.as_deref_mut(),
        ) else {
            return;
        };
        let result = controller.new_project(
            name.trim(),
            language,
            description,
            project_resource::NewProjectStoragePolicy::Deferred,
            None,
            adapter,
        );
        let completed = matches!(result, project_controller::ProjectCommandResult::Completed);
        self.handle_project_result(result, None);
        if completed {
            self.emit_project_event(project_event::ProjectEventKind::Created);
        }
    }

    fn initial_project_browser_directory(&self) -> std::path::PathBuf {
        self.project_controller
            .as_ref()
            .and_then(|controller| controller.session.as_ref())
            .and_then(|session| session.context())
            .and_then(|context| context.root().parent())
            .map(std::path::Path::to_path_buf)
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_default()
    }

    fn project_open(&mut self) {
        self.project_ui.folder_dialog = Some(ProjectFolderDialogState {
            mode: ProjectFolderDialogMode::Open,
            directory: self
                .project_controller
                .as_ref()
                .and_then(|controller| controller.session.as_ref())
                .and_then(|session| session.context())
                .map(|context| context.root().to_path_buf())
                .or_else(|| {
                    self.recent_projects
                        .paths
                        .first()
                        .filter(|path| path.is_dir())
                        .cloned()
                })
                .unwrap_or_else(|| self.initial_project_browser_directory()),
            project_name: String::new(),
            reveal_tree_selection: true,
            selected_entry: None,
            new_folder_name: String::new(),
            create_folder_parent: None,
            delete_folder_pending: None,
            delete_folder_error: None,
        });
    }

    fn save_recent_projects(&self) {
        if let Some(path) = recent_projects::settings_path(
            &self.config.id,
            self.config.user_config_directory_name.as_deref(),
        ) && let Err(error) = self.recent_projects.save(&path)
        {
            tracing::warn!(%error, "Could not save recent projects");
        }
    }

    fn record_current_project_in_history(&mut self) {
        let root = self
            .project_controller
            .as_ref()
            .and_then(|controller| controller.session.as_ref())
            .and_then(|session| session.context())
            .map(|context| context.root().to_path_buf());
        if let Some(root) = root {
            self.recent_projects.record(root);
            self.save_recent_projects();
        }
    }

    fn open_project_at(&mut self, root: std::path::PathBuf) {
        let context = project::ProjectContext::new(root);
        let (Some(controller), Some(adapter)) = (
            self.project_controller.as_mut(),
            self.project_adapter.as_deref_mut(),
        ) else {
            return;
        };
        let result = controller.open(context.clone(), adapter);
        let completed = matches!(result, project_controller::ProjectCommandResult::Completed);
        self.handle_project_result(result, Some(context.clone()));
        if completed {
            self.recent_projects.record(context.root().to_path_buf());
            self.save_recent_projects();
            self.restore_project_workspaces();
            self.emit_project_event(project_event::ProjectEventKind::Opened);
        }
    }

    /// Keep independent live dock trees for each workspace across selection changes.
    fn sync_active_workspace_dock(&mut self) -> Result<(), workspace::WorkspaceError> {
        workspace_dock::switch_workspace_projection(
            &self.workspace_registry,
            &mut self.active_workspace_dock_id,
            &mut self.workspace_docks,
            &mut self.workspace_docks_by_id,
        )
    }

    fn capture_project_workspaces(&mut self) -> Result<(), workspace::WorkspaceError> {
        self.sync_active_workspace_dock()?;
        let mut projections = std::collections::BTreeMap::new();
        for workspace in self.workspace_registry.iter() {
            let projection = if workspace.id == self.active_workspace_dock_id {
                self.workspace_docks.clone()
            } else if let Some(cached) = self.workspace_docks_by_id.get(&workspace.id) {
                cached.clone()
            } else {
                workspace_dock::project_workspace(workspace)?
            };
            projections.insert(workspace.id.clone(), projection);
        }
        if let Some(controller) = self.project_controller.as_mut() {
            controller.capture_workspace_layouts(&self.workspace_registry, &projections)?;
        }
        Ok(())
    }

    fn restore_project_workspaces(&mut self) {
        let Some(controller) = self.project_controller.as_ref() else {
            return;
        };
        match controller.restore_project_workspace_layouts(&mut self.workspace_registry) {
            Ok(mut restored) => {
                let selected = self.workspace_registry.selected_id().to_owned();
                if let Some(projection) = restored.remove(&selected) {
                    self.workspace_docks = projection;
                } else if let Some(workspace) = self.workspace_registry.get(&selected)
                    && let Ok(projection) = workspace_dock::project_workspace(workspace)
                {
                    self.workspace_docks = projection;
                }
                self.workspace_docks_by_id = restored;
                self.active_workspace_dock_id = selected;
            }
            Err(error) => {
                self.project_ui.message =
                    Some(format!("Workspace layout restore failed: {error:?}"));
            }
        }
    }

    fn sync_unsaved_notifications(&mut self) {
        let revision = self.unsaved_notification.revision();
        if revision != self.unsaved_revision_seen {
            self.unsaved_revision_seen = revision;
            if let Some(controller) = self.project_controller.as_mut() {
                controller.mark_unsaved();
            }
        }
    }

    fn project_save(&mut self) {
        self.sync_unsaved_notifications();
        self.refresh_application_project_panel();
        if let Err(error) = self.capture_project_workspaces() {
            self.project_ui.message = Some(format!("Workspace layout capture failed: {error:?}"));
            return;
        }
        let (save_id, saved_at) = project_save_stamp();
        let (Some(controller), Some(adapter)) = (
            self.project_controller.as_mut(),
            self.project_adapter.as_deref_mut(),
        ) else {
            return;
        };
        let result = controller.save(adapter, &save_id, &saved_at);
        let completed = matches!(result, project_controller::ProjectCommandResult::Completed);
        self.handle_project_result(result, None);
        if completed {
            self.record_current_project_in_history();
            self.emit_project_event(project_event::ProjectEventKind::Saved);
        }
    }

    fn project_save_as(&mut self) {
        let project_name = self
            .project_controller
            .as_ref()
            .and_then(|controller| controller.project_name())
            .unwrap_or_default()
            .to_owned();
        self.project_ui.folder_dialog = Some(ProjectFolderDialogState {
            mode: ProjectFolderDialogMode::SaveAs,
            directory: self.initial_project_browser_directory(),
            project_name,
            reveal_tree_selection: true,
            selected_entry: None,
            new_folder_name: String::new(),
            create_folder_parent: None,
            delete_folder_pending: None,
            delete_folder_error: None,
        });
    }

    fn save_project_as_at(&mut self, parent: std::path::PathBuf, project_name: String) {
        let project_name = project_name.trim().to_owned();
        if project_name.is_empty() {
            return;
        }
        let root = parent.join(&project_name);
        if root.exists() {
            self.project_ui.message = Some(localization::text("project.folder.project_exists"));
            return;
        }
        if let Err(error) = self.capture_project_workspaces() {
            self.project_ui.message = Some(format!("Workspace layout capture failed: {error:?}"));
            return;
        }
        let (save_id, saved_at) = project_save_stamp();
        let (Some(controller), Some(adapter)) = (
            self.project_controller.as_mut(),
            self.project_adapter.as_deref_mut(),
        ) else {
            return;
        };
        let result = controller.save_as(
            project::ProjectContext::new(root),
            project_name,
            adapter,
            &save_id,
            &saved_at,
        );
        let completed = matches!(result, project_controller::ProjectCommandResult::Completed);
        self.handle_project_result(result, None);
        if completed {
            self.record_current_project_in_history();
            self.emit_project_event(project_event::ProjectEventKind::SavedAs);
        }
    }

    fn project_properties_open(&mut self) {
        let Some(controller) = self.project_controller.as_ref() else {
            return;
        };
        let Some(name) = controller.project_name() else {
            return;
        };
        self.project_ui.project_properties = Some(ProjectPropertiesDialogState {
            mode: ProjectPropertiesMode::Edit,
            data: ProjectPropertiesData {
                name: name.to_owned(),
                description: controller
                    .project_description()
                    .unwrap_or_default()
                    .to_owned(),
            },
        });
    }

    fn project_properties_create_details(&mut self) {
        let Some(new_project) = self.project_ui.new_project.as_ref() else {
            return;
        };
        self.project_ui.project_properties = Some(ProjectPropertiesDialogState {
            mode: ProjectPropertiesMode::Create,
            data: ProjectPropertiesData {
                name: new_project.name.clone(),
                description: new_project.description.clone(),
            },
        });
    }

    fn apply_project_properties(
        &mut self,
        mode: ProjectPropertiesMode,
        data: ProjectPropertiesData,
    ) {
        match mode {
            ProjectPropertiesMode::Create => {
                self.project_ui.new_project = None;
                self.create_new_project(data.name, data.description);
            }
            ProjectPropertiesMode::Edit => {
                if let Some(controller) = self.project_controller.as_mut() {
                    let description =
                        (!data.description.trim().is_empty()).then_some(data.description);
                    let result = controller.update_project_metadata(data.name, description);
                    self.handle_project_result(result, None);
                }
            }
        }
    }

    fn project_close(&mut self, confirmed: bool) {
        self.emit_project_event(project_event::ProjectEventKind::CloseRequested);
        let Some(controller) = self.project_controller.as_mut() else {
            return;
        };
        let result = controller.close(confirmed);
        let completed = matches!(result, project_controller::ProjectCommandResult::Completed);
        self.handle_project_result(result, None);
        if completed {
            self.emit_project_event(project_event::ProjectEventKind::Closed);
        }
    }

    fn project_menu_item(ui: &mut egui::Ui, label: String, shortcut: &str, enabled: bool) -> bool {
        // Reserve separate space for translated labels and shortcut hints.
        // The old 230-point row allowed long Japanese labels to overlap Ctrl+Shift+S.
        let label_width = 230.0;
        let shortcut_width = 115.0;
        let width = label_width + shortcut_width;
        ui.add_enabled_ui(enabled, |ui| {
            ui.set_min_width(width);
            ui.allocate_ui_with_layout(
                egui::vec2(width, 22.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    let label_response = ui.add_sized(
                        egui::vec2(label_width, 22.0),
                        egui::Button::selectable(false, label),
                    );
                    ui.allocate_ui_with_layout(
                        egui::vec2(shortcut_width, 22.0),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.weak(shortcut);
                        },
                    );
                    label_response.clicked()
                },
            )
            .inner
        })
        .inner
    }

    /// Avoid triggering application commands while an editor or IME owns keyboard focus.
    fn shortcuts_blocked(ctx: &egui::Context) -> bool {
        ctx.egui_wants_keyboard_input()
    }

    /// Framework project shortcuts cannot be overridden by consumer commands.
    fn is_reserved_project_shortcut(shortcut: &egui::KeyboardShortcut) -> bool {
        let modifiers = shortcut.modifiers;
        (modifiers.ctrl || modifiers.command)
            && !modifiers.alt
            && matches!(
                shortcut.logical_key,
                egui::Key::N | egui::Key::O | egui::Key::S
            )
    }

    fn project_shortcuts(&mut self, ui: &egui::Ui) {
        if Self::shortcuts_blocked(ui.ctx()) {
            return;
        }
        let modifiers = ui.input(|input| input.modifiers);
        if modifiers.alt && ui.input(|input| input.key_pressed(egui::Key::F4)) {
            self.request_exit(ui.ctx());
            return;
        }
        let ctrl = modifiers.command;
        if !ctrl {
            return;
        }
        let is_open = self
            .project_controller
            .as_ref()
            .is_some_and(|c| c.is_open());
        if ui.input(|input| input.key_pressed(egui::Key::N)) {
            self.project_new();
        } else if ui.input(|input| input.key_pressed(egui::Key::O)) {
            self.project_open();
        } else if is_open && modifiers.shift && ui.input(|input| input.key_pressed(egui::Key::S)) {
            self.project_save_as();
        } else if is_open && ui.input(|input| input.key_pressed(egui::Key::S)) {
            self.project_save();
        }
    }

    fn request_exit(&mut self, ctx: &egui::Context) {
        self.application_events
            .emit(application_event::ApplicationEvent::CloseRequested);
        let is_open = self
            .project_controller
            .as_ref()
            .is_some_and(|controller| controller.is_open());
        if !is_open {
            self.application_events
                .emit(application_event::ApplicationEvent::Closing);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.project_ui.exit_after_close = true;
        self.project_close(false);
        if !self.project_ui.confirm_close
            && self
                .project_controller
                .as_ref()
                .is_none_or(|controller| !controller.is_open())
        {
            self.application_events
                .emit(application_event::ApplicationEvent::Closing);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn consumer_menu_items_between(
        &mut self,
        ui: &mut egui::Ui,
        location: command::MenuLocation,
        menu_id: Option<&str>,
        previous: Option<&str>,
        next: Option<&str>,
    ) {
        let framework: &[&str] = match location {
            command::MenuLocation::File => &[
                "file.new_project",
                "file.open_project",
                "file.recent_projects",
                "file.save",
                "file.save_as",
                "file.project_properties",
                "file.close_project",
                "file.exit",
            ],
            command::MenuLocation::Help => &["help.about"],
            command::MenuLocation::Custom => &[],
        };
        let items = self
            .menu_items
            .iter()
            .filter(|item| item.location == location && menu_id.is_none_or(|id| item.menu_id == id))
            .cloned()
            .collect::<Vec<_>>();
        let order = command::resolve_menu_order(framework, &items);
        let begin = previous
            .and_then(|id| order.iter().position(|x| x == id))
            .map_or(0, |i| i + 1);
        let end = next
            .and_then(|id| order.iter().position(|x| x == id))
            .unwrap_or(order.len());
        if begin > end {
            return;
        }
        for id in &order[begin..end] {
            let Some(item) = items.iter().find(|item| &item.command_id == id) else {
                continue;
            };
            let Some(command) = self.commands.get(&item.command_id) else {
                continue;
            };
            let label = localization::text(&command.label);
            let shortcut = command.shortcut.clone().unwrap_or_default();
            let enabled = command.enabled;
            if Self::project_menu_item(ui, label, &shortcut, enabled) {
                ui.close();
                let _ = self.commands.dispatch(&item.command_id);
            }
        }
    }

    fn application_status(&self, ui: &mut egui::Ui) {
        let Ok(items) = self.status.snapshot() else {
            return;
        };
        let visible = items
            .into_iter()
            .filter(|item| item.visible)
            .collect::<Vec<_>>();
        if visible.is_empty() {
            return;
        }
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            for (index, item) in visible.iter().enumerate() {
                if index > 0 {
                    ui.separator();
                }
                ui.label(format!("{}: {}", item.label, item.value));
            }
        });
    }

    fn emit_project_event(&mut self, kind: project_event::ProjectEventKind) {
        let root = self
            .project_controller
            .as_ref()
            .and_then(|controller| controller.session.as_ref())
            .and_then(|session| session.context())
            .map(|context| context.root().to_path_buf());
        let name = self
            .project_controller
            .as_ref()
            .and_then(|controller| controller.project_name())
            .map(str::to_owned);
        match kind {
            project_event::ProjectEventKind::Closed => {
                self.project_panel_lifecycle.close_project();
            }
            project_event::ProjectEventKind::Opened
            | project_event::ProjectEventKind::Created
            | project_event::ProjectEventKind::SavedAs => {
                if let Some(ref root) = root {
                    self.project_panel_lifecycle
                        .activate_project(&project::ProjectContext::new(root.clone()));
                } else {
                    self.project_panel_lifecycle.close_project();
                }
            }
            project_event::ProjectEventKind::CloseRequested => {}
            project_event::ProjectEventKind::Saved => {
                self.project_panel_lifecycle
                    .invalidate_application_project_panel();
            }
        }
        self.project_events
            .emit(project_event::ProjectEvent::new(kind, root, name));
    }

    fn application_menu(&mut self, ui: &mut egui::Ui) {
        if self.modal_dialog_open() {
            let frame = egui::Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .inner_margin(egui::Margin::symmetric(6, 3));
            frame.show(ui, |ui| {
                ui.add_enabled_ui(false, |ui| {
                    egui::MenuBar::new().ui(ui, |ui| {
                        let _ = ui.button(localization::text("file.menu"));
                        let _ = ui.button(localization::text("workspace.menu"));
                        let _ = ui.button(localization::text("help.menu"));
                    });
                });
            });
            return;
        }
        self.project_shortcuts(ui);
        // Consumer shortcuts are handled after the framework's reserved shortcuts.
        // Do not steal keys from active text widgets or IME composition.
        if !Self::shortcuts_blocked(ui.ctx()) {
            let shortcuts: Vec<_> = self
                .commands
                .iter()
                .filter(|command| command.enabled)
                .filter_map(|command| {
                    command
                        .shortcut
                        .as_deref()
                        .and_then(command::parse_shortcut)
                })
                .filter(|shortcut| !Self::is_reserved_project_shortcut(shortcut))
                .collect();
            for shortcut in shortcuts {
                if ui.input_mut(|input| input.consume_shortcut(&shortcut)) {
                    let _ = self.commands.dispatch_shortcut(&shortcut);
                    break;
                }
            }
        }
        let frame = egui::Frame::new()
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(egui::Margin::symmetric(6, 3))
            .stroke(egui::Stroke::new(
                1.0_f32,
                ui.visuals().widgets.noninteractive.bg_stroke.color,
            ));
        frame.show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let custom_menu_items = self.menu_items.iter()
                    .filter(|item| item.location == command::MenuLocation::Custom)
                    .fold(Vec::<command::MenuItemDefinition>::new(), |mut acc, item| {
                        if !acc.iter().any(|x| x.menu_id == item.menu_id) { acc.push(item.clone()); }
                        acc
                    });
                let top_level_order = command::resolve_menu_order(
                    &["menu.file", "menu.workspace", "menu.help"],
                    &custom_menu_items.iter().map(|item| {
                        let mut item = item.clone();
                        item.command_id = item.menu_id.clone();
                        item.placement = item.top_level_placement.clone();
                        item
                    }).collect::<Vec<_>>(),
                );
                self.render_custom_menus_at(ui, &top_level_order, None, Some("menu.file"));
                let file_access_key =
                    ui.input(|input| input.modifiers.alt && input.key_pressed(egui::Key::F));
                let file_response = ui.add(egui::Button::new(localization::text("file.menu")));
                if file_access_key {
                    egui::Popup::open_id(ui.ctx(), file_response.id.with("popup"));
                }
                egui::Popup::menu(&file_response).show(|ui| {
                    let has_project_support = self.project_controller.is_some();
                    let is_open = self
                        .project_controller
                        .as_ref()
                        .is_some_and(|c| c.is_open());
                    self.consumer_menu_items_between(ui, command::MenuLocation::File, None, None, Some("file.new_project"));
                    if Self::project_menu_item(
                        ui,
                        localization::text("file.new_project"),
                        "Ctrl+N",
                        has_project_support,
                    ) {
                        ui.close();
                        self.project_new();
                    }
                    self.consumer_menu_items_between(ui, command::MenuLocation::File, None, Some("file.new_project"), Some("file.open_project"));
                    if Self::project_menu_item(
                        ui,
                        localization::text("file.open_project"),
                        "Ctrl+O",
                        has_project_support,
                    ) {
                        ui.close();
                        self.project_open();
                    }
                    self.consumer_menu_items_between(ui, command::MenuLocation::File, None, Some("file.open_project"), Some("file.recent_projects"));
                    ui.menu_button(localization::text("file.recent_projects"), |ui| {
                        ui.set_min_width(420.0);
                        if self.recent_projects.paths.is_empty() {
                            ui.add_enabled(false, egui::Label::new(localization::text("file.recent_empty")));
                        }
                        for path in self.recent_projects.paths.clone() {
                            let valid = project_folder_kind(&path, &self.config.id) == ProjectFolderKind::Project;
                            let name = path.file_name().unwrap_or(path.as_os_str()).to_string_lossy();
                            let name: String = if name.chars().count() > 30 {
                                format!("{}…", name.chars().take(29).collect::<String>())
                            } else {
                                name.into_owned()
                            };
                            let saved_at = recent_projects::saved_at_label(&path, &self.config.id);
                            ui.horizontal(|ui| {
                                let response = ui.add_enabled(
                                    valid,
                                    egui::Button::new(name).min_size(egui::vec2(248.0, 22.0)),
                                ).on_hover_text(path.display().to_string());
                                ui.label(saved_at);
                                if response.clicked() {
                                    ui.close();
                                    self.open_project_at(path);
                                }
                            });
                        }
                        if !self.recent_projects.paths.is_empty() {
                            ui.separator();
                            if ui.button(localization::text("file.recent_clear")).clicked() {
                                self.recent_projects.clear();
                                self.save_recent_projects();
                                ui.close();
                            }
                        }
                    });
                    ui.separator();

                    self.consumer_menu_items_between(ui, command::MenuLocation::File, None, Some("file.recent_projects"), Some("file.save"));
                    if Self::project_menu_item(
                        ui,
                        localization::text("file.save"),
                        "Ctrl+S",
                        is_open,
                    ) {
                        ui.close();
                        self.project_save();
                    }

                    self.consumer_menu_items_between(ui, command::MenuLocation::File, None, Some("file.save"), Some("file.save_as"));
                    if Self::project_menu_item(
                        ui,
                        localization::text("file.save_as"),
                        "Ctrl+Shift+S",
                        is_open,
                    ) {
                        ui.close();
                        self.project_save_as();
                    }
                    ui.separator();
                    self.consumer_menu_items_between(ui, command::MenuLocation::File, None, Some("file.save_as"), Some("file.project_properties"));
                    if Self::project_menu_item(
                        ui,
                        localization::text("file.project_properties"),
                        "",
                        is_open,
                    ) {
                        ui.close();
                        self.project_properties_open();
                    }
                    ui.separator();
                    self.consumer_menu_items_between(ui, command::MenuLocation::File, None, Some("file.project_properties"), Some("file.close_project"));
                    if Self::project_menu_item(
                        ui,
                        localization::text("file.close_project"),
                        "",
                        is_open,
                    ) {
                        ui.close();
                        self.project_close(false);
                    }
                    ui.separator();
                    self.consumer_menu_items_between(ui, command::MenuLocation::File, None, Some("file.close_project"), Some("file.exit"));
                    if Self::project_menu_item(ui, localization::text("file.exit"), "Alt+F4", true)
                    {
                        ui.close();
                        self.request_exit(ui.ctx());
                    }
                    self.consumer_menu_items_between(ui, command::MenuLocation::File, None, Some("file.exit"), None);

                });

                self.render_custom_menus_at(ui, &top_level_order, Some("menu.file"), Some("menu.workspace"));
                let workspace_access_key =
                    ui.input(|input| input.modifiers.alt && input.key_pressed(egui::Key::W));
                let workspace_response = ui.add(egui::Button::new(localization::text("workspace.menu")));
                if workspace_access_key {
                    egui::Popup::open_id(ui.ctx(), workspace_response.id.with("popup"));
                }
                egui::Popup::menu(&workspace_response).show(|ui| {
                    let entries: Vec<(String, String)> = self.workspace_registry
                        .iter()
                        .map(|workspace| (workspace.id.clone(), workspace.name.clone()))
                        .collect();
                    for (id, name) in entries {
                        let selected = self.workspace_registry.selected_id() == id;
                        if ui.selectable_label(selected, name).clicked() {
                            if let Err(error) = self.workspace_registry.select(&id)
                                .and_then(|_| self.sync_active_workspace_dock())
                            {
                                tracing::warn!(target: "wfide::workspace", ?error, "workspace selection failed");
                            }
                            ui.close();
                        }
                    }
                    ui.separator();
                    if ui.button(localization::text("workspace.add")).clicked() {
                        let id = self.workspace_registry.next_user_workspace_id();
                        let ordinal = id.rsplit('.').next().unwrap_or("1");
                        let name = format!("Workspace {ordinal}");
                        if let Err(error) = self.workspace_registry.register(&id, name)
                            .and_then(|_| self.workspace_registry.select(&id))
                            .and_then(|_| self.sync_active_workspace_dock())
                        {
                            tracing::warn!(target: "wfide::workspace", ?error, "workspace creation failed");
                        } else if let Some(session) = self.project_controller
                            .as_mut().and_then(|controller| controller.session.as_mut())
                        {
                            session.mark_framework_dirty();
                        }
                        ui.close();
                    }
                });

                self.render_custom_menus_at(ui, &top_level_order, Some("menu.workspace"), Some("menu.help"));
                let help_access_key =
                    ui.input(|input| input.modifiers.alt && input.key_pressed(egui::Key::H));
                let help_response = ui.add(egui::Button::new(localization::text("help.menu")));
                if help_access_key {
                    egui::Popup::open_id(ui.ctx(), help_response.id.with("popup"));
                }
                egui::Popup::menu(&help_response).show(|ui| {

                    self.consumer_menu_items_between(ui, command::MenuLocation::Help, None, None, Some("help.about"));
                    if Self::project_menu_item(ui, localization::text("help.about"), "", true) {
                        ui.close();
                        self.project_ui.show_about = true;
                    }
                    self.consumer_menu_items_between(ui, command::MenuLocation::Help, None, Some("help.about"), None);


                });

                self.render_custom_menus_at(ui, &top_level_order, Some("menu.help"), None);
            });
        });
    }

    fn render_custom_menus_at(
        &mut self,
        ui: &mut egui::Ui,
        order: &[String],
        previous: Option<&str>,
        next: Option<&str>,
    ) {
        let begin = previous
            .and_then(|id| order.iter().position(|x| x == id))
            .map_or(0, |i| i + 1);
        let end = next
            .and_then(|id| order.iter().position(|x| x == id))
            .unwrap_or(order.len());
        if begin > end {
            return;
        }
        for id in &order[begin..end] {
            if matches!(id.as_str(), "menu.file" | "menu.workspace" | "menu.help") {
                continue;
            }
            let Some(item) = self
                .menu_items
                .iter()
                .find(|item| item.location == command::MenuLocation::Custom && item.menu_id == *id)
            else {
                continue;
            };
            let menu_id = item.menu_id.clone();
            let menu_label = item.menu_label.clone();
            let access_key_pressed = item
                .access_key
                .and_then(|letter| egui::Key::from_name(&letter.to_string()))
                .is_some_and(|key| {
                    ui.input(|input| {
                        input.modifiers.alt
                            && !input.modifiers.ctrl
                            && !input.modifiers.shift
                            && input.key_pressed(key)
                    })
                });
            let mut display_label = localization::text(&menu_label).to_string();
            if let Some(key) = item.access_key {
                display_label.push_str(&format!(" ({})", key.to_ascii_uppercase()));
            }
            let response = ui.add(egui::Button::new(display_label));
            if access_key_pressed {
                egui::Popup::open_id(ui.ctx(), response.id.with("popup"));
            }
            egui::Popup::menu(&response).show(|ui| {
                self.consumer_menu_items_between(
                    ui,
                    command::MenuLocation::Custom,
                    Some(&menu_id),
                    None,
                    None,
                );
            });
        }
    }

    fn project_dialogs(&mut self, ctx: &egui::Context) {
        if self.project_ui.folder_dialog.is_some() {
            let mut navigate_to = None;
            let mut open_project = None;
            let mut save_parent = None;
            let mut save_name = None;
            let mut cancel = false;
            let mut create_folder = None;
            let mut context_create_parent = None;
            let mut context_delete_target = None;
            let state = self
                .project_ui
                .folder_dialog
                .as_ref()
                .expect("project folder dialog");
            let is_save = matches!(state.mode, ProjectFolderDialogMode::SaveAs);
            let title = localization::text(if is_save {
                "project.folder.save_title"
            } else {
                "project.folder.open_title"
            });
            let directory = state.directory.clone();
            let reveal_tree_selection = state.reveal_tree_selection;
            egui::Window::new(title)
                .collapsible(false)
                .resizable(true)
                .default_size([620.0, 420.0])
                .show(ctx, |ui| {
                    ui.label(localization::text("project.folder.location"));
                    ui.horizontal(|ui| {
                        if ui.button(localization::text("project.folder.up")).clicked()
                            && let Some(parent) = directory.parent()
                        {
                            navigate_to = Some(parent.to_path_buf());
                        }
                        ui.monospace(directory.display().to_string());
                    });
                    ui.label(localization::text(if is_save {
                        "project.folder.save_hint"
                    } else {
                        "project.folder.open_hint"
                    }));
                    ui.separator();
                    let application_id = self.config.id.clone();
                    ui.columns(2, |columns| {
                        egui::ScrollArea::vertical()
                            .id_salt("project_browser_tree")
                            .max_height(280.0)
                            .show(&mut columns[0], |ui| {
                                ui.strong(localization::text("project.folder.computer"));
                                for root in project_browser_roots() {
                                    show_project_folder_tree(
                                        ui,
                                        &root,
                                        &directory,
                                        &application_id,
                                        &mut navigate_to,
                                        &mut open_project,
                                        reveal_tree_selection,
                                        &mut context_create_parent,
                                        &mut context_delete_target,
                                    );
                                }
                            });
                        let project_details = if !is_save
                            && matches!(
                                project_folder_kind(&directory, &application_id),
                                ProjectFolderKind::Project
                                    | ProjectFolderKind::OtherApplicationProject(_)
                            ) {
                            std::fs::read_to_string(directory.join("project.toml"))
                                .ok()
                                .and_then(|contents| {
                                    toml::from_str::<project::ProjectFile>(&contents).ok()
                                })
                        } else {
                            None
                        };
                        let entries = if project_details.is_some() {
                            Vec::new()
                        } else {
                            project_browser_directories(&directory)
                        };
                        egui::ScrollArea::vertical()
                            .id_salt(("project_browser_contents", &directory))
                            .max_height(280.0)
                            .show(&mut columns[1], |ui| {
                                if let Some(details) = &project_details {
                                    ui.strong(&details.project.name);
                                    ui.separator();
                                    if let Some(description) = &details.project.description {
                                        ui.label(description);
                                    }
                                    ui.label(format!(
                                        "{}: {}",
                                        localization::text("project.new_dialog.language"),
                                        details.project.language
                                    ));
                                    ui.label(format!(
                                        "{}: {}",
                                        localization::text("project.folder.saved_at"),
                                        details.project.saved_at
                                    ));
                                    ui.separator();
                                    ui.label(format!(
                                        "{}: {}",
                                        details.application.name, details.application.id
                                    ));
                                    if let Some(description) = &details.application.description {
                                        ui.label(description);
                                    }
                                }
                                for path in entries {
                                    let kind = project_folder_kind(&path, &application_id);
                                    let name = path
                                        .file_name()
                                        .map(|name| name.to_string_lossy())
                                        .unwrap_or_default();
                                    let label = match &kind {
                                        ProjectFolderKind::Folder => name.into_owned(),
                                        ProjectFolderKind::Project => format!(
                                            "{name}    [{}]",
                                            localization::text("project.folder.project")
                                        ),
                                        ProjectFolderKind::OtherApplicationProject(
                                            application_name,
                                        ) => format!(
                                            "{name}    [{application_name} {}]",
                                            localization::text("project.folder.project")
                                        ),
                                        ProjectFolderKind::UnavailableProject => format!(
                                            "{name}    [{}]",
                                            localization::text(
                                                "project.folder.unavailable_project"
                                            )
                                        ),
                                    };
                                    let enabled = !is_save || kind == ProjectFolderKind::Folder;
                                    let selected = self
                                        .project_ui
                                        .folder_dialog
                                        .as_ref()
                                        .and_then(|state| state.selected_entry.as_ref())
                                        .is_some_and(|selected| selected == &path);
                                    let response = ui.add_enabled(
                                        enabled,
                                        egui::Button::selectable(selected, label),
                                    );
                                    project_tree_context_menu(
                                        &response,
                                        &path,
                                        kind == ProjectFolderKind::Folder,
                                        kind == ProjectFolderKind::Folder,
                                        &mut context_create_parent,
                                        &mut context_delete_target,
                                    );
                                    if response.clicked()
                                        && let Some(state) = self.project_ui.folder_dialog.as_mut()
                                    {
                                        state.selected_entry = Some(path.clone());
                                    }
                                    if response.double_clicked() {
                                        match kind {
                                            ProjectFolderKind::Folder => {
                                                navigate_to = Some(path.clone())
                                            }
                                            ProjectFolderKind::Project if !is_save => {
                                                open_project = Some(path.clone())
                                            }
                                            ProjectFolderKind::OtherApplicationProject(_)
                                                if !is_save =>
                                            {
                                                navigate_to = Some(path.clone())
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                // The unused area belongs to the current directory, not to
                                // the last listed folder. Give it its own context menu.
                                let background = ui.allocate_response(
                                    egui::vec2(
                                        ui.available_width().max(1.0),
                                        ui.available_height().max(1.0),
                                    ),
                                    egui::Sense::click(),
                                );
                                background.context_menu(|ui| {
                                    if project_folder_kind(&directory, &application_id)
                                        == ProjectFolderKind::Folder
                                        && self
                                            .project_ui
                                            .folder_dialog
                                            .as_ref()
                                            .and_then(|state| state.selected_entry.as_ref())
                                            .is_none_or(|selected| {
                                                project_folder_kind(selected, &application_id)
                                                    == ProjectFolderKind::Folder
                                            })
                                        && ui
                                            .button(localization::text(
                                                "project.folder.create_folder",
                                            ))
                                            .clicked()
                                    {
                                        context_create_parent = Some(directory.clone());
                                        ui.close();
                                    }
                                });
                            });
                    });
                    ui.separator();
                    if is_save {
                        ui.horizontal(|ui| {
                            ui.label(localization::text("project.new_dialog.name"));
                            if let Some(state) = self.project_ui.folder_dialog.as_mut() {
                                ui.text_edit_singleline(&mut state.project_name);
                            }
                        });
                    }
                    ui.horizontal(|ui| {
                        if !is_save
                            && ui
                                .add_enabled(
                                    project_folder_kind(&directory, &application_id)
                                        == ProjectFolderKind::Project,
                                    egui::Button::new(localization::text(
                                        "project.folder.open_selected",
                                    )),
                                )
                                .clicked()
                        {
                            open_project = Some(directory.clone());
                        }
                        if is_save {
                            let project_name = self
                                .project_ui
                                .folder_dialog
                                .as_ref()
                                .map(|state| state.project_name.trim())
                                .unwrap_or_default();
                            let inside_project = directory
                                .ancestors()
                                .any(|ancestor| ancestor.join("project.toml").is_file());
                            let target_exists =
                                !project_name.is_empty() && directory.join(project_name).exists();
                            if ui
                                .add_enabled(
                                    !inside_project && !target_exists && !project_name.is_empty(),
                                    egui::Button::new(localization::text(
                                        "project.folder.save_here",
                                    )),
                                )
                                .clicked()
                            {
                                save_parent = Some(directory.clone());
                                save_name = Some(project_name.to_owned());
                            }
                            if target_exists {
                                ui.weak(localization::text("project.folder.project_exists"));
                            }
                        }
                        if ui.button(localization::text("common.cancel")).clicked() {
                            cancel = true;
                        }
                    });
                });
            if let Some(parent) = self
                .project_ui
                .folder_dialog
                .as_ref()
                .and_then(|state| state.create_folder_parent.clone())
            {
                let mut dismiss_create = false;
                egui::Window::new(localization::text("project.folder.create_folder"))
                    .id(egui::Id::new("project_folder_create"))
                    .collapsible(false)
                    .resizable(false)
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        ui.label(parent.display().to_string());
                        if let Some(state) = self.project_ui.folder_dialog.as_mut() {
                            ui.text_edit_singleline(&mut state.new_folder_name);
                            let name = state.new_folder_name.as_str();
                            let invalid_char = name == "."
                                || name == ".."
                                || name.starts_with(' ')
                                || name.ends_with('.')
                                || name.ends_with(' ')
                                || name.chars().any(|c| {
                                    c == char::from(92)
                                        || matches!(
                                            c,
                                            '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
                                        )
                                });
                            let exists = parent.join(name).exists();
                            if !name.is_empty() && (invalid_char || exists) {
                                ui.label(localization::text(if invalid_char {
                                    "project.folder.invalid_folder_name"
                                } else {
                                    "project.folder.folder_exists"
                                }));
                            }
                            ui.horizontal(|ui| {
                                if ui
                                    .add_enabled(
                                        !name.is_empty() && !invalid_char && !exists,
                                        egui::Button::new(localization::text(
                                            "project.folder.create_folder",
                                        )),
                                    )
                                    .clicked()
                                {
                                    create_folder = Some(parent.join(name));
                                }
                                if ui.button(localization::text("common.cancel")).clicked() {
                                    dismiss_create = true;
                                }
                            });
                        }
                    });
                if dismiss_create && let Some(state) = self.project_ui.folder_dialog.as_mut() {
                    state.create_folder_parent = None;
                    state.new_folder_name.clear();
                }
            }
            if let Some(path) = self
                .project_ui
                .folder_dialog
                .as_ref()
                .and_then(|state| state.delete_folder_pending.clone())
            {
                egui::Window::new(localization::text("project.folder.confirm_delete"))
                    .id(egui::Id::new("project_folder_delete_confirm"))
                    .collapsible(false)
                    .resizable(false)
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        ui.label(path.display().to_string());
                        ui.horizontal(|ui| {
                            if ui
                                .button(localization::text("project.folder.delete_folder"))
                                .clicked()
                            {
                                match std::fs::remove_dir(&path) {
                                    Ok(()) => {
                                        navigate_to =
                                            path.parent().map(std::path::Path::to_path_buf)
                                    }
                                    Err(error) => {
                                        if let Some(state) = self.project_ui.folder_dialog.as_mut()
                                        {
                                            let reason = if error.kind()
                                                == std::io::ErrorKind::DirectoryNotEmpty
                                            {
                                                localization::text(
                                                    "project.folder.delete_not_empty",
                                                )
                                            } else {
                                                localization::text("project.folder.delete_failed")
                                            };
                                            state.delete_folder_error = Some(format!(
                                                "{}\n{}\n{}: {}",
                                                path.display(),
                                                reason,
                                                localization::text("project.folder.os_error"),
                                                error
                                            ));
                                        }
                                    }
                                }
                                if let Some(state) = self.project_ui.folder_dialog.as_mut() {
                                    state.delete_folder_pending = None;
                                }
                            }
                            if ui.button(localization::text("common.cancel")).clicked()
                                && let Some(state) = self.project_ui.folder_dialog.as_mut()
                            {
                                state.delete_folder_pending = None;
                            }
                        });
                    });
            }
            if let Some(error) = self
                .project_ui
                .folder_dialog
                .as_ref()
                .and_then(|state| state.delete_folder_error.clone())
            {
                egui::Window::new(localization::text("project.folder.delete_error"))
                    .id(egui::Id::new("project_folder_delete_error"))
                    .collapsible(false)
                    .resizable(false)
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        ui.label(error);
                        if ui.button(localization::text("common.ok")).clicked()
                            && let Some(state) = self.project_ui.folder_dialog.as_mut()
                        {
                            state.delete_folder_error = None;
                        }
                    });
            }
            if let Some(parent) = context_create_parent
                && let Some(state) = self.project_ui.folder_dialog.as_mut()
            {
                state.create_folder_parent = Some(parent);
                state.new_folder_name.clear();
            }
            if let Some(path) = context_delete_target
                && let Some(state) = self.project_ui.folder_dialog.as_mut()
            {
                state.delete_folder_pending = Some(path);
            }
            if let Some(path) = create_folder {
                match std::fs::create_dir(&path) {
                    Ok(()) => {
                        if let Some(state) = self.project_ui.folder_dialog.as_mut() {
                            state.new_folder_name.clear();
                            state.create_folder_parent = None;
                        }
                        navigate_to = Some(path);
                    }
                    Err(error) => self.project_ui.message = Some(error.to_string()),
                }
            }
            if let Some(path) = navigate_to {
                if let Some(state) = self.project_ui.folder_dialog.as_mut() {
                    state.directory = path;
                    state.selected_entry = None;
                    // User-initiated selection must not auto-scroll the tree.
                    state.reveal_tree_selection = false;
                }
            } else if let Some(state) = self.project_ui.folder_dialog.as_mut() {
                state.reveal_tree_selection = false;
            }
            if let Some(path) = open_project {
                self.project_ui.folder_dialog = None;
                self.open_project_at(path);
            } else if let (Some(parent), Some(project_name)) = (save_parent, save_name) {
                self.project_ui.folder_dialog = None;
                self.save_project_as_at(parent, project_name);
            } else if cancel {
                self.project_ui.folder_dialog = None;
            }
        }
        if self.project_ui.new_project.is_some() && self.project_ui.project_properties.is_none() {
            let mut create = false;
            let mut details = false;
            let mut cancel = false;
            egui::Window::new(localization::text("project.new_dialog.title"))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    let state = self
                        .project_ui
                        .new_project
                        .as_mut()
                        .expect("new project dialog state");
                    ui.label(localization::text("project.new_dialog.name"));
                    ui.text_edit_singleline(&mut state.name);
                    if state.name.trim().is_empty() {
                        ui.weak(localization::text("project.new_dialog.name_required"));
                    }
                    ui.horizontal(|ui| {
                        create = ui
                            .add_enabled(
                                !state.name.trim().is_empty(),
                                egui::Button::new(localization::text("project.new_dialog.create")),
                            )
                            .clicked();
                        details = ui
                            .button(localization::text("project.new_dialog.details"))
                            .clicked();
                        cancel = ui.button(localization::text("common.cancel")).clicked();
                    });
                });
            if create {
                if let Some(state) = self.project_ui.new_project.take() {
                    self.create_new_project(state.name, String::new());
                }
            } else if details {
                self.project_properties_create_details();
            } else if cancel {
                self.project_ui.new_project = None;
            }
        }
        if self.project_ui.confirm_close {
            egui::Window::new(localization::text("project.unsaved.title"))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(localization::text("project.unsaved.message"));
                    ui.horizontal(|ui| {
                        if ui
                            .button(localization::text("project.unsaved.close"))
                            .clicked()
                        {
                            self.project_ui.confirm_close = false;
                            let exit = self.project_ui.exit_after_close;
                            self.project_close(true);
                            if exit {
                                self.application_events
                                    .emit(application_event::ApplicationEvent::Closing);
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                        if ui.button(localization::text("common.cancel")).clicked() {
                            self.project_ui.confirm_close = false;
                            self.project_ui.exit_after_close = false;
                        }
                    });
                });
        }
        if self.project_ui.pending_open.is_some() {
            egui::Window::new(localization::text("project.attention.title"))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(localization::text("project.attention.message"));
                    let can_continue = self
                        .project_ui
                        .pending_open
                        .as_ref()
                        .is_some_and(|(_, result)| project_open::open_can_continue(result));
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                can_continue,
                                egui::Button::new(localization::text(
                                    "project.attention.open_recovery",
                                )),
                            )
                            .clicked()
                            && self.project_controller.is_some()
                            && let Some((context, result)) = self.project_ui.pending_open.take()
                            && let Some(controller) = self.project_controller.as_mut()
                        {
                            let outcome = controller.accept_open(context, result);
                            let completed = matches!(
                                outcome,
                                project_controller::ProjectCommandResult::Completed
                            );
                            self.handle_project_result(outcome, None);
                            if completed {
                                self.restore_project_workspaces();
                                self.emit_project_event(project_event::ProjectEventKind::Opened);
                            }
                        }
                        if ui.button(localization::text("common.cancel")).clicked() {
                            self.project_ui.pending_open = None;
                        }
                    });
                });
        }
        if self.project_ui.show_about {
            if let Some(renderer) = self.about_renderer.as_mut() {
                renderer(
                    ctx,
                    FrameworkInfo::current(),
                    &mut self.project_ui.show_about,
                );
            } else {
                let mut open = self.project_ui.show_about;
                egui::Window::new(localization::text("about.title"))
                    .open(&mut open)
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.heading(&self.config.name);
                        if let Some(version) = &self.config.version {
                            ui.label(format!(
                                "{}: {}",
                                localization::text("about.version"),
                                version
                            ));
                        }
                        ui.separator();
                        let framework = FrameworkInfo::current();
                        ui.label(format!(
                            "{}: {}",
                            localization::text("about.framework"),
                            framework.name
                        ));
                        ui.label(format!(
                            "{}: {}",
                            localization::text("about.version"),
                            framework.version
                        ));
                    });
                self.project_ui.show_about = open;
            }
        }
        if self.project_ui.project_properties.is_some() {
            let mut open = true;
            let mut commit = false;
            {
                let state = self
                    .project_ui
                    .project_properties
                    .as_mut()
                    .expect("project properties state");
                if let Some(renderer) = self.project_properties_renderer.as_mut() {
                    renderer(ctx, state.mode, &mut state.data, &mut open, &mut commit);
                } else {
                    let mut cancel_properties = false;
                    egui::Window::new(localization::text("project.properties.title"))
                        .open(&mut open)
                        .collapsible(false)
                        .resizable(false)
                        .show(ctx, |ui| {
                            ui.label(localization::text("project.new_dialog.name"));
                            ui.text_edit_singleline(&mut state.data.name);
                            ui.label(localization::text("project.new_dialog.description"));
                            ui.text_edit_multiline(&mut state.data.description);
                            if state.data.name.trim().is_empty() {
                                ui.weak(localization::text("project.new_dialog.name_required"));
                            }
                            ui.horizontal(|ui| {
                                let action = if state.mode == ProjectPropertiesMode::Create {
                                    "project.new_dialog.create"
                                } else {
                                    "common.ok"
                                };
                                if ui
                                    .add_enabled(
                                        !state.data.name.trim().is_empty(),
                                        egui::Button::new(localization::text(action)),
                                    )
                                    .clicked()
                                {
                                    commit = true;
                                }
                                if ui.button(localization::text("common.cancel")).clicked() {
                                    cancel_properties = true;
                                }
                            });
                        });
                    if cancel_properties {
                        open = false;
                    }
                }
            }
            if commit {
                if let Some(state) = self.project_ui.project_properties.take() {
                    self.apply_project_properties(state.mode, state.data);
                }
            } else if !open {
                self.project_ui.project_properties = None;
            }
        }
        if let Some(message) = self.project_ui.message.clone() {
            egui::Window::new(localization::text("project.error.title"))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(message);
                    if ui.button(localization::text("common.ok")).clicked() {
                        self.project_ui.message = None;
                    }
                });
        }
    }
}

impl eframe::App for FrameworkHost {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.sync_unsaved_notifications();
        if let Err(error) = self.sync_active_workspace_dock() {
            tracing::warn!(target: "wfide::workspace", ?error, "workspace dock switch rejected");
        }
        if !self.started_emitted {
            self.application_events
                .emit(application_event::ApplicationEvent::Started);
            self.started_emitted = true;
        }
        if !self.fonts_initialized {
            let locale = localization::current_locale();
            match locale_font::install_for_locale(
                ui.ctx(),
                &locale,
                self.config.appearance.font_path.as_deref(),
            ) {
                Ok(paths) if !paths.is_empty() => {
                    tracing::info!(target: "wfide::font", %locale, count = paths.len(), "initial-frame fallback fonts loaded")
                }
                Ok(_) => {
                    tracing::warn!(target: "wfide::font", %locale, "no OS or locale fallback font found on initial frame")
                }
                Err(error) => {
                    tracing::warn!(target: "wfide::font", %locale, %error, "failed to configure fonts on initial frame")
                }
            }
            self.fonts_initialized = true;
            ui.ctx().request_repaint();
        }
        if self.config.appearance.theme == theme::Theme::System {
            let system_dark =
                ui.ctx().system_theme().unwrap_or(egui::Theme::Dark) == egui::Theme::Dark;
            self.config
                .appearance
                .theme
                .apply_with_system_dark(ui.ctx(), system_dark);
        }

        if let Some(editor) = self.theme_editor {
            editor.apply(ui.ctx());
        }

        ui.set_style(ui.ctx().global_style());
        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, ui.visuals().panel_fill);

        self.application_menu(ui);
        self.project_dialogs(ui.ctx());
        self.application_status(ui);

        let application_title = self
            .config
            .window
            .title
            .as_deref()
            .unwrap_or(&self.config.name);
        let window_title = self
            .project_controller
            .as_ref()
            .and_then(|controller| {
                controller.project_name().map(|project_name| {
                    let dirty = controller
                        .session
                        .as_ref()
                        .is_some_and(|session| session.dirty.is_dirty());
                    project_window_title(application_title, project_name, dirty)
                })
            })
            .unwrap_or_else(|| application_title.to_owned());
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::Title(window_title));

        if !self.workspace_docks.normal.is_empty() || !self.workspace_docks.floating.is_empty() {
            ui.separator();
            let mut viewer = FrameworkTabViewer {
                panels: &self.config.panels,
                probe_enabled: self.config.probe_panel,
                logging_settings_enabled: self.config.logging_settings_panel,
                language_settings_enabled: self.config.language_settings_panel,
                theme_settings_enabled: self.config.theme_settings_panel,
                theme: &mut self.config.appearance.theme,
                theme_editor: &mut self.theme_editor,
                application_font_path: self.config.appearance.font_path.as_deref(),
                dock_active: true,
                logging_directory: self.config.logging.directory.clone(),
                logging_file_prefix: self.config.logging.file_prefix.clone(),
                logging_retention: self.config.logging.retention_days,
                text_editors: &mut self.text_editors,
                text_editor_options: &mut self.text_editor_options,
                log_viewers: &mut self.log_viewers,
                tree_viewers: &mut self.tree_viewers,
                flow_editors: &mut self.flow_editors,
                property_panels: &mut self.property_panels,
                flow_property_links: &self.flow_property_links,
                controller_panels: &mut self.controller_panels,
                controller_property_links: &self.controller_property_links,
            };
            for (container_id, dock_state) in &mut self.workspace_docks.normal {
                if let Some(dock_state) = dock_state {
                    ui.push_id(container_id, |ui| {
                        let mut workspace_viewer = WorkspaceTabViewer { inner: &mut viewer };
                        egui_dock::DockArea::new(dock_state).show_inside(ui, &mut workspace_viewer);
                    });
                }
            }
            // Keep registry tab order aligned with user-driven tab rearrangement.
            // Do not recreate dock trees here: that would erase splits.
            let selected = self.workspace_registry.selected_id().to_owned();
            if let Err(error) = workspace_dock::sync_normal_tab_order(
                &mut self.workspace_registry,
                &selected,
                &self.workspace_docks,
            ) {
                tracing::warn!(target: "wfide::workspace", ?error, "dock tab reconciliation rejected");
            }
            // Capture floating window changes after rendering, then update the
            // authoritative registry and rebuild the projection on change.
            let selected_workspace = self.workspace_registry.selected_id().to_owned();
            let mut gestures = Vec::new();
            if let Some(workspace) = self.workspace_registry.get(&selected_workspace) {
                for (container_id, panel) in &self.workspace_docks.floating {
                    if let Some(geometry) = workspace.floating_geometry.get(container_id) {
                        let mut open = true;
                        let response =
                            egui::Window::new(format!("{}##{}", panel.definition_id, container_id))
                                .id(egui::Id::new((
                                    "workspace.floating",
                                    &selected_workspace,
                                    container_id,
                                )))
                                .default_pos(egui::pos2(geometry.position[0], geometry.position[1]))
                                .default_size(egui::vec2(geometry.size[0], geometry.size[1]))
                                .open(&mut open)
                                .show(ui.ctx(), |ui| {
                                    let mut key = panel.clone();
                                    let mut workspace_viewer =
                                        WorkspaceTabViewer { inner: &mut viewer };
                                    <WorkspaceTabViewer<'_, '_> as egui_dock::TabViewer>::ui(
                                        &mut workspace_viewer,
                                        ui,
                                        &mut key,
                                    );
                                });
                        if !open {
                            gestures.push(workspace_dock::DockGesture::Hide {
                                panel: workspace::PanelInstanceId::new(
                                    &panel.definition_id,
                                    &panel.instance_id,
                                ),
                            });
                        } else if let Some(response) = response {
                            let rect = response.response.rect;
                            let updated = workspace::FloatingGeometry {
                                position: [rect.min.x, rect.min.y],
                                size: [rect.width(), rect.height()],
                            };
                            if updated != *geometry {
                                gestures.push(workspace_dock::DockGesture::ResizeFloating {
                                    container: container_id.clone(),
                                    geometry: updated,
                                });
                            }
                        }
                    }
                }
            }
            if !gestures.is_empty() {
                for gesture in gestures {
                    if let Err(error) = workspace_dock::apply_dock_gesture(
                        &mut self.workspace_registry,
                        &selected_workspace,
                        gesture,
                    ) {
                        tracing::warn!(target: "wfide::workspace", ?error, "floating window gesture rejected");
                    }
                }
                if let Some(workspace) = self.workspace_registry.get(&selected_workspace) {
                    match workspace_dock::project_workspace(workspace) {
                        Ok(projection) => {
                            // Floating window updates must not flatten live normal
                            // dock trees, which may contain user-created splits.
                            if self
                                .workspace_docks
                                .normal
                                .keys()
                                .eq(projection.normal.keys())
                            {
                                self.workspace_docks.floating = projection.floating;
                            } else {
                                tracing::warn!(target: "wfide::workspace", "normal container set changed during floating update");
                            }
                        }
                        Err(error) => {
                            tracing::error!(target: "wfide::workspace", ?error, "workspace projection failed")
                        }
                    }
                }
            }
        } else if !self.config.panels.is_empty() {
            ui.separator();
            ui.label("Declared panels:");
            for panel in &self.config.panels {
                ui.label(format!(
                    "{} [{}] {:?} visible={}",
                    localization::text(&panel.title_key),
                    panel.id,
                    panel.kind,
                    panel.initially_visible
                ));
            }
        }
    }
}

struct FrameworkTabViewer<'a> {
    panels: &'a [PanelDefinition],
    probe_enabled: bool,
    logging_settings_enabled: bool,
    language_settings_enabled: bool,
    theme_settings_enabled: bool,
    theme: &'a mut theme::Theme,
    theme_editor: &'a mut Option<theme::ThemeEditor>,
    application_font_path: Option<&'a std::path::Path>,
    dock_active: bool,
    logging_directory: std::path::PathBuf,
    logging_file_prefix: String,
    logging_retention: usize,
    text_editors: &'a mut std::collections::HashMap<String, text_editor::TextDocument>,
    text_editor_options: &'a mut std::collections::HashMap<String, text_editor::TextEditorOptions>,
    log_viewers: &'a mut std::collections::HashMap<String, log_viewer::LogViewerOptions>,
    tree_viewers: &'a mut std::collections::HashMap<String, tree_viewer::TreeModel>,
    flow_editors: &'a mut std::collections::HashMap<String, flow_editor::FlowModel>,
    property_panels: &'a mut std::collections::HashMap<String, property_panel::PropertyModel>,
    flow_property_links: &'a [FlowPropertyLink],
    controller_panels: &'a mut std::collections::HashMap<String, controller_panel::ControllerModel>,
    controller_property_links: &'a [ControllerPropertyLink],
}

/// Instance-aware renderer for Workspace-owned DockState trees.
///
/// Panel content remains keyed by definition ID until per-instance model
/// storage is introduced; tab identity is always the full instance key.
struct WorkspaceTabViewer<'a, 'b> {
    inner: &'a mut FrameworkTabViewer<'b>,
}

impl egui_dock::TabViewer for WorkspaceTabViewer<'_, '_> {
    type Tab = workspace_dock::DockPanelKey;

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(("workspace.panel", &tab.definition_id, &tab.instance_id))
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        <FrameworkTabViewer<'_> as egui_dock::TabViewer>::title(self.inner, &mut tab.definition_id)
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        <FrameworkTabViewer<'_> as egui_dock::TabViewer>::ui(
            self.inner,
            ui,
            &mut tab.definition_id,
        );
    }

    fn is_closeable(&self, _tab: &Self::Tab) -> bool {
        false
    }
}

impl egui_dock::TabViewer for FrameworkTabViewer<'_> {
    type Tab = String;

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(tab.as_str())
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        if let Some(panel) = self.panels.iter().find(|panel| panel.id == *tab) {
            return localization::text(&panel.title_key).into();
        }
        match tab.as_str() {
            "__wfide_language_settings" => localization::text("language.title").into(),
            "__wfide_logging_settings" => localization::text("logging.title").into(),
            "__wfide_theme_settings" => localization::text("theme.title").into(),
            _ => tab.as_str().into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        if self.theme_settings_enabled && tab == "__wfide_theme_settings" {
            if theme::show_settings(ui, self.theme, self.theme_editor) {
                apply_theme_mode(ui.ctx(), *self.theme);
                let system_theme = match self.theme.is_dark() {
                    Some(true) => egui::SystemTheme::Dark,
                    Some(false) => egui::SystemTheme::Light,
                    None => egui::SystemTheme::SystemDefault,
                };
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::SetTheme(system_theme));
                *self.theme_editor = None;
                tracing::info!(target: "wfide::theme", theme = ?self.theme, "theme changed");
            }
            return;
        }

        if self.language_settings_enabled && tab == "__wfide_language_settings" {
            ui.heading(localization::text("language.title"));
            ui.label(localization::text("language.description"));
            ui.separator();

            let current = localization::current_locale();
            for (locale, name) in localization::locales() {
                let selected = current == locale;
                if ui.radio(selected, name).clicked() && !selected {
                    if let Err(error) = localization::set_locale(&locale) {
                        tracing::error!(target: "wfide::i18n", %error, %locale, "failed to change locale");
                    } else {
                        match locale_font::install_for_locale(
                            ui.ctx(),
                            &locale,
                            self.application_font_path,
                        ) {
                            Ok(paths) if !paths.is_empty() => {
                                tracing::info!(target: "wfide::font", %locale, count = paths.len(), "fallback fonts loaded")
                            }
                            Ok(_) => {
                                tracing::warn!(target: "wfide::font", %locale, "no OS or locale fallback font found")
                            }
                            Err(error) => {
                                tracing::warn!(target: "wfide::font", %locale, %error, "failed to configure locale font")
                            }
                        }
                    }
                }
            }
            return;
        }

        if self.logging_settings_enabled && tab == "__wfide_logging_settings" {
            ui.heading(localization::text("logging.title"));
            ui.label(localization::text("logging.framework_panel"));
            ui.separator();

            let current = logging::level();
            ui.label(localization::text("logging.runtime_level"));
            ui.label(localization::text("logging.description"));
            ui.label(localization::text("logging.error"));
            ui.label(localization::text("logging.warn"));
            ui.label(localization::text("logging.info"));
            ui.label(localization::text("logging.debug"));
            ui.label(localization::text("logging.trace"));
            ui.label(localization::text("logging.load_warning"));
            ui.add_space(4.0);
            for level in [
                logging::LogLevel::Error,
                logging::LogLevel::Warn,
                logging::LogLevel::Info,
                logging::LogLevel::Debug,
                logging::LogLevel::Trace,
            ] {
                let selected = current == level;
                if ui.radio(selected, format!("{level:?}")).clicked()
                    && !selected
                    && let Err(error) = logging::set_level(level)
                {
                    tracing::error!(target: "wfide::logging", %error, "failed to change runtime log level");
                }
            }

            ui.separator();
            ui.label(format!(
                "File directory: {}",
                self.logging_directory.display()
            ));
            ui.label(format!("File prefix: {}", self.logging_file_prefix));
            ui.label("Rotation: Daily");
            ui.label(format!("Retention: {} files", self.logging_retention));
            ui.label("File settings are startup configuration in v0.1.0.");
            return;
        }

        if self.probe_enabled && tab == probe::PANEL_ID {
            ui.heading("WFIDE Probe");
            ui.label("Framework maintenance diagnostics; not a Consumer panel.");
            ui.separator();
            let model = probe::table_model(true, self.dock_active);
            table::show(ui, "wfide_probe_table", &model);
            return;
        }

        if let Some(model) = self.controller_panels.get_mut(tab) {
            let response = controller_panel::show(ui, model);
            for action in response.actions {
                for link in self
                    .controller_property_links
                    .iter()
                    .filter(|link| link.controller_panel_id == *tab)
                {
                    match &action {
                        controller_panel::ControllerAction::ElementSelected { element_id }
                        | controller_panel::ControllerAction::ElementMoved { element_id, .. }
                        | controller_panel::ControllerAction::ElementResized {
                            element_id, ..
                        }
                        | controller_panel::ControllerAction::LineChanged { element_id } => {
                            if let Some(element) = model
                                .elements
                                .iter()
                                .find(|element| element.id == *element_id)
                            {
                                self.property_panels.insert(
                                    link.property_panel_id.clone(),
                                    controller_panel::property_model_for_element(element),
                                );
                            }
                        }
                        controller_panel::ControllerAction::CanvasSelected
                        | controller_panel::ControllerAction::CanvasResized { .. } => {
                            self.property_panels.insert(
                                link.property_panel_id.clone(),
                                controller_panel::property_model_for_canvas(model),
                            );
                        }
                        _ => {}
                    }
                }
                match &action {
                    controller_panel::ControllerAction::SliderChanged { element_id, .. }
                    | controller_panel::ControllerAction::JoystickChanged { element_id, .. } => {
                        for link in self
                            .controller_property_links
                            .iter()
                            .filter(|link| link.controller_panel_id == *tab)
                        {
                            let should_refresh = self
                                .property_panels
                                .get(&link.property_panel_id)
                                .and_then(|property| property.object_id.as_deref())
                                == Some(element_id.as_str());
                            if should_refresh
                                && let Some(element) = model
                                    .elements
                                    .iter()
                                    .find(|element| element.id == *element_id)
                            {
                                self.property_panels.insert(
                                    link.property_panel_id.clone(),
                                    controller_panel::property_model_for_element(element),
                                );
                            }
                        }
                    }
                    _ => {}
                }
                tracing::info!(target: "wfide::controller_panel", panel_id = %tab, action = ?action, "controller action");
            }
            return;
        }

        if let Some(model) = self.property_panels.get_mut(tab) {
            let response = property_panel::show(ui, model);
            for action in response.actions {
                for link in self
                    .flow_property_links
                    .iter()
                    .filter(|link| link.property_panel_id == *tab)
                {
                    if let Some(flow) = self.flow_editors.get_mut(&link.flow_panel_id)
                        && let Some(object_id) = model.object_id.as_deref()
                        && let Some(node) = flow.nodes.iter_mut().find(|node| node.id == object_id)
                    {
                        property_panel::apply_to_flow_node(node, &action);
                    }
                }
                for link in self
                    .controller_property_links
                    .iter()
                    .filter(|link| link.property_panel_id == *tab)
                {
                    if let Some(controller) =
                        self.controller_panels.get_mut(&link.controller_panel_id)
                        && let Some(object_id) = model.object_id.as_deref()
                    {
                        if object_id == "__controller_canvas__" {
                            controller_panel::apply_canvas_property_action(controller, &action);
                        } else if let Some(element) = controller
                            .elements
                            .iter_mut()
                            .find(|element| element.id == object_id)
                        {
                            controller_panel::apply_property_action(element, &action);
                        }
                    }
                }
                tracing::info!(
                    target: "wfide::property_panel",
                    panel_id = %tab,
                    action = ?action,
                    "property changed"
                );
            }
            return;
        }

        if let Some(model) = self.flow_editors.get_mut(tab) {
            let response = flow_editor::show(ui, model);
            for action in response.actions {
                for link in self
                    .flow_property_links
                    .iter()
                    .filter(|link| link.flow_panel_id == *tab)
                {
                    let next = match &action {
                        flow_editor::FlowAction::NodeSelected { node_id } => model
                            .nodes
                            .iter()
                            .find(|node| node.id == *node_id)
                            .map(property_panel::from_flow_node),
                        flow_editor::FlowAction::EdgeSelected { edge_id } => model
                            .edges
                            .iter()
                            .find(|edge| edge.id == *edge_id)
                            .map(property_panel::from_flow_edge),
                        flow_editor::FlowAction::SelectionCleared => {
                            Some(property_panel::PropertyModel::default())
                        }
                        _ => None,
                    };
                    if matches!(&action, flow_editor::FlowAction::SelectionCleared) {
                        self.property_panels.insert(
                            link.property_panel_id.clone(),
                            property_panel::PropertyModel::default(),
                        );
                    } else if let Some(next) = next {
                        self.property_panels
                            .insert(link.property_panel_id.clone(), next);
                    }
                }
                tracing::debug!(
                    target: "wfide::flow_editor",
                    panel_id = %tab,
                    action = ?action,
                    "flow editor action"
                );
            }
            return;
        }

        if let Some(model) = self.tree_viewers.get_mut(tab) {
            let response = tree_viewer::show(ui, model);
            for action in response.actions {
                match action {
                    tree_viewer::TreeAction::Selected { node_id } => {
                        tracing::info!(
                            target: "wfide::tree_viewer",
                            panel_id = %tab,
                            %node_id,
                            "tree node selected"
                        );
                    }
                    tree_viewer::TreeAction::SelectionChanged { .. }
                    | tree_viewer::TreeAction::Activated { .. }
                    | tree_viewer::TreeAction::ContextRequested { .. } => {}
                }
            }
            return;
        }

        if let Some(options) = self.log_viewers.get_mut(tab) {
            log_viewer::show(ui, options);
            return;
        }

        if let Some(document) = self.text_editors.get_mut(tab) {
            let options = self.text_editor_options.entry(tab.clone()).or_default();
            let response = text_editor::show_with_options(ui, document, options);
            if let Some(text_editor::TextEditorAction::SaveRequested) = response.action {
                tracing::info!(
                    target: "wfide::text_editor",
                    panel_id = %tab,
                    document_id = %document.id,
                    "text editor save requested"
                );
            }
            return;
        }

        if let Some(panel) = self.panels.iter().find(|panel| panel.id == *tab) {
            ui.heading(localization::text(&panel.title_key));
            ui.label(format!("Panel ID: {}", panel.id));
            ui.label(format!("Panel kind: {:?}", panel.kind));
            ui.label("Step 4 layout placeholder; panel content is implemented in a later step.");
        } else {
            ui.label(format!("Unknown panel: {tab}"));
        }
    }

    fn is_closeable(&self, _tab: &Self::Tab) -> bool {
        false
    }
}

pub mod resource_executor;

pub mod resource_recovery;
