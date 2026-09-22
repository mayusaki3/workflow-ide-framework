use eframe::egui;
pub mod flow_editor;
pub mod logging;
pub mod log_viewer;
pub mod layout;
pub mod localization;
pub mod probe;
pub mod table;
pub mod text_editor;
pub mod tree_viewer;
pub mod theme;
pub use layout::{LayoutConfig, SplitDirection};
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
    pub name: String,
    pub kind: PanelKind,
    pub initially_visible: bool,
}

impl PanelDefinition {
    pub fn new(id: impl Into<String>, name: impl Into<String>, kind: PanelKind) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
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

fn install_application_font(
    ctx: &egui::Context,
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bytes = std::fs::read(path)?;
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "wfide_application_font".to_owned(),
        egui::FontData::from_owned(bytes).into(),
    );
    for family in [
        egui::FontFamily::Proportional,
        egui::FontFamily::Monospace,
    ] {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, "wfide_application_font".to_owned());
    }
    ctx.set_fonts(fonts);
    Ok(())
}

pub struct Application {
    config: ApplicationConfig,
    text_editors: std::collections::HashMap<String, text_editor::TextDocument>,
    text_editor_options: std::collections::HashMap<String, text_editor::TextEditorOptions>,
    log_viewers: std::collections::HashMap<String, log_viewer::LogViewerOptions>,
    tree_viewers: std::collections::HashMap<String, tree_viewer::TreeModel>,
    flow_editors: std::collections::HashMap<String, flow_editor::FlowModel>,
    theme_editor: Option<theme::ThemeEditor>,
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
            theme_editor: None,
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

    pub fn language_settings_panel(mut self) -> Self {
        self.config.language_settings_panel = true;
        self
    }

    pub fn theme_settings_panel(mut self) -> Self {
        self.config.theme_settings_panel = true;
        self
    }

    pub fn run(self) -> eframe::Result<()> {
        let mut config = self.config;
        let text_editors = self.text_editors;
        let text_editor_options = self.text_editor_options;
        let log_viewers = self.log_viewers;
        let tree_viewers = self.tree_viewers;
        let flow_editors = self.flow_editors;
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
                PanelDefinition::new("__wfide_logging_settings", "Logging Settings", PanelKind::StandardUi)
            );
            if let Some(layout) = &mut config.layout {
                layout.root_panel_ids.push("__wfide_logging_settings".to_owned());
            }
        }
        if config.language_settings_panel && !config.panels.iter().any(|panel| panel.id == "__wfide_language_settings") {
            config.panels.push(
                PanelDefinition::new("__wfide_language_settings", "Language Settings", PanelKind::StandardUi)
            );
            if let Some(layout) = &mut config.layout {
                layout.root_panel_ids.push("__wfide_language_settings".to_owned());
            }
        }
        if config.theme_settings_panel && !config.panels.iter().any(|panel| panel.id == "__wfide_theme_settings") {
            config.panels.push(
                PanelDefinition::new("__wfide_theme_settings", "Theme Settings", PanelKind::StandardUi)
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
                if let Some(font_path) = config.appearance.font_path.as_deref() {
                    if let Err(error) = install_application_font(&cc.egui_ctx, font_path) {
                        tracing::warn!(
                            target: "wfide::font",
                            path = %font_path.display(),
                            %error,
                            "failed to load application font; using egui defaults"
                        );
                    } else {
                        tracing::info!(
                            target: "wfide::font",
                            path = %font_path.display(),
                            "application font loaded"
                        );
                    }
                }

                Ok(Box::new(FrameworkHost {
                    config,
                    dock_state,
                    text_editors,
                    text_editor_options,
                    log_viewers,
                    tree_viewers,
                    flow_editors,
                    theme_editor: None,
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
    theme_editor: Option<theme::ThemeEditor>,
}

impl eframe::App for FrameworkHost {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
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

        // Theme Editor is an override layer on top of the selected base theme.
        // In System mode the base palette is refreshed every frame to follow
        // the OS, so re-apply the editor override afterwards.
        if let Some(editor) = self.theme_editor {
            editor.apply(ui.ctx());
        }

        // eframe supplies the root Ui before App::ui is called. A runtime
        // theme change therefore updates Context styles immediately, but this
        // already-created Ui can still hold the previous frame's style.
        // Refresh the root Ui style so the application header follows the same
        // theme in the same frame as Dock/Panel content.
        ui.set_style(ui.ctx().global_style());
        // The root Ui background was already painted by eframe before App::ui.
        // Repaint the application area with the active theme so runtime
        // changes affect the header as well as subsequently-created panels.
        ui.painter().rect_filled(
            ui.max_rect(),
            0.0,
            ui.visuals().panel_fill,
        );

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
                dock_active: true,
                logging_directory: self.config.logging.directory.clone(),
                logging_file_prefix: self.config.logging.file_prefix.clone(),
                logging_retention: self.config.logging.retention_days,
                text_editors: &mut self.text_editors,
                text_editor_options: &mut self.text_editor_options,
                log_viewers: &mut self.log_viewers,
                tree_viewers: &mut self.tree_viewers,
                flow_editors: &mut self.flow_editors,
            };
            egui_dock::DockArea::new(dock_state).show_inside(ui, &mut viewer);
        } else if !self.config.panels.is_empty() {
            ui.separator();
            ui.label("Declared panels:");
            for panel in &self.config.panels {
                ui.label(format!(
                    "{} [{}] {:?} visible={}",
                    panel.name, panel.id, panel.kind, panel.initially_visible
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
    dock_active: bool,
    logging_directory: std::path::PathBuf,
    logging_file_prefix: String,
    logging_retention: usize,
    text_editors: &'a mut std::collections::HashMap<String, text_editor::TextDocument>,
    text_editor_options: &'a mut std::collections::HashMap<String, text_editor::TextEditorOptions>,
    log_viewers: &'a mut std::collections::HashMap<String, log_viewer::LogViewerOptions>,
    tree_viewers: &'a mut std::collections::HashMap<String, tree_viewer::TreeModel>,
    flow_editors: &'a mut std::collections::HashMap<String, flow_editor::FlowModel>,
}

impl egui_dock::TabViewer for FrameworkTabViewer<'_> {
    type Tab = String;

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(tab.as_str())
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        self.panels
            .iter()
            .find(|panel| panel.id == *tab)
            .map(|panel| panel.name.as_str())
            .unwrap_or(tab.as_str())
            .into()
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

        if let Some(model) = self.flow_editors.get_mut(tab) {
            let response = flow_editor::show(ui, model);
            for action in response.actions {
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
