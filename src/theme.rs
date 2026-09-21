use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    System,
    Dark,
    Light,
}

impl Theme {
    pub fn apply(self, ctx: &egui::Context) {
        match self {
            Self::System => {
                // eframe/egui initializes visuals from the native integration.
                // Runtime System selection follows the current native preference
                // when it is available through the viewport.
                let dark = ctx.input(|i| i.viewport().native_pixels_per_point.is_some())
                    && ctx.style().visuals.dark_mode;
                ctx.set_visuals(if dark { egui::Visuals::dark() } else { egui::Visuals::light() });
            }
            Self::Dark => ctx.set_visuals(egui::Visuals::dark()),
            Self::Light => ctx.set_visuals(egui::Visuals::light()),
        }
    }
}

pub fn show_settings(ui: &mut egui::Ui, current: &mut Theme) -> bool {
    ui.heading(crate::localization::text("theme.title"));
    ui.label(crate::localization::text("theme.description"));
    ui.separator();

    let mut changed = false;
    for (theme, key) in [
        (Theme::System, "theme.system"),
        (Theme::Dark, "theme.dark"),
        (Theme::Light, "theme.light"),
    ] {
        let selected = *current == theme;
        if ui.radio(selected, crate::localization::text(key)).clicked() && !selected {
            *current = theme;
            changed = true;
        }
    }
    changed
}
