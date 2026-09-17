//! WV-12-02 wgpu GPU Surface Dock 表示 Probe。
//!
//! 目的:
//! - eframe が保持する wgpu Device / Queue を利用して off-screen Texture を生成する。
//! - GPU の RenderPass で Texture を継続更新する。
//! - egui-wgpu に native Texture として登録し、egui_dock の Dock 内へ表示する。
//! - Native Child Window を使用しない GPU Surface 経路を検証する。
//!
//! 本ファイルは技術検証用であり、正式 Surface API ではない。

use eframe::egui;
use egui_dock::{DockArea, DockState, TabViewer};
use std::time::Instant;

const TEXTURE_WIDTH: u32 = 800;
const TEXTURE_HEIGHT: u32 = 600;

#[derive(Debug)]
enum ProbeTab {
    GpuSurface,
    Info,
}

struct GpuResources {
    _texture: wgpu::Texture,
    texture_id: egui::TextureId,
    generation: u64,
}

struct ProbeApp {
    dock_state: DockState<ProbeTab>,
    render_state: egui_wgpu::RenderState,
    gpu: GpuResources,
    started: Instant,
}

impl ProbeApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("WV-12-02 requires the eframe wgpu renderer")
            .clone();

        let texture = render_state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("WV-12-02 GPU Surface off-screen texture"),
            size: wgpu::Extent3d {
                width: TEXTURE_WIDTH,
                height: TEXTURE_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let texture_id = render_state.renderer.write().register_native_texture(
            &render_state.device,
            &view,
            wgpu::FilterMode::Linear,
        );

        let dock_state = DockState::new(vec![ProbeTab::GpuSurface, ProbeTab::Info]);

        println!(
            "WV-12-02 wgpu Dock Surface probe start: texture={}x{}",
            TEXTURE_WIDTH, TEXTURE_HEIGHT
        );
        println!("wgpu adapter: {:?}", render_state.adapter.get_info());

        Self {
            dock_state,
            render_state,
            gpu: GpuResources {
                _texture: texture,
                texture_id,
                generation: 0,
            },
            started: Instant::now(),
        }
    }

    fn render_gpu_frame(&mut self) {
        let t = self.started.elapsed().as_secs_f64();
        let r = 0.25 + 0.20 * t.sin();
        let g = 0.30 + 0.20 * (t * 0.73).sin();
        let b = 0.55 + 0.20 * (t * 1.21).sin();

        let view = self
            .gpu
            ._texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .render_state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("WV-12-02 GPU Surface encoder"),
            });

        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("WV-12-02 GPU Surface render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.render_state.queue.submit(Some(encoder.finish()));
        self.gpu.generation = self.gpu.generation.saturating_add(1);

        if self.gpu.generation == 1 || self.gpu.generation % 120 == 0 {
            println!(
                "wgpu GPU Surface frame: generation={}",
                self.gpu.generation
            );
        }
    }
}

struct ProbeTabViewer<'a> {
    texture_id: egui::TextureId,
    generation: u64,
    started: Instant,
    marker: std::marker::PhantomData<&'a ()>,
}

impl TabViewer for ProbeTabViewer<'_> {
    type Tab = ProbeTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            ProbeTab::GpuSurface => "GPU Surface".into(),
            ProbeTab::Info => "Probe Info".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            ProbeTab::GpuSurface => {
                let available = ui.available_size();
                ui.image((self.texture_id, available));
            }
            ProbeTab::Info => {
                ui.heading("WV-12-02 GPU Surface");
                ui.label("wgpu off-screen render -> native egui Texture -> egui_dock");
                ui.label("Native Child Window: not used");
                ui.label(format!("GPU frame generation: {}", self.generation));
                ui.label(format!(
                    "Elapsed: {:.1}s",
                    self.started.elapsed().as_secs_f32()
                ));
                ui.separator();
                ui.label("確認:");
                ui.label("1. GPU Surface タブに色が継続変化する矩形が表示されること");
                ui.label("2. Probe Info と GPU Surface を切り替えても更新が継続すること");
                ui.label("3. Application Window をリサイズしても Dock 内表示が維持されること");
            }
        }
    }
}

impl eframe::App for ProbeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_gpu_frame();

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = ProbeTabViewer {
                texture_id: self.gpu.texture_id,
                generation: self.gpu.generation,
                started: self.started,
                marker: std::marker::PhantomData,
            };
            DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);
        });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-12-02 wgpu GPU Surface Dock Probe")
            .with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-12-02 wgpu GPU Surface Dock Probe",
        options,
        Box::new(|cc| Ok(Box::new(ProbeApp::new(cc)))),
    )
}
