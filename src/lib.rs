use eframe::egui;
pub mod controller_panel;
pub mod flow_editor;
pub mod logging;
pub mod log_viewer;
pub mod layout;
pub mod localization;
pub mod locale_font;
pub mod probe;
pub mod property_panel;
pub mod project_resource;
pub mod project;
pub mod project_lifecycle;
pub mod project_adapter;
pub mod project_adapter_erased;
pub mod project_controller;
pub mod project_io;
pub mod framework_settings;
pub mod project_save;
pub mod project_open;
pub mod project_save_as;
pub mod resource_registry;
pub mod resource_state;
pub mod file_resource_selector;
pub mod resource_selection_registry;
pub mod locate_replace;
pub mod resource_operation;
pub mod resource_journal;
pub mod table;
pub mod text_editor;
pub mod tree_viewer;
pub mod theme;
pub use layout::{LayoutConfig, LayoutSplit, SplitDirection};
pub use tracing;


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

#[derive(Debug, Clone)]
pub struct ApplicationConfig {
    pub id: String,
    pub name: String,
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

#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: Option<String>,
    pub initial_size: Option<[f32; 2]>,
    pub min_size: Option<[f32; 2]>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: None,
            initial_size: None,
            min_size: None,
        }
    }
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
    config: ApplicationConfig,
    text_editors: std::collections::HashMap<String, text_editor::TextDocument>,
    text_editor_options: std::collections::HashMap<String, text_editor::TextEditorOptions>,
    log_viewers: std::collections::HashMap<String, log_viewer::LogViewerOptions>,
    tree_viewers: std::collections::HashMap<String, tree_viewer::TreeModel>,
    flow_editors: std::collections::HashMap<String, flow_editor::FlowModel>,
    property_panels: std::collections::HashMap<String, property_panel::PropertyModel>,
    flow_property_links: Vec<FlowPropertyLink>,
    controller_panels: std::collections::HashMap<String, controller_panel::ControllerModel>,
    controller_property_links: Vec<ControllerPropertyLink>,
    project_adapter: Option<Box<dyn project_adapter_erased::ErasedApplicationProjectAdapter>>,
}

