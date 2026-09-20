use eframe::egui;

#[derive(Debug, Clone)]
pub struct TextDocument {
    pub id: String,
    pub display_name: String,
    pub text: String,
    pub read_only: bool,
    pub modified: bool,
    pub language_hint: Option<String>,
}

impl TextDocument {
    pub fn new(id: impl Into<String>, display_name: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            text: text.into(),
            read_only: false,
            modified: false,
            language_hint: None,
        }
    }

    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn language_hint(mut self, language: impl Into<String>) -> Self {
        self.language_hint = Some(language.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEditorAction {
    SaveRequested,
}

#[derive(Debug, Default)]
pub struct TextEditorResponse {
    pub changed: bool,
    pub action: Option<TextEditorAction>,
}

/// Framework Standard Text Editor.
///
/// The Framework owns the editing UI and transient document editing state.
/// Loading, persistence, validation, and domain semantics belong to the
/// Consumer / Document Provider.
pub fn show(ui: &mut egui::Ui, document: &mut TextDocument) -> TextEditorResponse {
    let mut result = TextEditorResponse::default();

    ui.horizontal(|ui| {
        ui.strong(&document.display_name);
        if document.modified {
            ui.label("*");
        }
        if let Some(language) = &document.language_hint {
            ui.separator();
            ui.label(language);
        }
        if document.read_only {
            ui.separator();
            ui.label("Read only");
        }
        if ui
            .add_enabled(!document.read_only && document.modified, egui::Button::new("Save"))
            .clicked()
        {
            result.action = Some(TextEditorAction::SaveRequested);
        }
    });
    ui.separator();

    let response = ui.add(
        egui::TextEdit::multiline(&mut document.text)
            .desired_width(f32::INFINITY)
            .desired_rows(20)
            .interactive(!document.read_only)
            .code_editor(),
    );

    if response.changed() {
        document.modified = true;
        result.changed = true;
    }

    result
}
