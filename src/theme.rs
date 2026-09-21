use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    System,
    Dark,
    Light,
    Sakura,
    Izumi,
    Ao,
}

impl Theme {
    pub fn apply(self, ctx: &egui::Context) {
        match self {
            Self::System => {
                // Temporary v0.1.0 behavior. Native OS theme tracking is handled separately.
                let dark = ctx.style().visuals.dark_mode;
                ctx.set_visuals(if dark { egui::Visuals::dark() } else { egui::Visuals::light() });
            }
            Self::Dark => ctx.set_visuals(egui::Visuals::dark()),
            Self::Light => ctx.set_visuals(egui::Visuals::light()),
            Self::Sakura => ctx.set_visuals(sakura_light()),
            Self::Izumi => ctx.set_visuals(izumi_light()),
            Self::Ao => ctx.set_visuals(ao_light()),
        }
    }
}

fn light_palette(background: egui::Color32, panel: egui::Color32, accent: egui::Color32) -> egui::Visuals {
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = panel;
    visuals.window_fill = panel;
    visuals.extreme_bg_color = egui::Color32::WHITE; // text/code input areas
    visuals.faint_bg_color = background;
    visuals.selection.bg_fill = accent;
    visuals.hyperlink_color = accent;
    visuals.widgets.noninteractive.bg_fill = panel;
    visuals.widgets.inactive.bg_fill = background;
    visuals.widgets.hovered.bg_fill = accent.gamma_multiply(0.35);
    visuals.widgets.active.bg_fill = accent.gamma_multiply(0.55);
    visuals
}

/// 桜: pale sakura pink with white input/editor surfaces.
fn sakura_light() -> egui::Visuals {
    light_palette(
        egui::Color32::from_rgb(255, 238, 242),
        egui::Color32::from_rgb(255, 246, 248),
        egui::Color32::from_rgb(220, 112, 140),
    )
}

/// 泉: pale spring-water blue with young-grass green accents.
fn izumi_light() -> egui::Visuals {
    light_palette(
        egui::Color32::from_rgb(232, 247, 250),
        egui::Color32::from_rgb(242, 251, 247),
        egui::Color32::from_rgb(112, 166, 108),
    )
}

/// 蒼: Japanese indigo/navy-led palette. Light variant keeps content surfaces readable.
fn ao_light() -> egui::Visuals {
    let mut visuals = light_palette(
        egui::Color32::from_rgb(230, 237, 245),
        egui::Color32::from_rgb(238, 243, 249),
        egui::Color32::from_rgb(32, 72, 112),
    );
    visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(20, 45, 72);
    visuals
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
        (Theme::Sakura, "theme.sakura"),
        (Theme::Izumi, "theme.izumi"),
        (Theme::Ao, "theme.ao"),
    ] {
        let selected = *current == theme;
        if ui.radio(selected, crate::localization::text(key)).clicked() && !selected {
            *current = theme;
            changed = true;
        }
    }
    changed
}