impl Application {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::with_config(ApplicationConfig::new(id, name))
    }

    pub fn with_config(config: ApplicationConfig) -> Self {
        Self {
            config,
            text_editors: std::collections::HashMap::new(),
            text_editor_options: std::collections::HashMap::new(),
            log_viewers: std::collections::HashMap::new(),
            tree_viewers: std::collections::HashMap::new(),
            flow_editors: std::collections::HashMap::new(),
            property_panels: std::collections::HashMap::new(),
            flow_property_links: Vec::new(),
            controller_panels: std::collections::HashMap::new(),
            controller_property_links: Vec::new(),
            project_adapter: None,
        }
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
        self.text_editor_options.entry(panel_id.into()).or_default().ime_debug = enabled;
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
        self.config.localization.application_resource_directories.push(directory.into());
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
        let text_editors = self.text_editors;
        let text_editor_options = self.text_editor_options;
        let log_viewers = self.log_viewers;
        let tree_viewers = self.tree_viewers;
        let flow_editors = self.flow_editors;
        let property_panels = self.property_panels;
        let flow_property_links = self.flow_property_links;
        let controller_panels = self.controller_panels;
        let controller_property_links = self.controller_property_links;
        let project_adapter = self.project_adapter;
        if config.probe_panel && !config.panels.iter().any(|panel| panel.id == probe::PANEL_ID) {
            config.panels.push(
                PanelDefinition::new(probe::PANEL_ID, "WFIDE Probe", PanelKind::StandardUi)
            );
            if let Some(layout) = &mut config.layout {
                layout.root_panel_ids.push(probe::PANEL_ID.to_owned());
            }
        }
        if config.logging_settings_panel && !config.panels.iter().any(|panel| panel.id == "__wfide_logging_settings") {
            config.panels.push(
                PanelDefinition::new("__wfide_logging_settings", "logging.title", PanelKind::StandardUi)
            );
            if let Some(layout) = &mut config.layout {
                layout.root_panel_ids.push("__wfide_logging_settings".to_owned());
            }
        }
        if config.language_settings_panel && !config.panels.iter().any(|panel| panel.id == "__wfide_language_settings") {
            config.panels.push(
                PanelDefinition::new("__wfide_language_settings", "language.title", PanelKind::StandardUi)
            );
            if let Some(layout) = &mut config.layout {
                layout.root_panel_ids.push("__wfide_language_settings".to_owned());
            }
        }
        if config.theme_settings_panel && !config.panels.iter().any(|panel| panel.id == "__wfide_theme_settings") {
            config.panels.push(
                PanelDefinition::new("__wfide_theme_settings", "theme.title", PanelKind::StandardUi)
            );
            if let Some(layout) = &mut config.layout {
                layout.root_panel_ids.push("__wfide_theme_settings".to_owned());
            }
        }
        localization::init(&config.localization);
        let _logging_guard = logging::init(&config.id, &config.logging)
            .map_err(eframe::Error::AppCreation)?;
        tracing::info!(target: "wfide::application", application_id = %config.id, "WFIDE application starting");
        let dock_state = config
            .layout
            .as_ref()
            .map(|layout| layout::build_dock_state(layout, &config.panels))
            .transpose()
            .map_err(|error| eframe::Error::AppCreation(Box::new(error)))?;
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
                    Ok(paths) if !paths.is_empty() => tracing::info!(target: "wfide::font", %locale, count = paths.len(), "fallback fonts loaded"),
                    Ok(_) => tracing::warn!(target: "wfide::font", %locale, "no OS or locale fallback font found"),
                    Err(error) => tracing::warn!(target: "wfide::font", %locale, %error, "failed to configure fonts"),
                }

                let project_controller = project_adapter.as_ref().map(|_| {
                    project_controller::ProjectController::new(config.id.clone(), config.name.clone())
                });

                Ok(Box::new(FrameworkHost {
                    config,
                    dock_state,
                    text_editors,
                    text_editor_options,
                    log_viewers,
                    tree_viewers,
                    flow_editors,
                    property_panels,
                    flow_property_links,
                    controller_panels,
                    controller_property_links,
                    project_controller,
                    project_adapter,
                    project_ui: ProjectUiState::default(),
                    theme_editor: None,
                    fonts_initialized: false,
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
    config: ApplicationConfig,
    dock_state: Option<egui_dock::DockState<String>>,
    text_editors: std::collections::HashMap<String, text_editor::TextDocument>,
    text_editor_options: std::collections::HashMap<String, text_editor::TextEditorOptions>,
    log_viewers: std::collections::HashMap<String, log_viewer::LogViewerOptions>,
    tree_viewers: std::collections::HashMap<String, tree_viewer::TreeModel>,
    flow_editors: std::collections::HashMap<String, flow_editor::FlowModel>,
    property_panels: std::collections::HashMap<String, property_panel::PropertyModel>,
    flow_property_links: Vec<FlowPropertyLink>,
    controller_panels: std::collections::HashMap<String, controller_panel::ControllerModel>,
    controller_property_links: Vec<ControllerPropertyLink>,
    project_adapter: Option<Box<dyn project_adapter_erased::ErasedApplicationProjectAdapter>>,
    project_controller: Option<project_controller::ProjectController>,
    project_ui: ProjectUiState,
    theme_editor: Option<theme::ThemeEditor>,
    fonts_initialized: bool,
}

#[derive(Default)]
struct ProjectUiState {
    message: Option<String>,
    pending_open: Option<(project::ProjectContext, project_open::ProjectOpenResult)>,
    confirm_close: bool,
}

fn project_save_stamp() -> (String, String) {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQUENCE: AtomicU64 = AtomicU64::new(1);
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    (format!("save-{}-{}", now.as_nanos(), sequence), format!("unix:{}.{:09}", now.as_secs(), now.subsec_nanos()))
}

impl FrameworkHost {
    fn handle_project_result(&mut self, result: project_controller::ProjectCommandResult, open_context: Option<project::ProjectContext>) {
        use project_controller::ProjectCommandResult;
        match result {
            ProjectCommandResult::Completed => self.project_ui.message = None,
            ProjectCommandResult::NeedsSaveLocation => self.project_save_as(),
            ProjectCommandResult::NeedsOpenDecision(result) => {
                if let Some(context) = open_context { self.project_ui.pending_open = Some((context, result)); }
            }
            ProjectCommandResult::NeedsDirtyConfirmation => self.project_ui.confirm_close = true,
            ProjectCommandResult::Failed(error) => self.project_ui.message = Some(error),
        }
    }

    fn project_new(&mut self) {
        let (Some(controller), Some(adapter)) = (self.project_controller.as_mut(), self.project_adapter.as_deref_mut()) else { return; };
        let result = controller.new_project("Untitled", localization::current_locale(), project_resource::NewProjectStoragePolicy::Deferred, None, adapter);
        self.handle_project_result(result, None);
    }

    fn project_folder_dialog(&self) -> rfd::FileDialog {
        let parent = self.project_controller.as_ref()
            .and_then(|controller| controller.session.as_ref())
            .and_then(|session| session.context())
            .and_then(|context| context.root().parent());
        match parent {
            Some(parent) => rfd::FileDialog::new().set_directory(parent),
            None => rfd::FileDialog::new(),
        }
    }

    fn project_open(&mut self) {
        let Some(root) = self.project_folder_dialog().pick_folder() else { return; };
        let context = project::ProjectContext::new(root);
        let (Some(controller), Some(adapter)) = (self.project_controller.as_mut(), self.project_adapter.as_deref_mut()) else { return; };
        let result = controller.open(context.clone(), adapter);
        self.handle_project_result(result, Some(context));
    }

    fn project_save(&mut self) {
        let (save_id, saved_at) = project_save_stamp();
        let (Some(controller), Some(adapter)) = (self.project_controller.as_mut(), self.project_adapter.as_deref_mut()) else { return; };
        let result = controller.save(adapter, &save_id, &saved_at);
        self.handle_project_result(result, None);
    }

    fn project_save_as(&mut self) {
        let Some(parent) = self.project_folder_dialog().pick_folder() else { return; };
        let Some(project_name) = self.project_controller.as_ref().and_then(|controller| controller.project_name()).map(str::to_owned) else { return; };
        let root = parent.join(&project_name);
        if root.exists() {
            self.project_ui.message = Some(format!("Project folder already exists: {}", root.display()));
            return;
        }
        let (save_id, saved_at) = project_save_stamp();
        let (Some(controller), Some(adapter)) = (self.project_controller.as_mut(), self.project_adapter.as_deref_mut()) else { return; };
        let result = controller.save_as(project::ProjectContext::new(root), adapter, &save_id, &saved_at);
        self.handle_project_result(result, None);
    }

    fn project_close(&mut self, confirmed: bool) {
        let Some(controller) = self.project_controller.as_mut() else { return; };
        let result = controller.close(confirmed);
        self.handle_project_result(result, None);
    }

    fn project_menu(&mut self, ui: &mut egui::Ui) {
        if self.project_controller.is_none() { return; }
        let frame = egui::Frame::new()
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(egui::Margin::symmetric(6, 3))
            .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color));
        frame.show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button(localization::text("project.menu"), |ui| {
                if ui.button(localization::text("project.new")).clicked() { ui.close(); self.project_new(); }
                if ui.button(localization::text("project.open")).clicked() { ui.close(); self.project_open(); }
                let is_open = self.project_controller.as_ref().is_some_and(|c| c.is_open());
                ui.add_enabled_ui(is_open, |ui| {
                    if ui.button(localization::text("project.save")).clicked() { ui.close(); self.project_save(); }
                    if ui.button(localization::text("project.save_as")).clicked() { ui.close(); self.project_save_as(); }
                    if ui.button(localization::text("project.close")).clicked() { ui.close(); self.project_close(false); }
                });
            });
            });
        });
    }

    fn project_dialogs(&mut self, ctx: &egui::Context) {
        if self.project_ui.confirm_close {
            egui::Window::new("Unsaved changes").collapsible(false).resizable(false).show(ctx, |ui| {
                ui.label("This project has unsaved changes. Close it without saving?");
                ui.horizontal(|ui| {
                    if ui.button("Close without saving").clicked() { self.project_ui.confirm_close = false; self.project_close(true); }
                    if ui.button("Cancel").clicked() { self.project_ui.confirm_close = false; }
                });
            });
        }
        if self.project_ui.pending_open.is_some() {
            egui::Window::new("Project requires attention").collapsible(false).resizable(false).show(ctx, |ui| {
                ui.label("The project has compatibility, consistency, framework settings, or pending resource-operation information that requires a decision.");
                let can_continue = self.project_ui.pending_open.as_ref().is_some_and(|(_, result)| project_open::open_can_continue(result));
                ui.horizontal(|ui| {
                    if ui.add_enabled(can_continue, egui::Button::new("Open with recovery defaults")).clicked() {
                        if let Some((context, result)) = self.project_ui.pending_open.take() {
                            if let Some(controller) = self.project_controller.as_mut() {
                                let outcome = controller.accept_open(context, result);
                                self.handle_project_result(outcome, None);
                            }
                        }
                    }
                    if ui.button("Cancel").clicked() { self.project_ui.pending_open = None; }
                });
            });
        }
        if let Some(message) = self.project_ui.message.clone() {
            egui::Window::new("Project error").collapsible(false).resizable(false).show(ctx, |ui| {
                ui.label(message);
                if ui.button("OK").clicked() { self.project_ui.message = None; }
            });
        }
    }
}

