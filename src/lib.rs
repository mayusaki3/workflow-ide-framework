use eframe::egui;
pub mod logging;
pub mod layout;
pub mod localization;
pub mod probe;
pub mod table;
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
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            ui_scale: None,
            font_path: None,
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
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "wfide_application_font".to_owned());
    ctx.set_fonts(fonts);
    Ok(())
}

pub struct Application {
    config: ApplicationConfig,
}

impl Application {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::with_config(ApplicationConfig::new(id, name))
    }

    pub fn with_config(config: ApplicationConfig) -> Self {
        Self { config }
    }

    pub fn panel(mut self, panel: PanelDefinition) -> Self {
        self.config.panels.push(panel);
        self
    }

    pub fn layout(mut self, layout: LayoutConfig) -> Self {
        self.config.layout = Some(layout);
        self
    }

    /// Enables the Framework-maintenance Probe Panel.
    /// Consumer applications should not enable this in normal builds.
    pub fn framework_probe_panel(mut self) -> Self {
        self.config.probe_panel = true;
        self
    }

    /// Enables the Framework standard Logging Settings Panel.
    pub fn logging_settings_panel(mut self) -> Self {
        self.config.logging_settings_panel = true;
        self
    }

    /// Enables the Framework standard Language Settings Panel.
    pub fn language_settings_panel(mut self) -> Self {
        self.config.language_settings_panel = true;
        self
    }

    pub fn run(self) -> eframe::Result<()> {
        let mut config = self.config;
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

                Ok(Box::new(FrameworkHost { config, dock_state }))
            }),
        )
    }
}

struct FrameworkHost {
    config: ApplicationConfig,
    dock_state: Option<egui_dock::DockState<String>>,
}

impl eframe::App for FrameworkHost {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
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
                dock_active: true,
                logging_directory: self.config.logging.directory.clone(),
                logging_file_prefix: self.config.logging.file_prefix.clone(),
                logging_retention: self.config.logging.retention_days,
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
    dock_active: bool,
    logging_directory: std::path::PathBuf,
    logging_file_prefix: String,
    logging_retention: usize,
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
            ui.label(format!("Rotation: Daily"));
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
