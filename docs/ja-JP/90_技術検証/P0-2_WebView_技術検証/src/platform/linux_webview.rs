//! Linux向け WebView / GTK Fixed PoC処理。
//!
//! WV-04-07
//!
//! 前回(WV-04-06)
//! - build_gtk() 成功
//! - move_() 成功
//! - set_size_request() 成功
//! - 状態変化時のみ同期
//! - ただし UI 応答停止は継続
//!
//! 今回(WV-04-07)
//! - flush_gtk_events() を sync_child_window() から除去
//! - GTKイベント処理が応答停止原因か切り分ける

use eframe::{egui, CreationContext};
use gtk::prelude::*;
use wry::{
    dpi::{LogicalPosition, LogicalSize},
    Rect, WebViewBuilder, WebViewBuilderExtUnix,
};

static mut GTK_WINDOW: Option<gtk::Window> = None;
static mut ROOT_FIXED: Option<gtk::Fixed> = None;
static mut CHILD_FIXED: Option<gtk::Fixed> = None;
static mut WEBVIEW_CREATED: bool = false;
static mut WEBVIEW: Option<wry::WebView> = None;

#[derive(Clone, Copy, PartialEq, Eq)]
struct SurfaceState {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    visible: bool,
}

static mut LAST_SURFACE_STATE: Option<SurfaceState> = None;

pub fn initialize_root_window(_cc: &CreationContext<'_>) {
    unsafe {
        if GTK_WINDOW.is_some() {
            return;
        }

        if let Err(error) = gtk::init() {
            println!("WV-04 Linux gtk::init failed = {:?}", error);
            return;
        }

        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_title("WV-04 Linux GTK WebView Host");
        window.set_default_size(800, 600);

        let root_fixed = gtk::Fixed::new();
        root_fixed.set_size_request(800, 600);

        window.add(&root_fixed);
        window.show_all();

        GTK_WINDOW = Some(window);
        ROOT_FIXED = Some(root_fixed);

        println!("WV-04 Linux GTK root window initialized");
    }
}

pub fn ensure_webview_initialized(
    initial_rect: Option<egui::Rect>,
    scale: f32,
) {
    unsafe {
        if WEBVIEW_CREATED {
            return;
        }

        let Some(root_fixed) = ROOT_FIXED.as_ref() else {
            println!("WV-04 Linux ROOT_FIXED not initialized");
            return;
        };

        let (x, y, width, height) = initial_rect
            .map(|rect| rect_to_i32_bounds(rect, scale))
            .unwrap_or((0, 0, 800, 600));

        let child_fixed = gtk::Fixed::new();
        child_fixed.set_size_request(width, height);

        root_fixed.put(&child_fixed, x, y);
        child_fixed.show_all();
        root_fixed.show_all();

        let bounds = Rect {
            position: LogicalPosition::new(0, 0).into(),
            size: LogicalSize::new(width as u32, height as u32).into(),
        };

        let result = WebViewBuilder::new()
            .with_bounds(bounds)
            .with_url("https://example.com")
            .build_gtk(&child_fixed);

        match result {
            Ok(webview) => {
                CHILD_FIXED = Some(child_fixed);
                WEBVIEW = Some(webview);
                WEBVIEW_CREATED = true;

                println!("WV-04 Linux WebView create success");
            }
            Err(error) => {
                println!("WV-04 Linux WebView create failed = {:?}", error);
            }
        }

        flush_gtk_events();
    }
}

pub fn sync_child_window(
    _ctx: &egui::Context,
    webview_rect: Option<egui::Rect>,
    should_show_native_surface: bool,
) {
    unsafe {
        let Some(root_fixed) = ROOT_FIXED.as_ref() else {
            return;
        };

        let Some(child_fixed) = CHILD_FIXED.as_ref() else {
            return;
        };

        let (x, y, width, height) = webview_rect
            .map(|r| rect_to_i32_bounds(r, 1.0))
            .unwrap_or((0, 0, 800, 600));

        let new_state = SurfaceState {
            x,
            y,
            width,
            height,
            visible: should_show_native_surface,
        };

        if LAST_SURFACE_STATE == Some(new_state) {
            return;
        }

        LAST_SURFACE_STATE = Some(new_state);

        if !new_state.visible {
            child_fixed.hide();

            println!("WV-04-07 hide child surface");

            return;
        }

        child_fixed.show();

        root_fixed.move_(
            child_fixed,
            new_state.x,
            new_state.y,
        );

        child_fixed.set_size_request(
            new_state.width,
            new_state.height,
        );

        println!(
            "WV-04-07 sync child surface x={} y={} w={} h={}",
            new_state.x,
            new_state.y,
            new_state.width,
            new_state.height
        );

        root_fixed.show_all();
    }
}

fn rect_to_i32_bounds(
    rect: egui::Rect,
    scale: f32,
) -> (i32, i32, i32, i32) {
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

fn flush_gtk_events() {
    while gtk::events_pending() {
        gtk::main_iteration_do(false);
    }
}
