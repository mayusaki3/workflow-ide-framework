use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableColumnType {
    Text,
    Integer,
    Number,
    Boolean,
    Status,
}

#[derive(Debug, Clone)]
pub struct TableColumn {
    pub id: String,
    pub title: String,
    pub column_type: TableColumnType,
}

impl TableColumn {
    pub fn new(id: impl Into<String>, title: impl Into<String>, column_type: TableColumnType) -> Self {
        Self { id: id.into(), title: title.into(), column_type }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableValue {
    Text(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
    Status(String),
    Empty,
}

impl TableValue {
    pub fn display_text(&self) -> String {
        match self {
            Self::Text(value) | Self::Status(value) => value.clone(),
            Self::Integer(value) => value.to_string(),
            Self::Number(value) => value.to_string(),
            Self::Boolean(value) => value.to_string(),
            Self::Empty => String::new(),
        }
    }
}

impl From<&str> for TableValue {
    fn from(value: &str) -> Self { Self::Text(value.to_owned()) }
}

impl From<String> for TableValue {
    fn from(value: String) -> Self { Self::Text(value) }
}

#[derive(Debug, Clone)]
pub struct TableRow {
    pub id: String,
    pub values: Vec<TableValue>,
}

impl TableRow {
    pub fn new(id: impl Into<String>, values: Vec<TableValue>) -> Self {
        Self { id: id.into(), values }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TableModel {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
}

impl TableModel {
    pub fn new(columns: Vec<TableColumn>) -> Self {
        Self { columns, rows: Vec::new() }
    }

    pub fn row(mut self, row: TableRow) -> Self {
        self.rows.push(row);
        self
    }
}

/// Framework Standard Data / Table Viewer.
///
/// v0.1.0 starts with a read-only viewer. Sorting, filtering, selection,
/// editing and virtualization are intentionally left as later extensions.
pub fn show(ui: &mut egui::Ui, id: impl std::hash::Hash, model: &TableModel) {
    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
        egui::Grid::new(id).striped(true).show(ui, |ui| {
            for column in &model.columns {
                ui.strong(&column.title);
            }
            ui.end_row();

            for row in &model.rows {
                for index in 0..model.columns.len() {
                    let text = row.values.get(index)
                        .map(TableValue::display_text)
                        .unwrap_or_default();
                    ui.label(text);
                }
                ui.end_row();
            }
        });
    });
}
