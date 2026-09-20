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

#[derive(Debug)]
pub struct TextEditorOptions {
    pub word_wrap: bool,
}

impl Default for TextEditorOptions {
    fn default() -> Self {
        Self { word_wrap: false }
    }
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
    let mut options = TextEditorOptions::default();
    show_with_options(ui, document, &mut options)
}

pub fn show_with_options(
    ui: &mut egui::Ui,
    document: &mut TextDocument,
    options: &mut TextEditorOptions,
) -> TextEditorResponse {
    let mut result = TextEditorResponse::default();

    let full = ui.available_rect_before_wrap();
    let mut root = ui.new_child(
        egui::UiBuilder::new()
            .id_salt("text_editor_root")
            .max_rect(full)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    egui::Panel::top(root.id().with("toolbar"))
        .resizable(false)
        .show_inside(&mut root, |ui| {
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
                ui.separator();
                ui.checkbox(
                    &mut options.word_wrap,
                    crate::localization::text("text_editor.word_wrap"),
                );
            });
        });

    egui::Panel::bottom(root.id().with("status"))
        .resizable(false)
        .show_inside(&mut root, |ui| {
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

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show_inside(&mut root, |ui| {
            egui::ScrollArea::both()
                .id_salt("text_editor_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let viewport = ui.available_size();
                    let word_wrap = options.word_wrap;
                    let mut layouter =
                        move |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                            let max_width = if word_wrap {
                                wrap_width
                            } else {
                                f32::INFINITY
                            };
                            let mut job = egui::text::LayoutJob::simple(
                                text.as_str().to_owned(),
                                egui::TextStyle::Monospace.resolve(ui.style()),
                                ui.visuals().text_color(),
                                max_width,
                            );
                            job.wrap.max_width = max_width;
                            ui.fonts_mut(|fonts| fonts.layout_job(job))
                        };

                    let output = egui::TextEdit::multiline(&mut document.text)
                        .desired_width(if word_wrap { viewport.x } else { f32::INFINITY })
                        .desired_rows(1)
                        .min_size(viewport)
                        .interactive(!document.read_only)
                        .code_editor()
                        .layouter(&mut layouter)
                        .show(ui);

                    // egui emits an IME target for the focused TextEdit. Keep its
                    // candidate/composition target on the actual primary caret,
                    // including the ScrollArea offset, instead of allowing the
                    // platform integration to fall back to the editor/window edge.
                    if output.response.has_focus() {
                        if let Some(range) = output.cursor_range {
                            let caret_rect = output
                                .galley
                                .pos_from_cursor(range.primary)
                                .translate(output.galley_pos.to_vec2());
                            ui.ctx().output_mut(|platform| {
                                if let Some(ime) = &mut platform.ime {
                                    ime.cursor_rect = caret_rect;
                                }
                            });
                        }
                    }

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
        });

    ui.allocate_rect(full, egui::Sense::hover());

    result
}
