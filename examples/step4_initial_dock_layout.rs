use workflow_ide_framework as wfide;

fn main() {
    let mut config = wfide::ApplicationConfig::new(
        "workflow-ide-framework-step4-sample",
        "Workflow IDE Framework Step 4",
    );
    config.window.title = Some("Workflow IDE Framework - Step 4 Dock Layout Sample".into());
    config.window.initial_size = Some([1100.0, 720.0]);
    config.window.min_size = Some([760.0, 520.0]);

    let layout = wfide::LayoutConfig::new(["simulation-view", "help"])
        .split_left("simulation-view", 0.72, ["runtime-control", "runtime-status"])
        .split_below("simulation-view", 0.72, ["log"])
        .selected("simulation-view");

    wfide::Application::with_config(config)
        .panel(wfide::PanelDefinition::new(
            "simulation-view",
            "Simulation View",
            wfide::PanelKind::GpuViewport,
        ))
        .panel(wfide::PanelDefinition::new(
            "runtime-control",
            "Runtime Control",
            wfide::PanelKind::StandardUi,
        ))
        .panel(wfide::PanelDefinition::new(
            "runtime-status",
            "Runtime Status",
            wfide::PanelKind::StandardUi,
        ))
        .panel(wfide::PanelDefinition::new(
            "log",
            "Log",
            wfide::PanelKind::StandardUi,
        ))
        .panel(wfide::PanelDefinition::new(
            "help",
            "Help",
            wfide::PanelKind::Browser,
        ))
        .layout(layout)
        .run()
        .expect("failed to start workflow IDE framework Step 4 sample");
}
