//! WV-12-03 GPU Surface Resize / Visibility / Lifecycle Probe。
//!
//! 目的:
//! - Dock 内の利用可能サイズに合わせて off-screen GPU Texture を再生成する。
//! - GPU Surface の表示 / 非表示を切り替えられることを確認する。
//! - GPU Surface を明示的に破棄し、再生成できることを確認する。
//! - 再生成後も描画更新が継続することを確認する。
//!
//! 本ファイルは技術検証用であり、正式 Surface API ではない。

use eframe::egui;
use egui_dock::{DockArea, DockState, TabViewer};
use std::time::Instant;

const MIN_TEXTURE_SIZE: u32 = 1;

#[derive(Debug)]
enum ProbeTab { GpuSurface, Info }

struct GpuResources {
    texture: wgpu::Texture,
    texture_id: egui::TextureId,
    width: u32,
    height: u32,
    generation: u64,
    instance: u64,
}

struct ProbeApp {
    dock_state: DockState<ProbeTab>,
    render_state: egui_wgpu::RenderState,
    gpu: Option<GpuResources>,
    started: Instant,
    visible: bool,
    surface_enabled: bool,
    next_instance: u64,
    requested_size: [u32; 2],
}

impl ProbeApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let render_state = cc.wgpu_render_state.as_ref()
            .expect("WV-12-03 requires the eframe wgpu renderer").clone();
        println!("WV-12-03 GPU Surface lifecycle probe start");
        println!("wgpu adapter: {:?}", render_state.adapter.get_info());
        Self {
            dock_state: DockState::new(vec![ProbeTab::GpuSurface, ProbeTab::Info]),
            render_state, gpu: None, started: Instant::now(), visible: true,
            surface_enabled: true, next_instance: 1, requested_size: [800, 600],
        }
    }

    fn create_gpu(&mut self, width: u32, height: u32) {
        let width = width.max(MIN_TEXTURE_SIZE);
        let height = height.max(MIN_TEXTURE_SIZE);
        let instance = self.next_instance;
        self.next_instance = self.next_instance.saturating_add(1);
        let texture = self.render_state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("WV-12-03 GPU Surface off-screen texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_id = self.render_state.renderer.write().register_native_texture(
            &self.render_state.device, &view, wgpu::FilterMode::Linear);
        self.gpu = Some(GpuResources { texture, texture_id, width, height, generation: 0, instance });
        println!("GPU Surface create: instance={instance}, size={width}x{height}");
    }

    fn destroy_gpu(&mut self) {
        if let Some(gpu) = self.gpu.take() {
            self.render_state.renderer.write().free_texture(&gpu.texture_id);
            println!("GPU Surface destroy: instance={}, final_generation={}, size={}x{}",
                gpu.instance, gpu.generation, gpu.width, gpu.height);
        }
    }

    fn ensure_gpu_size(&mut self) {
        if !self.surface_enabled || !self.visible { return; }
        let [width, height] = self.requested_size;
        let recreate = match &self.gpu {
            Some(gpu) => gpu.width != width || gpu.height != height,
            None => true,
        };
        if recreate { self.destroy_gpu(); self.create_gpu(width, height); }
    }

    fn render_gpu_frame(&mut self) {
        if !self.surface_enabled || !self.visible { return; }
        let Some(gpu) = self.gpu.as_mut() else { return; };
        let t = self.started.elapsed().as_secs_f64();
        let (r, g, b) = (0.25 + 0.20*t.sin(), 0.30 + 0.20*(t*0.73).sin(), 0.55 + 0.20*(t*1.21).sin());
        let view = gpu.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.render_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("WV-12-03 GPU Surface encoder") });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("WV-12-03 GPU Surface render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view, resolve_target: None, depth_slice: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a: 1.0 }), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
            });
        }
        self.render_state.queue.submit(Some(encoder.finish()));
        gpu.generation = gpu.generation.saturating_add(1);
        if gpu.generation == 1 || gpu.generation % 120 == 0 {
            println!("GPU Surface frame: instance={}, generation={}, size={}x{}", gpu.instance, gpu.generation, gpu.width, gpu.height);
        }
    }
}

impl Drop for ProbeApp {
    fn drop(&mut self) { self.destroy_gpu(); println!("WV-12-03 GPU Surface lifecycle probe shutdown"); }
}

