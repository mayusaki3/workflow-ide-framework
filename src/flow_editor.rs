use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection { Input, Output }

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
        self.data_type = Some(data_type.into()); self
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
    pub fn port(mut self, port: FlowPort) -> Self { self.ports.push(port); self }
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
    pub fn new(id: impl Into<String>, from_node: impl Into<String>, from_port: impl Into<String>,
        to_node: impl Into<String>, to_port: impl Into<String>) -> Self {
        Self { id: id.into(), from_node: from_node.into(), from_port: from_port.into(),
            to_node: to_node.into(), to_port: to_port.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortRef { pub node_id: String, pub port_id: String }

#[derive(Debug, Clone)]
pub struct FlowModel {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
    pub selected_node_id: Option<String>,
    pub selected_edge_id: Option<String>,
    pub pending_connection: Option<PortRef>,
    /// Canvas pan in screen points.
    pub pan: egui::Vec2,
    /// Canvas zoom. 1.0 = 100%.
    pub zoom: f32,
}

impl Default for FlowModel {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            selected_node_id: None,
            selected_edge_id: None,
            pending_connection: None,
            pan: egui::Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlowAction {
    NodeSelected { node_id: String },
    EdgeSelected { edge_id: String },
    SelectionCleared,
    NodeMoved { node_id: String, position: [f32; 2] },
    ConnectionCreated { edge_id: String, from: PortRef, to: PortRef },
    ConnectionRejected { from: PortRef, to: PortRef, reason: String },
    EdgeDeleted { edge_id: String },
}
#[derive(Debug, Default)]
pub struct FlowResponse { pub actions: Vec<FlowAction> }

pub type ConnectionValidator = dyn Fn(&FlowModel, &PortRef, &PortRef) -> Result<(), String>;

fn next_edge_id(model: &FlowModel) -> String {
    let mut index = model.edges.len() + 1;
    loop {
        let id = format!("edge-{index}");
        if !model.edges.iter().any(|edge| edge.id == id) { return id; }
        index += 1;
    }
}

fn port_direction(model: &FlowModel, port: &PortRef) -> Option<PortDirection> {
    model.nodes.iter().find(|node| node.id == port.node_id)
        .and_then(|node| node.ports.iter().find(|candidate| candidate.id == port.port_id))
        .map(|port| port.direction)
}

fn port_position(rect: egui::Rect, node: &FlowNode, port_id: &str, node_size: egui::Vec2, pan: egui::Vec2, zoom: f32) -> Option<egui::Pos2> {
    let node_rect = egui::Rect::from_min_size(
        rect.min + pan + node.position.to_vec2() * zoom,
        node_size * zoom,
    );
    let mut input_y = node_rect.top() + 34.0 * zoom;
    let mut output_y = node_rect.top() + 34.0 * zoom;
    for port in &node.ports {
        let p = match port.direction {
            PortDirection::Input => { let p=egui::pos2(node_rect.left(), input_y); input_y+=18.0 * zoom; p }
            PortDirection::Output => { let p=egui::pos2(node_rect.right(), output_y); output_y+=18.0 * zoom; p }
        };
        if port.id == port_id { return Some(p); }
    }
    None
}

fn distance_to_segment(point: egui::Pos2, a: egui::Pos2, b: egui::Pos2) -> f32 {
    let ab = b - a;
    let length_sq = ab.length_sq();
    if length_sq <= f32::EPSILON { return point.distance(a); }
    let t = ((point - a).dot(ab) / length_sq).clamp(0.0, 1.0);
    point.distance(a + ab * t)
}

pub fn show(ui: &mut egui::Ui, model: &mut FlowModel) -> FlowResponse {
    show_with_validator(ui, model, None)
}

pub fn show_with_validator(ui: &mut egui::Ui, model: &mut FlowModel, validator: Option<&ConnectionValidator>) -> FlowResponse {
    egui::ScrollArea::both()
        .id_salt("wfide_flow_canvas_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| show_canvas(ui, model, validator))
        .inner
}

fn show_canvas(ui: &mut egui::Ui, model: &mut FlowModel, validator: Option<&ConnectionValidator>) -> FlowResponse {
    let mut response = FlowResponse::default();
    let viewport = ui.available_rect_before_wrap();
    let node_size = egui::vec2(170.0, 84.0);
    let content_max = model.nodes.iter().fold(egui::vec2(viewport.width(), viewport.height()), |max, node| {
        egui::vec2(max.x.max(node.position.x + node_size.x + 40.0), max.y.max(node.position.y + node_size.y + 40.0))
    });
    let canvas_size = egui::vec2(content_max.x.max(viewport.width()), content_max.y.max(viewport.height()));
    let (rect, canvas_hit) = ui.allocate_exact_size(canvas_size, egui::Sense::click_and_drag());
    model.zoom = model.zoom.clamp(0.25, 4.0);

    // Background drag pans the graph. Node/port interactions take precedence.
    if canvas_hit.dragged() {
        model.pan += ui.input(|input| input.pointer.delta());
    }

    // Ctrl+wheel zooms around the pointer. Plain wheel remains available to
    // the surrounding ScrollArea.
    let zoom_delta = ui.input(|input| {
        if input.modifiers.ctrl { input.raw_scroll_delta.y } else { 0.0 }
    });
    if zoom_delta != 0.0 {
        let old_zoom = model.zoom;
        let factor = (zoom_delta * 0.002).exp();
        model.zoom = (model.zoom * factor).clamp(0.25, 4.0);
        if let Some(pointer) = ui.input(|input| input.pointer.hover_pos()) {
            let local = pointer - rect.min;
            let world = (local - model.pan) / old_zoom;
            model.pan = local - world * model.zoom;
        }
    }
    let painter = ui.painter_at(rect);

    let mut delete_edge: Option<String> = None;
    for edge in &model.edges {
        let from = model.nodes.iter().find(|n| n.id == edge.from_node)
            .and_then(|n| port_position(rect, n, &edge.from_port, node_size, model.pan, model.zoom));
        let to = model.nodes.iter().find(|n| n.id == edge.to_node)
            .and_then(|n| port_position(rect, n, &edge.to_port, node_size, model.pan, model.zoom));
        if let (Some(a), Some(b)) = (from, to) {
            let selected = model.selected_edge_id.as_deref() == Some(edge.id.as_str());
            let stroke = if selected {
                egui::Stroke::new(3.0, ui.visuals().selection.stroke.color)
            } else {
                ui.visuals().widgets.inactive.fg_stroke
            };
            painter.line_segment([a,b], stroke);
            let edge_rect = egui::Rect::from_two_pos(a, b).expand(8.0);
            let edge_hit = ui.interact(
                edge_rect,
                ui.make_persistent_id(("wfide_edge", &edge.id)),
                egui::Sense::click(),
            );
            let near_edge = edge_hit.hover_pos()
                .map(|pointer| distance_to_segment(pointer, a, b) <= 8.0)
                .unwrap_or(false);
            if near_edge {
                if edge_hit.clicked() {
                    model.selected_node_id = None;
                    model.selected_edge_id = Some(edge.id.clone());
                    response.actions.push(FlowAction::EdgeSelected { edge_id: edge.id.clone() });
                }
                if edge_hit.double_clicked() {
                    delete_edge = Some(edge.id.clone());
                }
            }
        }
    }

    if let Some(edge_id) = delete_edge {
        model.edges.retain(|edge| edge.id != edge_id);
        response.actions.push(FlowAction::EdgeDeleted { edge_id });
    }

    if let (Some(start), Some(pointer)) = (&model.pending_connection, ui.input(|i| i.pointer.hover_pos())) {
        if let Some(node) = model.nodes.iter().find(|n| n.id == start.node_id) {
            if let Some(a) = port_position(rect, node, &start.port_id, node_size, model.pan, model.zoom) {
                painter.line_segment([a,pointer], ui.visuals().widgets.hovered.fg_stroke);
            }
        }
    }

    let mut clicked_port: Option<PortRef> = None;
    for node in &mut model.nodes {
        let node_rect=egui::Rect::from_min_size(
            rect.min + model.pan + node.position.to_vec2() * model.zoom,
            node_size * model.zoom,
        );
        let hit=ui.interact(node_rect,ui.make_persistent_id(("wfide_flow_node",&node.id)),egui::Sense::click_and_drag());
        if hit.clicked() {
            model.selected_node_id=Some(node.id.clone());
            model.selected_edge_id=None;
            response.actions.push(FlowAction::NodeSelected{node_id:node.id.clone()});
        }
        if hit.dragged() {
            node.position += ui.input(|i| i.pointer.delta()) / model.zoom;
            response.actions.push(FlowAction::NodeMoved{node_id:node.id.clone(),position:[node.position.x,node.position.y]});
        }
        let selected=model.selected_node_id.as_deref()==Some(node.id.as_str());
        let visuals=if selected {&ui.visuals().widgets.active}else{&ui.visuals().widgets.inactive};
        painter.rect(node_rect,6.0,visuals.bg_fill,visuals.bg_stroke,egui::StrokeKind::Inside);
        painter.text(node_rect.left_top()+egui::vec2(8.0,8.0),egui::Align2::LEFT_TOP,&node.label,
            egui::TextStyle::Button.resolve(ui.style()),visuals.fg_stroke.color);

        let mut iy=node_rect.top()+34.0 * model.zoom; let mut oy=node_rect.top()+34.0 * model.zoom;
        for port in &node.ports {
            let p=match port.direction {
                PortDirection::Input=>{let p=egui::pos2(node_rect.left(),iy);iy+=18.0 * model.zoom;p}
                PortDirection::Output=>{let p=egui::pos2(node_rect.right(),oy);oy+=18.0 * model.zoom;p}
            };
            let port_hit=ui.interact(egui::Rect::from_center_size(p,egui::vec2(16.0,16.0)),
                ui.make_persistent_id(("wfide_port",&node.id,&port.id)),egui::Sense::click());
            painter.circle_filled(p,if port_hit.hovered(){6.0}else{4.0},visuals.fg_stroke.color);
            let (offset,align)=match port.direction {
                PortDirection::Input=>(egui::vec2(8.0,-7.0),egui::Align2::LEFT_TOP),
                PortDirection::Output=>(egui::vec2(-8.0,-7.0),egui::Align2::RIGHT_TOP),
            };
            painter.text(p+offset,align,&port.label,egui::TextStyle::Small.resolve(ui.style()),visuals.fg_stroke.color);
            if port_hit.clicked() { clicked_port=Some(PortRef{node_id:node.id.clone(),port_id:port.id.clone()}); }
        }
    }

    let selection_changed = response.actions.iter().any(|action| matches!(
        action,
        FlowAction::NodeSelected { .. } | FlowAction::EdgeSelected { .. }
    ));
    if canvas_hit.clicked() && !selection_changed && clicked_port.is_none() {
        model.selected_node_id = None;
        model.selected_edge_id = None;
        response.actions.push(FlowAction::SelectionCleared);
    }

    if let Some(clicked)=clicked_port {
        if let Some(start)=model.pending_connection.take() {
            if start != clicked {
                let normalized = match (port_direction(model, &start), port_direction(model, &clicked)) {
                    (Some(PortDirection::Output), Some(PortDirection::Input)) => Ok((start.clone(), clicked.clone())),
                    (Some(PortDirection::Input), Some(PortDirection::Output)) => Ok((clicked.clone(), start.clone())),
                    _ => Err("connection requires one Output and one Input".to_owned()),
                };
                match normalized {
                    Ok((from, to)) => {
                        match validator.map(|v|v(model,&from,&to)).unwrap_or(Ok(())) {
                            Ok(()) => {
                                let edge_id = next_edge_id(model);
                                model.edges.push(FlowEdge::new(
                                    edge_id.clone(),
                                    from.node_id.clone(), from.port_id.clone(),
                                    to.node_id.clone(), to.port_id.clone(),
                                ));
                                response.actions.push(FlowAction::ConnectionCreated { edge_id, from, to });
                            }
                            Err(reason) => response.actions.push(FlowAction::ConnectionRejected { from, to, reason }),
                        }
                    }
                    Err(reason) => response.actions.push(FlowAction::ConnectionRejected { from: start, to: clicked, reason }),
                }
            }
        } else { model.pending_connection=Some(clicked); }
    }

    response
}
