use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone)]
pub struct FlowPort {
    pub id: String,
    pub label: String,
    pub direction: PortDirection,
    pub data_type: Option<String>,
}

impl FlowPort {
    pub fn new(id: impl Into<String>, label: impl Into<String>, direction: PortDirection) -> Self {
        Self { id: id.into(), label: label.into(), direction, data_type: None }
    }

    pub fn data_type(mut self, data_type: impl Into<String>) -> Self {
        self.data_type = Some(data_type.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct FlowNode {
    pub id: String,
    pub label: String,
    pub position: egui::Pos2,
    pub ports: Vec<FlowPort>,
}

impl FlowNode {
    pub fn new(id: impl Into<String>, label: impl Into<String>, position: [f32; 2]) -> Self {
        Self { id: id.into(), label: label.into(), position: position.into(), ports: Vec::new() }
    }

    pub fn port(mut self, port: FlowPort) -> Self {
        self.ports.push(port);
        self
    }
}

#[derive(Debug, Clone)]
pub struct FlowEdge {
    pub id: String,
    pub from_node: String,
    pub from_port: String,
    pub to_node: String,
    pub to_port: String,
}

impl FlowEdge {
    pub fn new(
        id: impl Into<String>,
        from_node: impl Into<String>,
        from_port: impl Into<String>,
        to_node: impl Into<String>,
        to_port: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            from_node: from_node.into(),
            from_port: from_port.into(),
            to_node: to_node.into(),
            to_port: to_port.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FlowModel {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
    pub selected_node_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlowAction {
    NodeSelected { node_id: String },
    NodeMoved { node_id: String, position: [f32; 2] },
}

#[derive(Debug, Default)]
pub struct FlowResponse {
    pub actions: Vec<FlowAction>,
}

pub fn show(ui: &mut egui::Ui, model: &mut FlowModel) -> FlowResponse {
    let mut response = FlowResponse::default();
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter_at(rect);
    let node_size = egui::vec2(170.0, 84.0);

    // Edges are drawn first so nodes remain visually on top.
    for edge in &model.edges {
        let from = model.nodes.iter().find(|node| node.id == edge.from_node);
        let to = model.nodes.iter().find(|node| node.id == edge.to_node);
        if let (Some(from), Some(to)) = (from, to) {
            let a = rect.min + from.position.to_vec2() + egui::vec2(node_size.x, node_size.y * 0.5);
            let b = rect.min + to.position.to_vec2() + egui::vec2(0.0, node_size.y * 0.5);
            painter.line_segment([a, b], ui.visuals().widgets.inactive.fg_stroke);
        }
    }

    for node in &mut model.nodes {
        let node_rect = egui::Rect::from_min_size(rect.min + node.position.to_vec2(), node_size);
        let id = ui.make_persistent_id(("wfide_flow_node", &node.id));
        let hit = ui.interact(node_rect, id, egui::Sense::click_and_drag());

        if hit.clicked() {
            model.selected_node_id = Some(node.id.clone());
            response.actions.push(FlowAction::NodeSelected { node_id: node.id.clone() });
        }
        if hit.dragged() {
            // drag_delta() is the total displacement since drag start.
            // Applying it every frame accumulates the same displacement and can
            // move a node explosively far outside the canvas. Use the per-frame
            // pointer delta instead.
            let delta = ui.input(|input| input.pointer.delta());
            node.position += delta;
            response.actions.push(FlowAction::NodeMoved {
                node_id: node.id.clone(),
                position: [node.position.x, node.position.y],
            });
        }

        let selected = model.selected_node_id.as_deref() == Some(node.id.as_str());
        let visuals = if selected { &ui.visuals().widgets.active } else { &ui.visuals().widgets.inactive };
        painter.rect(node_rect, 6.0, visuals.bg_fill, visuals.bg_stroke, egui::StrokeKind::Inside);
        painter.text(
            node_rect.left_top() + egui::vec2(8.0, 8.0),
            egui::Align2::LEFT_TOP,
            &node.label,
            egui::TextStyle::Button.resolve(ui.style()),
            visuals.fg_stroke.color,
        );

        let mut input_y = node_rect.top() + 34.0;
        let mut output_y = node_rect.top() + 34.0;
        for port in &node.ports {
            match port.direction {
                PortDirection::Input => {
                    let p = egui::pos2(node_rect.left(), input_y);
                    painter.circle_filled(p, 4.0, visuals.fg_stroke.color);
                    painter.text(p + egui::vec2(8.0, -7.0), egui::Align2::LEFT_TOP, &port.label,
                        egui::TextStyle::Small.resolve(ui.style()), visuals.fg_stroke.color);
                    input_y += 18.0;
                }
                PortDirection::Output => {
                    let p = egui::pos2(node_rect.right(), output_y);
                    painter.circle_filled(p, 4.0, visuals.fg_stroke.color);
                    painter.text(p + egui::vec2(-8.0, -7.0), egui::Align2::RIGHT_TOP, &port.label,
                        egui::TextStyle::Small.resolve(ui.style()), visuals.fg_stroke.color);
                    output_y += 18.0;
                }
            }
        }
    }

    ui.allocate_rect(rect, egui::Sense::hover());
    response
}
