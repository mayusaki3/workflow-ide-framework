use crate::logging;

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
