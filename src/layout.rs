use std::{collections::HashSet, error::Error, fmt};

use egui_dock::DockState;

use crate::PanelDefinition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Left,
    Right,
    Above,
    Below,
}

#[derive(Debug, Clone)]
pub struct LayoutSplit {
    pub anchor_panel_id: String,
    pub direction: SplitDirection,
    pub fraction: f32,
    pub panel_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub root_panel_ids: Vec<String>,
    pub splits: Vec<LayoutSplit>,
    pub selected_panel_id: Option<String>,
}

impl LayoutConfig {
    pub fn new<I, S>(root_panel_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            root_panel_ids: root_panel_ids.into_iter().map(Into::into).collect(),
            splits: Vec::new(),
            selected_panel_id: None,
        }
    }

    pub fn split<I, S>(
        mut self,
        anchor_panel_id: impl Into<String>,
        direction: SplitDirection,
        fraction: f32,
        panel_ids: I,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.splits.push(LayoutSplit {
            anchor_panel_id: anchor_panel_id.into(),
            direction,
            fraction,
            panel_ids: panel_ids.into_iter().map(Into::into).collect(),
        });
        self
    }

    pub fn split_left<I, S>(self, anchor: impl Into<String>, fraction: f32, panels: I) -> Self
    where I: IntoIterator<Item = S>, S: Into<String> {
        self.split(anchor, SplitDirection::Left, fraction, panels)
    }

    pub fn split_right<I, S>(self, anchor: impl Into<String>, fraction: f32, panels: I) -> Self
    where I: IntoIterator<Item = S>, S: Into<String> {
        self.split(anchor, SplitDirection::Right, fraction, panels)
    }

    pub fn split_above<I, S>(self, anchor: impl Into<String>, fraction: f32, panels: I) -> Self
    where I: IntoIterator<Item = S>, S: Into<String> {
        self.split(anchor, SplitDirection::Above, fraction, panels)
    }

    pub fn split_below<I, S>(self, anchor: impl Into<String>, fraction: f32, panels: I) -> Self
    where I: IntoIterator<Item = S>, S: Into<String> {
        self.split(anchor, SplitDirection::Below, fraction, panels)
    }

    pub fn selected(mut self, panel_id: impl Into<String>) -> Self {
        self.selected_panel_id = Some(panel_id.into());
        self
    }
}

#[derive(Debug)]
pub struct LayoutError(String);

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for LayoutError {}

pub(crate) fn build_dock_state(
    layout: &LayoutConfig,
    panels: &[PanelDefinition],
) -> Result<DockState<String>, LayoutError> {
    if layout.root_panel_ids.is_empty() {
        return Err(LayoutError("initial layout root must contain at least one panel".into()));
    }

    let declared: HashSet<&str> = panels.iter().map(|panel| panel.id.as_str()).collect();
    let visible: HashSet<&str> = panels
        .iter()
        .filter(|panel| panel.initially_visible)
        .map(|panel| panel.id.as_str())
        .collect();

    let mut used = HashSet::new();
    let mut check = |id: &str| -> Result<(), LayoutError> {
        if !declared.contains(id) {
            return Err(LayoutError(format!("layout references unknown panel id '{id}'")));
        }
        if !visible.contains(id) {
            return Err(LayoutError(format!(
                "layout references initially hidden panel id '{id}'"
            )));
        }
        if !used.insert(id.to_owned()) {
            return Err(LayoutError(format!("panel id '{id}' appears more than once in layout")));
        }
        Ok(())
    };

    for id in &layout.root_panel_ids {
        check(id)?;
    }
    for split in &layout.splits {
        if !(0.0..=1.0).contains(&split.fraction) {
            return Err(LayoutError(format!(
                "split fraction for '{}' must be between 0 and 1",
                split.anchor_panel_id
            )));
        }
        if split.panel_ids.is_empty() {
            return Err(LayoutError(format!(
                "split anchored at '{}' must contain at least one panel",
                split.anchor_panel_id
            )));
        }
        for id in &split.panel_ids {
            check(id)?;
        }
    }

    let mut dock = DockState::new(layout.root_panel_ids.clone());

    for split in &layout.splits {
        let (anchor_node, _) = dock
            .main_surface()
            .find_tab(&split.anchor_panel_id)
            .ok_or_else(|| LayoutError(format!(
                "split anchor panel '{}' is not present in the current layout",
                split.anchor_panel_id
            )))?;

        let tree = dock.main_surface_mut();
        match split.direction {
            SplitDirection::Left => {
                tree.split_left(anchor_node, split.fraction, split.panel_ids.clone());
            }
            SplitDirection::Right => {
                tree.split_right(anchor_node, split.fraction, split.panel_ids.clone());
            }
            SplitDirection::Above => {
                tree.split_above(anchor_node, split.fraction, split.panel_ids.clone());
            }
            SplitDirection::Below => {
                tree.split_below(anchor_node, split.fraction, split.panel_ids.clone());
            }
        }
    }

    if let Some(selected) = &layout.selected_panel_id {
        let (node, tab) = dock
            .main_surface()
            .find_tab(selected)
            .ok_or_else(|| LayoutError(format!(
                "selected panel id '{selected}' is not present in the layout"
            )))?;
        dock.main_surface_mut()
            .set_active_tab(node, tab)
            .map_err(|error| LayoutError(format!("failed to select panel '{selected}': {error}")))?;
    }

    Ok(dock)
}
