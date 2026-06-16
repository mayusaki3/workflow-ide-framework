//! Linux向け Native Child Window 再導入検証処理。
//!
//! WV-09-06-04
//!
//! 役割:
//! - GTK Host Window を Native Child Window 相当の検証対象として生成する。
//! - GtkFixed および WebKitGTK WebView を生成する。
//! - eframe / winit 実行中に Native Child Window の位置・サイズ・表示状態を Dock矩形へ追従させる。
//! - WV-09-04で応答なし非再現だった構成へ Native Child Window 管理を再導入し、応答なしが再現するか確認する。
//!
//! 注意:
//! - 技術検証用コード。
//! - GDK_BACKEND=x11 前提。
//! - WV-09-04 の WebKitGTK + eframe / winit 共存構成を前提に、Native Child Window 管理を再導入する。
//! - この検証では原因切り分けを優先し、WebView の内容は固定HTMLとする。

use eframe::{egui, CreationContext};
use gtk::prelude::*;
use std::time::{Duration, Instant};
use wry::dpi::{PhysicalPosition, PhysicalSize};
use wry::{Rect, WebView, WebViewBuilder, WebViewBuilderExtUnix};

static mut GTK_WINDOW: Option<gtk::Window> = None;
static mut ROOT_FIXED: Option<gtk::Fixed> = None;
static mut WEBVIEW: Option<WebView> = None;
static mut LAST_SURFACE_STATE: Option<SurfaceState> = None;
static mut LAST_GTK_FLUSH_AT: Option<Instant> = None;

const GTK_FLUSH_MAX_ITERATIONS: usize = 64;
const GTK_FLUSH_INTERVAL: Duration = Duration::from_millis(500);

const WV_09_05_HTML: &str = r#"
<!doctype html>
<html lang="ja">
<head>
  <meta charset="utf-8">
  <style>
    html, body {
      margin: 0;
      padding: 0;
      width: 100%;
      height: 100%;
      overflow: hidden;
      background: #20242a;
      color: #f0f0f0;
      font-family: sans-serif;
    }
    main {
      box-sizing: border-box;
      width: 100%;
      height: 100%;
      padding: 16px;
      border: 4px solid #ffb74d;
    }
    h1 {
      margin: 0 0 8px 0;
      font-size: 20px;
    }
    p {
      margin: 4px 0;
      font-size: 14px;
    }
  </style>
</head>
<body>
  <main>
    <h1>WV-09-06-04</h1>
    <p>WebView visibility continuous call disabled validation</p>
    <p>WebView set_visible calls are disabled during continuous visibility synchronization.</p>
  </main>
</body>
</html>
"#;

#[derive(Clone, Copy, PartialEq, Eq)]
struct SurfaceState {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    visible: bool,
}

/// GTK Host Window と GtkFixed 階層を初期化する。
///
/// # 引数
///
/// * `_cc` - eframe の作成コンテキスト。この検証では利用しない。
///
/// # 戻り値
///
/// なし。初期化に失敗した場合はログを出力して処理を中断する。
pub fn initialize_root_window(_cc: &CreationContext<'_>) {
    println!("WV-09-06-04 gtk::init start");

    if let Err(err) = gtk::init() {
        println!("WV-09-06-04 gtk::init failed: {}", err);
        return;
    }

    println!("WV-09-06-04 gtk::init success");

    let window = gtk::Window::new(gtk::WindowType::Popup);
    window.set_title("WV-09-06-04 Native Child Window Host");
    window.set_default_size(300, 200);

    let root_fixed = gtk::Fixed::new();
    window.add(&root_fixed);
    window.show_all();

    unsafe {
        ROOT_FIXED = Some(root_fixed);
        GTK_WINDOW = Some(window);
    }

    println!("WV-09-06-04 GTK_WINDOW and ROOT_FIXED stored");

    flush_gtk_events_bounded("WV-09-06-04 initialize");
}

