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
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.split(anchor, SplitDirection::Left, fraction, panels)
    }

    pub fn split_right<I, S>(self, anchor: impl Into<String>, fraction: f32, panels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.split(anchor, SplitDirection::Right, fraction, panels)
    }

    pub fn split_above<I, S>(self, anchor: impl Into<String>, fraction: f32, panels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.split(anchor, SplitDirection::Above, fraction, panels)
    }

    pub fn split_below<I, S>(self, anchor: impl Into<String>, fraction: f32, panels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.split(anchor, SplitDirection::Below, fraction, panels)
    }

    pub fn selected(mut self, panel_id: impl Into<String>) -> Self {
        self.selected_panel_id = Some(panel_id.into());
        self
    }
}
