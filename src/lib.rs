use eframe::egui;

pub struct Application {
    id: String,
    name: String,
}

impl Application {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }

    pub fn run(self) -> eframe::Result<()> {
        let window_title = self.name.clone();
        let app_name = self.name;
        let app_id = self.id;

        eframe::run_native(
            &window_title,
            eframe::NativeOptions::default(),
            Box::new(move |_cc| {
                Ok(Box::new(FrameworkHost {
                    app_id,
                    app_name,
                }))
            }),
        )
    }
}

struct FrameworkHost {
    app_id: String,
    app_name: String,
}

impl eframe::App for FrameworkHost {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading(&self.app_name);
        ui.label(format!("Application ID: {}", self.app_id));
        ui.label("workflow-ide-framework v0.1.0 Step 1 sample");
    }
}