/// WebKitGTK WebView を初期化し、Native Child Window を Dock矩形へ追従させる。
///
/// # 引数
///
/// * `initial_rect` - WebView Panel のDock矩形。存在しない場合はNative Surfaceを非表示にする。
/// * `scale` - egui の pixels_per_point。
///
/// # 戻り値
///
/// なし。WebView生成または再配置に失敗した場合はログを出力する。
pub fn ensure_webview_initialized(initial_rect: Option<egui::Rect>, scale: f32) {
    let rect = match initial_rect {
        Some(rect) => rect,
        None => {
            sync_webview_visibility(false);
            return;
        }
    };

    ensure_webkit_webview();

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
            flush_gtk_events_throttled("WV-09-06-04 unchanged");
            return;
        }

        LAST_SURFACE_STATE = Some(state);

        if let Some(window) = GTK_WINDOW.as_ref() {
            println!(
                "WV-09-06-04 native child window sync start x={} y={} w={} h={}",
                x, y, width, height
            );

            window.move_(x, y);
            window.resize(width, height);
            window.show_all();

            println!("WV-09-06-04 native child window sync done");
        }

        if let Some(webview) = WEBVIEW.as_ref() {
            println!(
                "WV-09-06-04 webview set_bounds start x={} y={} w={} h={}",
                x, y, width, height
            );

            if let Err(err) = webview.set_bounds(Rect {
                position: PhysicalPosition::new(0, 0).into(),
                size: PhysicalSize::new(width as u32, height as u32).into(),
            }) {
                println!("WV-09-06-04 webview set_bounds failed: {}", err);
            } else {
                println!("WV-09-06-04 webview set_bounds success");
            }

            if let Err(err) = webview.set_visible(true) {
                println!("WV-09-06-04 webview set_visible(true) failed: {}", err);
            } else {
                println!("WV-09-06-04 webview set_visible(true) success");
            }
        }
    }

    flush_gtk_events_throttled("WV-09-06-04 sync");
}

/// Native Surface の表示状態を同期する。
///
/// # 引数
///
/// * `_ctx` - egui Context。この検証では利用しない。
/// * `_webview_rect` - WebView Panel の矩形。この検証では `ensure_webview_initialized()` 側で処理する。
/// * `should_show_native_surface` - Native Surface を表示する場合は true。
///
/// # 戻り値
///
/// なし。
pub fn sync_child_window(
    _ctx: &egui::Context,
    _webview_rect: Option<egui::Rect>,
    should_show_native_surface: bool,
) {
    sync_webview_visibility(should_show_native_surface);
}

/// WebKitGTK WebView を一度だけ生成する。
///
/// # 戻り値
///
/// なし。すでに生成済みの場合は何もしない。
fn ensure_webkit_webview() {
    unsafe {
        if WEBVIEW.is_some() {
            return;
        }

        let Some(root_fixed) = ROOT_FIXED.as_ref() else {
            println!("WV-09-06-04 WebView build skipped: ROOT_FIXED is none");
            return;
        };

        println!("WV-09-06-04 WebView build_gtk start");

        match WebViewBuilder::new()
            .with_html(WV_09_05_HTML)
            .build_gtk(root_fixed)
        {
            Ok(webview) => {
                WEBVIEW = Some(webview);
                println!("WV-09-06-04 WebView build_gtk success");
            }
            Err(err) => {
                println!("WV-09-06-04 WebView build_gtk failed: {}", err);
            }
        }
    }

    flush_gtk_events_bounded("WV-09-06-04 webview build");
}

/// GTK Host Window と WebView の表示状態を同期する。
///
/// # 引数
///
/// * `visible` - 表示する場合は true、非表示にする場合は false。
///
/// # 戻り値
///
/// なし。
fn sync_webview_visibility(visible: bool) {
    unsafe {
        if let Some(window) = GTK_WINDOW.as_ref() {
            if visible {
                println!("WV-09-06-04 native child window show");
                window.show_all();
            } else {
                println!("WV-09-06-04 native child window hide");
                window.hide();
            }
        }

        if let Some(webview) = WEBVIEW.as_ref() {
            println!("WV-09-06-04 webview set_visible({}) start", visible);

            if let Err(err) = webview.set_visible(visible) {
                println!("WV-09-06-04 webview set_visible({}) failed: {}", visible, err);
            } else {
                println!("WV-09-06-04 webview set_visible({}) success", visible);
            }
        }
    }

    flush_gtk_events_throttled("WV-09-06-04 visibility");
}

/// egui座標をNative Surface向けの整数座標へ変換する。
///
/// # 引数
///
/// * `rect` - egui の矩形。
/// * `scale` - pixels_per_point。
///
/// # 戻り値
///
/// `(x, y, width, height)` の整数座標。
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

/// GTKイベントを上限付きで処理する。
///
/// # 引数
///
/// * `label` - ログ識別子。
///
/// # 戻り値
///
/// なし。
fn flush_gtk_events_bounded(label: &str) {
    println!("WV-09-06-04 GTK event flush start label={}", label);

    for iteration in 0..GTK_FLUSH_MAX_ITERATIONS {
        if !gtk::events_pending() {
            println!(
                "WV-09-06-04 GTK event flush completed label={} iterations={}",
                label, iteration
            );
            return;
        }

        gtk::main_iteration_do(false);
    }

    println!(
        "WV-09-06-04 GTK event flush stopped by limit label={} limit={}",
        label, GTK_FLUSH_MAX_ITERATIONS
    );
}

/// GTKイベントを一定間隔で処理する。
///
/// # 引数
///
/// * `label` - ログ識別子。
///
/// # 戻り値
///
/// なし。
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
