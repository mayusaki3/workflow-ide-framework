//! Linux向け GTK Host Window 同期検証処理。
//!
//! WV-09-02
//!
//! 役割:
//! - GTK Host Window のみを生成する。
//! - WebKitGTK / wry WebView / GtkFixed 配下 WebView 操作を使用しない。
//! - eframe / winit 実行中に GTK Host Window 自体を Dock矩形へ move_() / resize() で追従させる。
//! - GTK Host Window が画面表示された状態でマウス移動により応答なしが発生するか確認する。
//!
//! 注意:
//! - 技術検証用コード。
//! - GDK_BACKEND=x11 前提。
//! - WV-07 と WV-08 の差分である「GTK Host Window 自体の Dock追従」を再導入する。

use eframe::{egui, CreationContext};
use gtk::prelude::*;
use std::time::{Duration, Instant};

static mut GTK_WINDOW: Option<gtk::Window> = None;
static mut LAST_SURFACE_STATE: Option<SurfaceState> = None;
static mut LAST_GTK_FLUSH_AT: Option<Instant> = None;

const GTK_FLUSH_MAX_ITERATIONS: usize = 64;
const GTK_FLUSH_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, PartialEq, Eq)]
struct SurfaceState {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    visible: bool,
}

pub fn initialize_root_window(_cc: &CreationContext<'_>) {
    println!("WV-09-02 gtk::init start");

    if let Err(err) = gtk::init() {
        println!("WV-09-02 gtk::init failed: {}", err);
        return;
    }

    println!("WV-09-02 gtk::init success");

    let window = gtk::Window::new(gtk::WindowType::Popup);
    window.set_title("WV-09-02 GTK Host Window");
    window.set_default_size(300, 200);

    let label = gtk::Label::new(Some("WV-09-02 GTK Host Window Only"));
    window.add(&label);

    window.show_all();

    unsafe {
        GTK_WINDOW = Some(window);
    }

    println!("WV-09-02 GTK_WINDOW stored");

    flush_gtk_events_bounded("WV-09-02 initialize");
}

pub fn ensure_webview_initialized(initial_rect: Option<egui::Rect>, scale: f32) {
    let rect = match initial_rect {
        Some(rect) => rect,
        None => {
            sync_host_window_visibility(false);
            return;
        }
    };

    let (x, y, width, height) = rect_to_i32_bounds(rect, scale);

    let state = SurfaceState {
        x,
        y,
        width,
        height,
        visible: true,
    };

    unsafe {
        if LAST_SURFACE_STATE == Some(state) {
            flush_gtk_events_throttled("WV-09-02 unchanged");
            return;
        }

        LAST_SURFACE_STATE = Some(state);

        if let Some(window) = GTK_WINDOW.as_ref() {
            println!(
                "WV-09-02 host window sync start x={} y={} w={} h={}",
                x, y, width, height
            );

            window.move_(x, y);
            window.resize(width, height);
            window.show_all();

            println!("WV-09-02 host window sync done");
        }
    }

    flush_gtk_events_throttled("WV-09-02 sync");
}

pub fn sync_child_window(
    _ctx: &egui::Context,
    _webview_rect: Option<egui::Rect>,
    should_show_native_surface: bool,
) {
    sync_host_window_visibility(should_show_native_surface);
}

fn sync_host_window_visibility(visible: bool) {
    unsafe {
        if let Some(window) = GTK_WINDOW.as_ref() {
            if visible {
                println!("WV-09-02 host window show");
                window.show_all();
            } else {
                println!("WV-09-02 host window hide");
                window.hide();
            }
        }
    }

    flush_gtk_events_throttled("WV-09-02 visibility");
}

fn rect_to_i32_bounds(rect: egui::Rect, scale: f32) -> (i32, i32, i32, i32) {
    let x = (rect.min.x * scale) as i32;
    let y = (rect.min.y * scale) as i32;
    let width = (rect.width() * scale) as i32;
    let height = (rect.height() * scale) as i32;

    (
        x.max(0).min(16_000),
        y.max(0).min(16_000),
        width.max(1).min(16_000),
        height.max(1).min(16_000),
    )
}

fn flush_gtk_events_bounded(label: &str) {
    println!("WV-09-02 GTK event flush start label={}", label);

    for iteration in 0..GTK_FLUSH_MAX_ITERATIONS {
        if !gtk::events_pending() {
            println!(
                "WV-09-02 GTK event flush completed label={} iterations={}",
                label, iteration
            );
            return;
        }

        gtk::main_iteration_do(false);
    }

    println!(
        "WV-09-02 GTK event flush stopped by limit label={} limit={}",
        label, GTK_FLUSH_MAX_ITERATIONS
    );
}

fn flush_gtk_events_throttled(label: &str) {
    unsafe {
        let now = Instant::now();

        let should_flush = LAST_GTK_FLUSH_AT
            .map(|last| now.duration_since(last) >= GTK_FLUSH_INTERVAL)
            .unwrap_or(true);

        if !should_flush {
            return;
        }

        LAST_GTK_FLUSH_AT = Some(now);
    }

    flush_gtk_events_bounded(label);
}