struct ProbeTabViewer<'a> {
    texture_id: Option<egui::TextureId>, texture_size: Option<[u32; 2]>, generation: u64, instance: u64,
    visible: &'a mut bool, surface_enabled: bool, requested_size: &'a mut [u32; 2],
    destroy_requested: &'a mut bool, create_requested: &'a mut bool,
}

impl TabViewer for ProbeTabViewer<'_> {
    type Tab = ProbeTab;
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab { ProbeTab::GpuSurface => "GPU Surface".into(), ProbeTab::Info => "Lifecycle Control".into() }
    }
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            ProbeTab::GpuSurface => {
                let available = ui.available_size();
                self.requested_size[0] = available.x.round().max(1.0) as u32;
                self.requested_size[1] = available.y.round().max(1.0) as u32;
                if !self.surface_enabled {
                    ui.centered_and_justified(|ui| { ui.label("GPU Surface destroyed"); });
                } else if !*self.visible {
                    ui.centered_and_justified(|ui| { ui.label("GPU Surface hidden"); });
                } else if let Some(texture_id) = self.texture_id {
                    ui.image((texture_id, available));
                } else {
                    ui.centered_and_justified(|ui| { ui.label("GPU Surface not created"); });
                }
            }
            ProbeTab::Info => {
                ui.heading("WV-12-03 Resize / Visibility / Lifecycle");
                ui.label(format!("Lifecycle: {}", if self.surface_enabled { "created" } else { "destroyed" }));
                ui.label(format!("Visible: {}", *self.visible));
                ui.label(format!("Surface instance: {}", self.instance));
                ui.label(format!("Frame generation: {}", self.generation));
                if let Some([w,h]) = self.texture_size { ui.label(format!("Texture size: {w}x{h}")); } else { ui.label("Texture size: none"); }
                ui.label(format!("Requested Dock size: {}x{}", self.requested_size[0], self.requested_size[1]));
                ui.separator();
                if ui.button(if *self.visible { "Hide GPU Surface" } else { "Show GPU Surface" }).clicked() { *self.visible = !*self.visible; }
                if ui.add_enabled(self.surface_enabled, egui::Button::new("Destroy GPU Surface")).clicked() { *self.destroy_requested = true; }
                if ui.button(if self.surface_enabled { "Recreate GPU Surface" } else { "Create GPU Surface" }).clicked() { *self.create_requested = true; }
                ui.separator();
                ui.label("確認:");
                ui.label("1. Window リサイズ後、Texture size が Dock サイズへ追従すること");
                ui.label("2. Hide / Show で表示状態を切り替えられること");
                ui.label("3. Destroy 後 Lifecycle=destroyed / Texture size=none となり、Surface が再生成されないこと");
                ui.label("4. Create で instance が増え、generation が 1 から再開すること");
                ui.label("5. 再生成後も色の継続変化と generation 増加が続くこと");
            }
        }
    }
}

impl eframe::App for ProbeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut destroy_requested = false;
        let mut create_requested = false;
        let texture_id = self.gpu.as_ref().map(|g| g.texture_id);
        let texture_size = self.gpu.as_ref().map(|g| [g.width,g.height]);
        let generation = self.gpu.as_ref().map_or(0, |g| g.generation);
        let instance = self.gpu.as_ref().map_or(0, |g| g.instance);
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = ProbeTabViewer {
                texture_id, texture_size, generation, instance, visible: &mut self.visible,
                surface_enabled: self.surface_enabled, requested_size: &mut self.requested_size,
                destroy_requested: &mut destroy_requested, create_requested: &mut create_requested,
            };
            DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);
        });
        if destroy_requested {
            self.surface_enabled = false;
            self.destroy_gpu();
        }
        if create_requested {
            self.destroy_gpu();
            self.surface_enabled = true;
            let [w,h] = self.requested_size;
            self.create_gpu(w,h);
        }
        self.ensure_gpu_size();
        self.render_gpu_frame();
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default().with_title("WV-12-03 GPU Surface Lifecycle Probe").with_inner_size([1000.0,720.0]),
        ..Default::default()
    };
    eframe::run_native("WV-12-03 GPU Surface Lifecycle Probe", options,
        Box::new(|cc| Ok(Box::new(ProbeApp::new(cc)))))
}
