use crate::{logging, table::{TableColumn, TableColumnType, TableModel, TableRow, TableValue}};

pub const PANEL_ID: &str = "__wfide_probe";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeStatus {
    Pass,
    Fail,
    Unknown,
}

impl ProbeStatus {
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Pass => "○",
            Self::Fail => "✕",
            Self::Unknown => "？",
        }
    }
}

pub struct ProbeRow {
    pub area: &'static str,
    pub status: ProbeStatus,
    pub detail: String,
}

pub fn collect(window_active: bool, dock_active: bool) -> Vec<ProbeRow> {
    let logging_active = !logging::snapshot().is_empty();
    vec![
        ProbeRow { area: "Window / Host", status: if window_active { ProbeStatus::Pass } else { ProbeStatus::Fail }, detail: "Framework Host is running".into() },
        ProbeRow { area: "Dock / Layout", status: if dock_active { ProbeStatus::Pass } else { ProbeStatus::Fail }, detail: "DockState is active".into() },
        ProbeRow { area: "WFIDE Logging", status: if logging_active { ProbeStatus::Pass } else { ProbeStatus::Fail }, detail: "in-memory logging buffer receives events".into() },
        ProbeRow { area: "Browser Surface", status: ProbeStatus::Unknown, detail: "not exercised by Step 4.5 host".into() },
        ProbeRow { area: "GPU Surface", status: ProbeStatus::Unknown, detail: "not exercised by Step 4.5 host".into() },
        ProbeRow { area: "Input / IME", status: ProbeStatus::Unknown, detail: "dedicated probe integration pending".into() },
        ProbeRow { area: "Lifecycle", status: ProbeStatus::Pass, detail: "Application -> Logging -> Host startup path reached".into() },
    ]
}


pub fn table_model(window_active: bool, dock_active: bool) -> TableModel {
    let mut model = TableModel::new(vec![
        TableColumn::new("area", "Area", TableColumnType::Text),
        TableColumn::new("result", "Result", TableColumnType::Status),
        TableColumn::new("detail", "Detail", TableColumnType::Text),
    ]);

    for (index, row) in collect(window_active, dock_active).into_iter().enumerate() {
        model.rows.push(TableRow::new(
            format!("probe-{index}"),
            vec![
                TableValue::Text(row.area.to_owned()),
                TableValue::Status(row.status.symbol().to_owned()),
                TableValue::Text(row.detail),
            ],
        ));
    }

    model
}
