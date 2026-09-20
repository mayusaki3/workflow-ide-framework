use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEncoding {
    Utf8,
    ShiftJis,
}

impl TextEncoding {
    pub fn label(self) -> &'static str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::ShiftJis => "Shift_JIS",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextDocument {
    pub id: String,
    pub display_name: String,
    pub text: String,
    pub read_only: bool,
    pub modified: bool,
    pub language_hint: Option<String>,
    pub encoding: TextEncoding,
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
            encoding: TextEncoding::Utf8,
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

    pub fn encoding(mut self, encoding: TextEncoding) -> Self {
        self.encoding = encoding;
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
    pub cursor_line: Option<usize>,
    pub cursor_column: Option<usize>,
}

fn line_column(text: &str, char_index: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut column = 1usize;
    for ch in text.chars().take(char_index) {
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

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
            ui.label(crate::localization::text("text_editor.read_only"));
        }
        if ui
            .add_enabled(
                !document.read_only && document.modified,
                egui::Button::new(crate::localization::text("text_editor.save")),
            )
            .clicked()
        {
            result.action = Some(TextEditorAction::SaveRequested);
        }
    });
    ui.separator();

    let status_height = ui.spacing().interact_size.y + ui.spacing().item_spacing.y;
    let editor_height = (ui.available_height() - status_height).max(80.0);
    let output = egui::TextEdit::multiline(&mut document.text)
        .desired_width(f32::INFINITY)
        .desired_rows(1)
        .interactive(!document.read_only)
        .code_editor()
        .show(ui);

    // Fill the remaining panel height instead of using a fixed row count.
    let current_height = output.response.rect.height();
    if current_height < editor_height {
        ui.allocate_space(egui::vec2(0.0, editor_height - current_height));
    }

    if output.response.changed() {
        document.modified = true;
        result.changed = true;
    }

    if let Some(range) = output.cursor_range {
        let (line, column) = line_column(&document.text, range.primary.index);
        result.cursor_line = Some(line);
        result.cursor_column = Some(column);
    }

    ui.separator();
    ui.horizontal(|ui| {
        let cursor = match (result.cursor_line, result.cursor_column) {
            (Some(line), Some(column)) => format!(
                "{} {}, {} {}",
                crate::localization::text("text_editor.line"),
                line,
                crate::localization::text("text_editor.column"),
                column
            ),
            _ => crate::localization::text("text_editor.no_cursor"),
        };
        ui.label(cursor);
        ui.separator();
        ui.label(document.encoding.label());
        if let Some(language) = &document.language_hint {
            ui.separator();
            ui.label(language);
        }
    });

    result
}
