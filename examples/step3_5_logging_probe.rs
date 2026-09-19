use workflow_ide_framework as wfide;

fn main() {
    let mut config = wfide::logging::LoggingConfig::default();
    config.directory = "logs/step3-5-probe".into();
    config.file_prefix = "wfide-step3-5-probe".into();
    config.retention_days = 7;
    config.memory_lines = 16;

    let guard = wfide::logging::init("step3-5-probe", &config)
        .expect("failed to initialize WFIDE logging");

    wfide::tracing::info!(target: "wfide::probe", "framework probe event");
    wfide::tracing::warn!(target: "consumer::sample", "consumer application probe event");

    let lines = guard.snapshot();
    let framework = lines.iter().any(|line| line.contains("framework probe event"));
    let consumer = lines.iter().any(|line| line.contains("consumer application probe event"));

    println!("WFIDE_LOGGING_PROBE framework_in_memory={framework}");
    println!("WFIDE_LOGGING_PROBE consumer_in_memory={consumer}");

    assert!(framework, "framework log was not captured in memory");
    assert!(consumer, "consumer log was not captured in memory");
}
