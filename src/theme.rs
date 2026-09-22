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
    Yozakura,
    YoruNoIzumi,
    FukaiAo,
}

impl Theme {
    pub const fn is_dark(self) -> Option<bool> {
        match self {
            Self::System => None,
            Self::Dark | Self::Yozakura | Self::YoruNoIzumi | Self::FukaiAo => Some(true),
            Self::Light | Self::Sakura | Self::Izumi | Self::Ao => Some(false),
        }
    }

    pub fn apply(self, ctx: &egui::Context) {
        self.apply_with_system_dark(ctx, ctx.global_style().visuals.dark_mode);
    }

    pub fn apply_with_system_dark(self, ctx: &egui::Context, system_dark: bool) {
        match self {
            Self::System => {
                ctx.set_visuals(if system_dark { egui::Visuals::dark() } else { egui::Visuals::light() });
            }
            Self::Dark => ctx.set_visuals(egui::Visuals::dark()),
            Self::Light => ctx.set_visuals(egui::Visuals::light()),
            Self::Sakura => ctx.set_visuals(sakura_light()),
            Self::Izumi => ctx.set_visuals(izumi_light()),
            Self::Ao => ctx.set_visuals(ao_light()),
            Self::Yozakura => ctx.set_visuals(yozakura_dark()),
            Self::YoruNoIzumi => ctx.set_visuals(yoru_no_izumi_dark()),
            Self::FukaiAo => ctx.set_visuals(fukai_ao_dark()),
        }
    }

    pub fn effective_dark(self, system_dark: bool) -> bool {
        self.is_dark().unwrap_or(system_dark)
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


fn dark_palette(background: egui::Color32, panel: egui::Color32, accent: egui::Color32) -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = panel;
    visuals.window_fill = panel;
    visuals.extreme_bg_color = background;
    visuals.faint_bg_color = panel;
    visuals.selection.bg_fill = accent;
    visuals.hyperlink_color = accent;
    visuals.widgets.noninteractive.bg_fill = panel;
    visuals.widgets.inactive.bg_fill = background;
    visuals.widgets.hovered.bg_fill = accent.gamma_multiply(0.55);
    visuals.widgets.active.bg_fill = accent.gamma_multiply(0.75);
    visuals
}

/// 夜桜: deep night background with subdued sakura-pink accents.
fn yozakura_dark() -> egui::Visuals {
    dark_palette(
        egui::Color32::from_rgb(28, 22, 30),
        egui::Color32::from_rgb(39, 29, 40),
        egui::Color32::from_rgb(214, 105, 145),
    )
}

/// 夜の泉: dark water blue with restrained young-grass accents.
fn yoru_no_izumi_dark() -> egui::Visuals {
    dark_palette(
        egui::Color32::from_rgb(17, 32, 38),
        egui::Color32::from_rgb(22, 43, 47),
        egui::Color32::from_rgb(104, 166, 119),
    )
}

/// 深い蒼: deep indigo/navy palette.
fn fukai_ao_dark() -> egui::Visuals {
    dark_palette(
        egui::Color32::from_rgb(12, 25, 43),
        egui::Color32::from_rgb(17, 35, 58),
        egui::Color32::from_rgb(72, 124, 174),
    )
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeEditor {
    pub panel_fill: egui::Color32,
    pub input_fill: egui::Color32,
    pub accent: egui::Color32,
    pub text: egui::Color32,
}

impl ThemeEditor {
    pub fn from_context(ctx: &egui::Context) -> Self {
        let visuals = &ctx.global_style().visuals;
        Self {
            panel_fill: visuals.panel_fill,
            input_fill: visuals.extreme_bg_color,
            accent: visuals.selection.bg_fill,
            text: visuals.text_color(),
        }
    }

    pub fn apply(self, ctx: &egui::Context) {
        ctx.global_style_mut(|style| {
            let visuals = &mut style.visuals;
            visuals.panel_fill = self.panel_fill;
            visuals.window_fill = self.panel_fill;
            visuals.extreme_bg_color = self.input_fill;
            visuals.selection.bg_fill = self.accent;
            visuals.hyperlink_color = self.accent;
            visuals.widgets.noninteractive.fg_stroke.color = self.text;
            visuals.widgets.inactive.fg_stroke.color = self.text;
            visuals.widgets.hovered.fg_stroke.color = self.text;
            visuals.widgets.active.fg_stroke.color = self.text;
        });
    }
}

pub fn show_settings(
    ui: &mut egui::Ui,
    current: &mut Theme,
    editor: &mut Option<ThemeEditor>,
) -> bool {
    ui.heading(crate::localization::text("theme.title"));
    ui.label(crate::localization::text("theme.description"));
    ui.separator();

    ui.label(crate::localization::text("theme.standard_group"));
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

    ui.add_space(6.0);
    ui.label(crate::localization::text("theme.japanese_light_group"));
    for (theme, key) in [
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

    ui.add_space(6.0);
    ui.label(crate::localization::text("theme.japanese_dark_group"));
    for (theme, key) in [
        (Theme::Yozakura, "theme.yozakura"),
        (Theme::YoruNoIzumi, "theme.yoru_no_izumi"),
        (Theme::FukaiAo, "theme.fukai_ao"),
    ] {
        let selected = *current == theme;
        if ui.radio(selected, crate::localization::text(key)).clicked() && !selected {
            *current = theme;
            changed = true;
        }
    }

    ui.add_space(10.0);
    ui.separator();
    egui::CollapsingHeader::new(crate::localization::text("theme.editor"))
        .default_open(false)
        .show(ui, |ui| {
            if editor.is_none() {
                *editor = Some(ThemeEditor::from_context(ui.ctx()));
            }
            let edit = editor.as_mut().expect("theme editor initialized");
            let mut edited = false;
            ui.horizontal(|ui| {
                ui.label(crate::localization::text("theme.editor_panel"));
                edited |= ui.color_edit_button_srgba(&mut edit.panel_fill).changed();
            });
            ui.horizontal(|ui| {
                ui.label(crate::localization::text("theme.editor_input"));
                edited |= ui.color_edit_button_srgba(&mut edit.input_fill).changed();
            });
            ui.horizontal(|ui| {
                ui.label(crate::localization::text("theme.editor_accent"));
                edited |= ui.color_edit_button_srgba(&mut edit.accent).changed();
            });
            ui.horizontal(|ui| {
                ui.label(crate::localization::text("theme.editor_text"));
                edited |= ui.color_edit_button_srgba(&mut edit.text).changed();
            });
            if edited {
                edit.apply(ui.ctx());
            }
            if ui.button(crate::localization::text("theme.editor_reset")).clicked() {
                current.apply(ui.ctx());
                *edit = ThemeEditor::from_context(ui.ctx());
            }
        });
    changed
}
