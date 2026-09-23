use workflow_ide_framework::{Application, LayoutConfig, PanelDefinition, PanelKind};

fn main() -> eframe::Result<()> {
    Application::new("step5-6-log-viewer", "Step 5.6 Log Viewer / ログ表示")
        .font_path("assets/fonts/default/NotoSansCJK-Regular.ttc")
        .panel(PanelDefinition::new("logs", "ログ / Logs", PanelKind::StandardUi))
        .layout(LayoutConfig::new(["logs"]))
        .log_viewer_panel("logs")
        .theme_settings_panel()
        .run()
}
