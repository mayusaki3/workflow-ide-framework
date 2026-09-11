//! WV-11-03 CEF Browser Surface Dock 表示 Probe。
//!
//! 役割:
//! - TEX-IT-SPEC-004 の Browser Surface Dock Panel 内表示を検証する。
//! - CEF OSR Paint を RGBA8 へ変換し、egui Texture として egui_dock 内へ描画する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - Resize 同期は TEX-IT-SPEC-005 で検証する。
//! - 継続更新は TEX-IT-SPEC-006 で検証する。
//! - 入力イベント転送は WV-11-04 で検証する。
//! - CEF subprocess は eframe 初期化を避けるため専用 helper EXE へ分離する。

use cef::*;
use eframe::egui;
use egui_dock::{DockArea, DockState, TabViewer};
use std::cell::RefCell;
use std::path::PathBuf;
use std::ptr;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

const VIEW_WIDTH: i32 = 800;
const VIEW_HEIGHT: i32 = 600;
const MESSAGE_PUMP_INTERVAL: Duration = Duration::from_millis(10);

#[cfg(target_os = "windows")]
const CEF_SUBPROCESS_HELPER_NAME: &str = "cef_subprocess_probe.exe";
#[cfg(not(target_os = "windows"))]
const CEF_SUBPROCESS_HELPER_NAME: &str = "cef_subprocess_probe";

/// CEF Paint の最新フレーム。
///
/// @hldocs.ref doc-20260911-120000Z-WV13#sec_h5v2c8m7p1rs
#[derive(Debug, Default)]
struct DockProbeState {
    width: i32,
    height: i32,
    rgba: Vec<u8>,
    generation: u64,
}

#[derive(Clone)]
struct DockProbeRenderHandler {
    state: Rc<RefCell<DockProbeState>>,
}

wrap_render_handler! {
    struct DockProbeRenderHandlerBuilder {
        handler: DockProbeRenderHandler,
    }

    impl RenderHandler {
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            if let Some(rect) = rect {
                rect.width = VIEW_WIDTH;
                rect.height = VIEW_HEIGHT;
            }
        }

        /// CEF BGRA Paint を RGBA8 に正規化して最新フレームとして保持する。
        ///
        /// @hldocs.ref doc-20260911-120000Z-WV13#sec_h5v2c8m7p1rs
        fn on_paint(
            &self,
            _browser: Option<&mut Browser>,
            _type_: PaintElementType,
            _dirty_rects: Option<&[Rect]>,
            buffer: *const u8,
            width: ::std::os::raw::c_int,
            height: ::std::os::raw::c_int,
        ) {
            if buffer.is_null() || width <= 0 || height <= 0 {
                return;
            }

            let Some(byte_len) = (width as usize)
                .checked_mul(height as usize)
                .and_then(|value| value.checked_mul(4))
            else {
                return;
            };

            let source = unsafe { std::slice::from_raw_parts(buffer, byte_len) };
            let mut rgba = Vec::with_capacity(byte_len);
            for pixel in source.chunks_exact(4) {
                rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
            }

            let mut state = self.handler.state.borrow_mut();
            state.width = width;
            state.height = height;
            state.rgba = rgba;
            state.generation = state.generation.saturating_add(1);
        }
    }
}

impl DockProbeRenderHandlerBuilder {
    fn build(state: Rc<RefCell<DockProbeState>>) -> RenderHandler {
        Self::new(DockProbeRenderHandler { state })
    }
}

wrap_client! {
    struct DockProbeClientBuilder {
        render_handler: RenderHandler,
    }

    impl Client {
        fn render_handler(&self) -> Option<RenderHandler> {
            Some(self.render_handler.clone())
        }
    }
}

impl DockProbeClientBuilder {
    fn build(state: Rc<RefCell<DockProbeState>>) -> Client {
        Self::new(DockProbeRenderHandlerBuilder::build(state))
    }
}

/// 現在の実行ファイルと同じディレクトリにある CEF subprocess helper を解決する。
///
/// # 戻り値
/// - 成功時: subprocess helper の絶対パス。
/// - 失敗時: 実行ファイル位置または helper の存在確認に失敗した理由。
fn resolve_subprocess_helper_path() -> Result<PathBuf, String> {
    let current_exe = std::env::current_exe()
        .map_err(|error| format!("Failed to resolve current executable path: {error}"))?;
    let executable_dir = current_exe.parent().ok_or_else(|| {
        format!(
            "Failed to resolve executable directory: {}",
            current_exe.display()
        )
    })?;
    let helper = executable_dir.join(CEF_SUBPROCESS_HELPER_NAME);

    if helper.exists() {
        Ok(helper)
    } else {
        Err(format!(
            "CEF subprocess helper not found: {}. Build it first with `cargo build --bin cef_subprocess_probe`.",
            helper.display()
        ))
    }
}

#[derive(Clone)]
enum DockProbeTab {
    Browser,
}

struct DockProbeViewer<'a> {
    texture: Option<&'a egui::TextureHandle>,
}