impl eframe::App for FrameworkHost {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if !self.fonts_initialized {
            let locale = localization::current_locale();
            match locale_font::install_for_locale(
                ui.ctx(),
                &locale,
                self.config.appearance.font_path.as_deref(),
            ) {
                Ok(paths) if !paths.is_empty() => tracing::info!(target: "wfide::font", %locale, count = paths.len(), "initial-frame fallback fonts loaded"),
                Ok(_) => tracing::warn!(target: "wfide::font", %locale, "no OS or locale fallback font found on initial frame"),
                Err(error) => tracing::warn!(target: "wfide::font", %locale, %error, "failed to configure fonts on initial frame"),
            }
            self.fonts_initialized = true;
            ui.ctx().request_repaint();
        }
        if self.config.appearance.theme == theme::Theme::System {
            let system_dark = ui
                .ctx()
                .system_theme()
                .unwrap_or(egui::Theme::Dark)
                == egui::Theme::Dark;
            self.config
                .appearance
                .theme
                .apply_with_system_dark(ui.ctx(), system_dark);
        }

        if let Some(editor) = self.theme_editor {
            editor.apply(ui.ctx());
        }

        ui.set_style(ui.ctx().global_style());
        ui.painter().rect_filled(
            ui.max_rect(),
            0.0,
            ui.visuals().panel_fill,
        );

