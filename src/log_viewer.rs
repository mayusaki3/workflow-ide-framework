use eframe::egui;

#[derive(Debug, Clone)]
pub struct LogViewerOptions {
    pub auto_scroll: bool,
    pub show_toolbar: bool,
}

impl Default for LogViewerOptions {
    fn default() -> Self {
        Self { auto_scroll: true, show_toolbar: true }
    }
}

/// Framework standard Log Viewer.
///
/// The viewer only consumes the in-memory logging snapshot. It does not own
/// logging initialization, filtering, file rotation, or retention.
pub fn show(ui: &mut egui::Ui, options: &mut LogViewerOptions) {
    if options.show_toolbar {
        ui.horizontal(|ui| {
            ui.checkbox(
                &mut options.auto_scroll,
                crate::localization::text("log_viewer.auto_scroll"),
            );
            ui.label(format!(
                "{}: {:?}",
                crate::localization::text("log_viewer.level"),
                crate::logging::level()
            ));
        });
        ui.separator();
    }

    let lines = crate::logging::snapshot();
    egui::ScrollArea::both()
        .stick_to_bottom(options.auto_scroll)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            if lines.is_empty() {
                ui.label(crate::localization::text("log_viewer.empty"));
            } else {
                for line in lines {
                    ui.monospace(line);
                }
            }
        });
}
