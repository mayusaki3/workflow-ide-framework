use workflow_ide_framework as wfide;

fn main() {
    let mut config = wfide::ApplicationConfig::new(
        "workflow-ide-framework-step3-5-sample",
        "Workflow IDE Framework Step 3.5",
    );
    config.window.title = Some("Workflow IDE Framework - Step 3.5 Logging Sample".into());
    config.window.initial_size = Some([960.0, 640.0]);
    config.window.min_size = Some([640.0, 480.0]);
    config.logging.directory = "logs/step3-5".into();
    config.logging.file_prefix = "wfide-step3-5".into();
    config.logging.retention_days = 7;

    wfide::Application::with_config(config)
        .run()
        .expect("failed to start workflow IDE framework Step 3.5 sample");
}
