use eframe::egui;

#[derive(Debug, Clone)]
pub struct ApplicationConfig {
    pub id: String,
    pub name: String,
    pub window: WindowConfig,
    pub appearance: AppearanceConfig,
}

impl ApplicationConfig {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            window: WindowConfig::default(),
            appearance: AppearanceConfig::default(),
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
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self { ui_scale: None }
    }
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

    pub fn run(self) -> eframe::Result<()> {
        let config = self.config;
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

                Ok(Box::new(FrameworkHost { config }))
            }),
        )
    }
}

struct FrameworkHost {
    config: ApplicationConfig,
}

impl eframe::App for FrameworkHost {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading(&self.config.name);
        ui.label(format!("Application ID: {}", self.config.id));
        ui.label("workflow-ide-framework v0.1.0 Step 1 sample");
    }
}
