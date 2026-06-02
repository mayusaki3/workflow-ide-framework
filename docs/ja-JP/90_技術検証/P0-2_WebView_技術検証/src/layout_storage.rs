//! Dock レイアウト保存・読込処理。
//!
//! 役割:
//! - egui_dock の DockState を JSON として保存・読込する。
//! - 初期 Dock レイアウトを生成する。
//!
//! 注意:
//! - P0-2 WebView 技術検証用のPoCコード。
//! - 保存形式は正式仕様ではなく、検証用の暫定形式。

use std::fs;
use std::path::Path;

use egui_dock::{DockState, NodeIndex};

use crate::panel_tab::PanelTab;

const LAYOUT_FILE_PATH: &str = "dock_layout.json";

/// Dock レイアウトを保存する。
///
/// # 引数
///
/// * `dock_state` - 保存対象の DockState。
///
/// # 注意
///
/// 保存失敗時はPoC継続を優先し、エラーを握りつぶす。
pub fn save_layout(dock_state: &DockState<PanelTab>) {
    if let Ok(json) = serde_json::to_string_pretty(dock_state) {
        let _ = fs::write(LAYOUT_FILE_PATH, json);
    }
}

/// Dock レイアウトを読み込む。
///
/// # 戻り値
///
/// 読込に成功した場合は `Some(DockState<PanelTab>)`。
/// ファイルが存在しない、または読込・復元に失敗した場合は `None`。
pub fn load_layout() -> Option<DockState<PanelTab>> {
    if !Path::new(LAYOUT_FILE_PATH).exists() {
        return None;
    }

    let json = fs::read_to_string(LAYOUT_FILE_PATH).ok()?;
    serde_json::from_str(&json).ok()
}

/// 初期 Dock レイアウトを作成する。
///
/// # 戻り値
///
/// P0-2 WebView 技術検証用の初期 DockState。
pub fn create_default_layout() -> DockState<PanelTab> {
    let mut dock_state = DockState::new(vec![PanelTab::Status]);

    let surface = dock_state.main_surface_mut();

    let [_, right] = surface.split_right(
        NodeIndex::root(),
        0.7,
        vec![PanelTab::Viewport],
    );

    let [_, bottom] = surface.split_below(
        right,
        0.8,
        vec![PanelTab::Log],
    );

    surface.split_right(
        bottom,
        0.5,
        vec![PanelTab::WebViewPlaceholder],
    );

    dock_state
}
