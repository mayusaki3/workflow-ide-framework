use workflow_ide_framework::{
    Application, LayoutConfig, PanelDefinition, PanelKind,
    controller_panel::{ControllerElement, ControllerElementKind, ControllerModel, ControllerMode},
};

fn main() -> eframe::Result<()> {
    let model = ControllerModel {
        mode: ControllerMode::Operate,
        canvas_size: [820.0, 520.0],
        selected_id: None,
        elements: vec![
            ControllerElement::new("title", "タイトル", [30.0, 25.0], [260.0, 32.0],
                ControllerElementKind::Label { text: "ロボット操作パネル".into() }),
            ControllerElement::new("separator", "区切り線", [30.0, 70.0], [500.0, 2.0],
                ControllerElementKind::Line { to: [530.0, 70.0], width: 2.0 }),
            ControllerElement::new("forward", "前進", [40.0, 105.0], [110.0, 40.0],
                ControllerElementKind::Button { text: "前進".into() }),
            ControllerElement::new("speed", "速度", [40.0, 170.0], [240.0, 36.0],
                ControllerElementKind::Slider { value: 0.5, min: 0.0, max: 1.0 }),
            ControllerElement::new("stick", "移動", [330.0, 105.0], [150.0, 150.0],
                ControllerElementKind::Joystick { value: [0.0, 0.0] }),
            ControllerElement::new("robot-image", "ロボット画像", [550.0, 100.0], [180.0, 150.0],
                ControllerElementKind::Image { source: "sample://ロボット画像".into() }),
        ],
    };

    Application::new("step5-10-controller-panel", "Step 5.10 Controller Panel / コントローラーパネル")
        .font_path("assets/fonts/default/NotoSansCJK-Regular.ttc")
        .panel(PanelDefinition::new("controller", "コントローラー / Controller", PanelKind::StandardUi))
        .layout(LayoutConfig::new(["controller"]))
        .controller_panel("controller", model)
        .run()
}
