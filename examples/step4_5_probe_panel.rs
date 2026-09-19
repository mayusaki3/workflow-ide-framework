use workflow_ide_framework as wfide;

fn main() {
    let mut config = wfide::ApplicationConfig::new(
        "workflow-ide-framework-step4-5-probe",
        "Workflow IDE Framework Step 4.5",
    );
    config.window.title = Some("Workflow IDE Framework - Step 4.5 Probe Panel".into());
    config.window.initial_size = Some([1100.0, 720.0]);
    config.window.min_size = Some([760.0, 520.0]);
    config.logging.directory = "logs/step4-5".into();
    config.logging.file_prefix = "wfide-step4-5".into();
    config.appearance.font_path = Some(
        "docs/ja-JP/90_技術検証/P0-1b_EmbeddedFont_技術検証/assets/fonts/default/NotoSansCJK-Regular.ttc".into(),
    );

    let layout = wfide::LayoutConfig::new(["simulation-view"])
        .split_left("simulation-view", 0.25, ["runtime-control"])
        .split_below("runtime-control", 0.50, ["runtime-status"])
        .split_below("simulation-view", 0.75, ["log"])
        .selected("simulation-view");

    wfide::Application::with_config(config)
        .panel(wfide::PanelDefinition::new("simulation-view", "Simulation View", wfide::PanelKind::GpuViewport))
        .panel(wfide::PanelDefinition::new("runtime-control", "Runtime Control", wfide::PanelKind::StandardUi))
        .panel(wfide::PanelDefinition::new("runtime-status", "Runtime Status", wfide::PanelKind::StandardUi))
        .panel(wfide::PanelDefinition::new("log", "Log", wfide::PanelKind::StandardUi))
        .layout(layout)
        .framework_probe_panel()
        .logging_settings_panel()
        .run()
        .expect("failed to start Step 4.5 Probe Panel sample");
}
