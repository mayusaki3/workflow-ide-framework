use workflow_ide_framework::{Application, ApplicationConfig};

fn main() {
    let mut config = ApplicationConfig::new(
        "workflow-ide-framework-step2-sample",
        "Workflow IDE Framework Step 2",
    );

    config.window.title = Some("Workflow IDE Framework - Step 2 Sample".into());
    config.window.initial_size = Some([960.0, 640.0]);
    config.window.min_size = Some([640.0, 480.0]);
    config.appearance.ui_scale = Some(1.0);

    Application::with_config(config)
        .run()
        .expect("failed to start workflow IDE framework Step 2 sample");
}
