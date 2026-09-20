use workflow_ide_framework as wfide;

fn main() {
    let mut config = wfide::ApplicationConfig::new(
        "workflow-ide-framework-step5-5-text-editor",
        "Workflow IDE Framework Step 5.5",
    );
    config.window.title = Some("Workflow IDE Framework - Step 5.5 Text Editor".into());
    config.window.initial_size = Some([1000.0, 700.0]);
    config.window.min_size = Some([720.0, 480.0]);
    config.appearance.font_path = Some("assets/fonts/default/NotoSansCJK-Regular.ttc".into());
    config.localization.default_locale = wfide::localization::JA_JP.to_owned();

    wfide::Application::with_config(config)
        .panel(wfide::PanelDefinition::new(
            "text-editor",
            "Text Editor",
            wfide::PanelKind::StandardUi,
        ))
        .text_editor_panel(
            "text-editor",
            wfide::text_editor::TextDocument::new(
                "sample-document",
                "sample.toml",
                "# workflow-ide-framework Text Editor\n\n[application]\nname = \"サンプル\"\nenabled = true\n",
            )
            .language_hint("TOML"),
        )
        .text_editor_ime_debug("text-editor", true)
        .layout(wfide::LayoutConfig::new(["text-editor"]).selected("text-editor"))
        .language_settings_panel()
        .run()
        .expect("failed to start Step 5.5 Text Editor sample");
}
