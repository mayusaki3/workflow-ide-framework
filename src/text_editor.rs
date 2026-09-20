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
    pub cursor_line: Option<usize>,
    pub cursor_column: Option<usize>,
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
            cursor_line: None,
            cursor_column: None,
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

    // Reserve the status bar at the bottom first so it can never escape the
    // Dock panel. The editor then consumes only the remaining central area.
    egui::TopBottomPanel::bottom(ui.id().with("text_editor_status"))
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let cursor = match (document.cursor_line, document.cursor_column) {
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
        });

    egui::TopBottomPanel::top(ui.id().with("text_editor_toolbar"))
        .resizable(false)
        .show_inside(ui, |ui| {
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
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show_inside(ui, |ui| {
            let available = ui.available_size();
            let output = egui::TextEdit::multiline(&mut document.text)
                .desired_width(available.x)
                .desired_rows(1)
                .min_size(available)
                .interactive(!document.read_only)
                .code_editor()
                .show(ui);

            if output.response.changed() {
                document.modified = true;
                result.changed = true;
            }

            if let Some(range) = output.cursor_range {
                let (line, column) = line_column(&document.text, range.primary.index);
                document.cursor_line = Some(line);
                document.cursor_column = Some(column);
                result.cursor_line = Some(line);
                result.cursor_column = Some(column);
            }
        });

    result
}
