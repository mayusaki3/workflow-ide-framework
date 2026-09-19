fn main() {
    workflow_ide_framework::Application::new(
        "workflow-ide-framework-step1-sample",
        "Workflow IDE Framework - Step 1 Sample",
    )
    .run()
    .expect("failed to start workflow IDE framework sample");
}
