use workflow_ide_framework::{
    Application, ApplicationConfig, PanelDefinition, PanelKind,
};

fn main() {
    let mut config = ApplicationConfig::new(
        "workflow-ide-framework-step3-sample",
        "Workflow IDE Framework Step 3",
    );
    config.window.title = Some("Workflow IDE Framework - Step 3 Sample".into());
    config.window.initial_size = Some([960.0, 640.0]);
    config.window.min_size = Some([640.0, 480.0]);

    Application::with_config(config)
        .panel(PanelDefinition::new(
            "properties",
            "Properties",
            PanelKind::StandardUi,
        ))
        .panel(PanelDefinition::new(
            "viewport",
            "Viewport",
            PanelKind::GpuViewport,
        ))
        .panel(PanelDefinition::new(
            "help",
            "Help",
            PanelKind::Browser,
        ).initially_visible(false))
        .run()
        .expect("failed to start workflow IDE framework Step 3 sample");
}