impl<'a> TabViewer for DockProbeViewer<'a> {
    type Tab = DockProbeTab;

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn title(&mut self, _tab: &mut Self::Tab) -> egui::WidgetText {
        "Browser Surface".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _tab: &mut Self::Tab) {
        let available = ui.available_size();
        if let Some(texture) = self.texture {
            let texture_size = texture.size_vec2();
            let scale = (available.x / texture_size.x)
                .min(available.y / texture_size.y)
                .max(0.01);
            let display_size = texture_size * scale;
            ui.centered_and_justified(|ui| {
                ui.image((texture.id(), display_size));
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Waiting for CEF OSR Paint...");
            });
        }
    }
}

struct CefDockRuntime {
    browser: Browser,
    state: Rc<RefCell<DockProbeState>>,
}

impl CefDockRuntime {
    /// Windowless Browser を初期化する。
    ///
    /// @hldocs.ref doc-20260911-120000Z-WV13#sec_h5v2c8m7p1rs
    fn new() -> Result<Self, String> {
        let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
        let args = args::Args::new();
        let process_result = execute_process(Some(args.as_main_args()), None, ptr::null_mut());
        if process_result >= 0 {
            return Err(format!(
                "Unexpected subprocess result in browser process: {process_result}"
            ));
        }

        let subprocess_path = resolve_subprocess_helper_path()?;
        let settings = Settings {
            windowless_rendering_enabled: 1,
            no_sandbox: 1,
            browser_subprocess_path: subprocess_path.to_string_lossy().as_ref().into(),
            ..Default::default()
        };

        let initialized = initialize(
            Some(args.as_main_args()),
            Some(&settings),
            None,
            ptr::null_mut(),
        );
        if initialized != 1 {
            return Err(format!("cef_initialize returned {initialized}"));
        }

        let state = Rc::new(RefCell::new(DockProbeState::default()));
        let mut client = DockProbeClientBuilder::build(state.clone());
        let window_info = WindowInfo {
            windowless_rendering_enabled: 1,
            ..Default::default()
        };
        let browser_settings = BrowserSettings {
            windowless_frame_rate: 30,
            ..Default::default()
        };

        let browser = browser_host_create_browser_sync(
            Some(&window_info),
            Some(&mut client),
            Some(&"data:text/html,<html><body style='margin:0;background:#17324d;color:white;font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh'><div style='text-align:center'><h1>WV-11-03 Browser Surface</h1><p>CEF OSR rendered inside egui_dock</p></div></body></html>".into()),
            Some(&browser_settings),
            None,
            None,
        )
        .ok_or_else(|| "CEF windowless browser creation returned None".to_string())?;

        Ok(Self { browser, state })
    }

    fn pump(&self) {
        do_message_loop_work();
    }
}

impl Drop for CefDockRuntime {
    fn drop(&mut self) {
        if let Some(host) = self.browser.host() {
            host.close_browser(true.into());
        }
        for _ in 0..50 {
            do_message_loop_work();
            sleep(MESSAGE_PUMP_INTERVAL);
        }
        shutdown();
    }
}

struct DockProbeApp {
    dock_state: DockState<DockProbeTab>,
    runtime: CefDockRuntime,
    texture: Option<egui::TextureHandle>,
    applied_generation: u64,
}

impl DockProbeApp {
    fn new(runtime: CefDockRuntime) -> Self {
        Self {
            dock_state: DockState::new(vec![DockProbeTab::Browser]),
            runtime,
            texture: None,
            applied_generation: 0,
        }
    }

    /// 最新 Paint を egui Texture へ反映する。
    fn update_texture(&mut self, ctx: &egui::Context) {
        let snapshot = {
            let state = self.runtime.state.borrow();
            if state.generation == self.applied_generation
                || state.width <= 0
                || state.height <= 0
                || state.rgba.is_empty()
            {
                None
            } else {
                Some((
                    state.generation,
                    state.width as usize,
                    state.height as usize,
                    state.rgba.clone(),
                ))
            }
        };

        let Some((generation, width, height, rgba)) = snapshot else {
            return;
        };

        let image = egui::ColorImage::from_rgba_unmultiplied([width, height], &rgba);
        match self.texture.as_mut() {
            Some(texture) if texture.size() == [width, height] => {
                texture.set(image, egui::TextureOptions::LINEAR);
            }
            _ => {
                self.texture = Some(ctx.load_texture(
                    "wv11_03_dock_probe",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        self.applied_generation = generation;
    }
}

impl eframe::App for DockProbeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.runtime.pump();
        self.update_texture(ctx);

        egui::TopBottomPanel::top("wv11_03_status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("WV-11-03 TEX-IT-SPEC-004");
                ui.separator();
                ui.label("Browser Surface is rendered as an egui Texture; no Browser Native Window.");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = DockProbeViewer {
                texture: self.texture.as_ref(),
            };
            DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);
        });

        ctx.request_repaint_after(MESSAGE_PUMP_INTERVAL);
    }
}

fn main() -> eframe::Result<()> {
    println!("WV-11-03 CEF Dock display probe start");

    let runtime = match CefDockRuntime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("WV-11-03 CEF Dock display probe failed: {error}");
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-03 Browser Surface Dock Probe")
            .with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-03 Browser Surface Dock Probe",
        native_options,
        Box::new(move |_cc| Ok(Box::new(DockProbeApp::new(runtime)))),
    )
}