        self.project_menu(ui);
        self.project_dialogs(ui.ctx());

        let application_title = self.config.window.title.as_deref().unwrap_or(&self.config.name);
        let window_title = self.project_controller.as_ref()
            .and_then(|controller| controller.project_name())
            .map(|project_name| format!("{application_title} - {project_name}"))
            .unwrap_or_else(|| application_title.to_owned());
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Title(window_title));

        ui.heading(&self.config.name);
        ui.label(format!("Application ID: {}", self.config.id));
        ui.label("workflow-ide-framework v0.1.0 Sample");

        if let Some(dock_state) = &mut self.dock_state {
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
            egui_dock::DockArea::new(dock_state).show_inside(ui, &mut viewer);
        } else if !self.config.panels.is_empty() {
            ui.separator();
            ui.label("Declared panels:");
            for panel in &self.config.panels {
                ui.label(format!(
                    "{} [{}] {:?} visible={}",
                    localization::text(&panel.title_key), panel.id, panel.kind, panel.initially_visible
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
                        match locale_font::install_for_locale(ui.ctx(), &locale, self.application_font_path) {
                            Ok(paths) if !paths.is_empty() => tracing::info!(target: "wfide::font", %locale, count = paths.len(), "fallback fonts loaded"),
                            Ok(_) => tracing::warn!(target: "wfide::font", %locale, "no OS or locale fallback font found"),
                            Err(error) => tracing::warn!(target: "wfide::font", %locale, %error, "failed to configure locale font"),
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
                if ui.radio(selected, format!("{level:?}")).clicked() && !selected {
                    if let Err(error) = logging::set_level(level) {
                        tracing::error!(target: "wfide::logging", %error, "failed to change runtime log level");
                    }
                }
            }

            ui.separator();
            ui.label(format!("File directory: {}", self.logging_directory.display()));
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
                for link in self.controller_property_links.iter().filter(|link| link.controller_panel_id == *tab) {
                    match &action {
                        controller_panel::ControllerAction::ElementSelected { element_id }
                        | controller_panel::ControllerAction::ElementMoved { element_id, .. }
                        | controller_panel::ControllerAction::ElementResized { element_id, .. }
                        | controller_panel::ControllerAction::LineChanged { element_id } => {
                            if let Some(element) = model.elements.iter().find(|element| element.id == *element_id) {
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
                        for link in self.controller_property_links.iter().filter(|link| link.controller_panel_id == *tab) {
                            let should_refresh = self.property_panels.get(&link.property_panel_id)
                                .and_then(|property| property.object_id.as_deref())
                                == Some(element_id.as_str());
                            if should_refresh {
                                if let Some(element) = model.elements.iter().find(|element| element.id == *element_id) {
                                    self.property_panels.insert(
                                        link.property_panel_id.clone(),
                                        controller_panel::property_model_for_element(element),
                                    );
                                }
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
                for link in self.flow_property_links.iter().filter(|link| link.property_panel_id == *tab) {
                    if let Some(flow) = self.flow_editors.get_mut(&link.flow_panel_id) {
                        if let Some(object_id) = model.object_id.as_deref() {
                            if let Some(node) = flow.nodes.iter_mut().find(|node| node.id == object_id) {
                                property_panel::apply_to_flow_node(node, &action);
                            }
                        }
                    }
                }
                for link in self.controller_property_links.iter().filter(|link| link.property_panel_id == *tab) {
                    if let Some(controller) = self.controller_panels.get_mut(&link.controller_panel_id) {
                        if let Some(object_id) = model.object_id.as_deref() {
                            if object_id == "__controller_canvas__" {
                                controller_panel::apply_canvas_property_action(controller, &action);
                            } else if let Some(element) = controller.elements.iter_mut().find(|element| element.id == object_id) {
                                controller_panel::apply_property_action(element, &action);
                            }
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
                for link in self.flow_property_links.iter().filter(|link| link.flow_panel_id == *tab) {
                    let next = match &action {
                        flow_editor::FlowAction::NodeSelected { node_id } =>
                            model.nodes.iter().find(|node| node.id == *node_id).map(property_panel::from_flow_node),
                        flow_editor::FlowAction::EdgeSelected { edge_id } =>
                            model.edges.iter().find(|edge| edge.id == *edge_id).map(property_panel::from_flow_edge),
                        flow_editor::FlowAction::SelectionCleared => Some(property_panel::PropertyModel::default()),
                        _ => None,
                    };
                    if matches!(&action, flow_editor::FlowAction::SelectionCleared) {
                        self.property_panels.insert(link.property_panel_id.clone(), property_panel::PropertyModel::default());
                    } else if let Some(next) = next {
                        self.property_panels.insert(link.property_panel_id.clone(), next);
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
            ui.heading(&panel.name);
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
