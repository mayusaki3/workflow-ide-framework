//! Linux向け WebView / GTK Host Window PoC処理。
//!
//! # 役割
//!
//! - Windows版 `windows_webview.rs` の `SetParent()` 構成に対応する Linux / X11 検証を行う。
//! - eframe / winit の X11 Window ID を親として取得する。
//! - GTK Toplevel Window の XID を取得し、`XReparentWindow()` で eframe Window の子にする。
//! - GTK / WebKitGTK を生成した状態で、親子化により応答なし条件が変化するか確認する。
//!
//! # 注意点
//!
//! - 技術検証用コードである。
//! - `WINIT_UNIX_BACKEND=x11` による X11 実行を前提とする。
//! - WV-10-08 の `WebView::set_bounds()` 停止状態は維持する。
//! - WV-10-09 の `root_fixed.set_sensitive(false)` 状態は維持する。
//! - 本検証の追加点は X11 親子化である。

use eframe::{egui, CreationContext};
use gtk::glib::translate::ToGlibPtr;
use gtk::prelude::*;
use std::os::raw::{c_char, c_int, c_uint, c_ulong, c_void};
use std::ptr;
use std::time::{Duration, Instant};
use wry::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use wry::{WebView, WebViewBuilder, WebViewBuilderExtUnix};

static mut ROOT_XID: Option<c_ulong> = None;
static mut GTK_WINDOW: Option<gtk::Window> = None;
static mut ROOT_FIXED: Option<gtk::Fixed> = None;
static mut WEBVIEW: Option<WebView> = None;
static mut GTK_XID: Option<c_ulong> = None;
static mut X_DISPLAY: Option<*mut c_void> = None;
static mut LAST_SURFACE_STATE: Option<SurfaceState> = None;
static mut LAST_VISIBLE: Option<bool> = None;
static mut LAST_GTK_FLUSH_AT: Option<Instant> = None;

/// GTKイベント flush の最大処理回数。
const GTK_FLUSH_MAX_ITERATIONS: usize = 64;

/// GTKイベント flush の最小間隔。
const GTK_FLUSH_INTERVAL: Duration = Duration::from_millis(16);

/// WebView初期表示用HTML。
const DEFAULT_HTML: &str = r#"
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
      border: 4px solid #7dd3fc;
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
    <h1>Linux WebView</h1>
    <p>WV-11 X11 reparent validation.</p>
  </main>
</body>
</html>
"#;

#[link(name = "X11")]
extern "C" {
    fn XOpenDisplay(display_name: *const c_char) -> *mut c_void;
    fn XReparentWindow(
        display: *mut c_void,
        window: c_ulong,
        parent: c_ulong,
        x: c_int,
        y: c_int,
    ) -> c_int;
    fn XMoveResizeWindow(
        display: *mut c_void,
        window: c_ulong,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
    ) -> c_int;
    fn XFlush(display: *mut c_void) -> c_int;
}

#[link(name = "gdk-3")]
extern "C" {
    fn gdk_x11_window_get_xid(window: *mut c_void) -> c_ulong;
}

/// Native Surface の同期状態。
///
/// # 役割
///
/// - 前回同期状態と比較し、不要な GTK / X11 操作を抑制する。
#[derive(Clone, Copy, PartialEq, Eq)]
struct SurfaceState {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

/// Linux向け GTK Root Window を初期化する。
///
/// # 役割
///
/// - eframe / winit の X11 Window ID を取得する。
/// - Xlib Display を開き、GTK Window の親子化に使う。
///
/// # 注意点
///
/// - WebView生成は行わない。
/// - X11以外の RawWindowHandle は対象外とする。
///
/// # 引数
///
/// * `cc` - eframe生成コンテキスト。
///
/// # 戻り値
///
/// なし。
pub fn initialize_root_window(cc: &CreationContext<'_>) {
    unsafe {
        if ROOT_XID.is_some() {
            return;
        }
    }

    let root_xid = match cc.window_handle() {
        Ok(window_handle) => match window_handle.as_raw() {
            RawWindowHandle::Xlib(handle) => Some(handle.window as c_ulong),
            RawWindowHandle::Xcb(handle) => Some(handle.window.get() as c_ulong),
            other => {
                println!("WV-11 unsupported raw window handle: {:?}", other);
                None
            }
        },
        Err(error) => {
            println!("WV-11 window_handle failed: {:?}", error);
            None
        }
    };

    unsafe {
        ROOT_XID = root_xid;

        if let Some(xid) = ROOT_XID {
            println!("WV-11 Root XID = 0x{:x}", xid);
        } else {
            println!("WV-11 Root XID not initialized");
        }

        let display = XOpenDisplay(ptr::null());

        if display.is_null() {
            println!("WV-11 XOpenDisplay failed");
        } else {
            X_DISPLAY = Some(display);
            println!("WV-11 XOpenDisplay success");
        }
    }
}

/// Linux向け WebView を初期化する。
///
/// # 役割
///
/// - GTK Host Window が未生成の場合は初期化する。
/// - GTK Host Window を eframe / winit の X11 Window の子へ reparent する。
/// - WebKitGTK WebView を初回のみ生成する。
/// - 初期矩形が存在する場合は Host Window へ適用する。
///
/// # 注意点
///
/// - WV-10-08同様、`WebView::set_bounds()` は呼ばない。
///
/// # 引数
///
/// * `initial_rect` - 初期配置矩形。
/// * `scale` - egui の pixels_per_point。
///
/// # 戻り値
///
/// なし。
pub fn ensure_webview_initialized(initial_rect: Option<egui::Rect>, scale: f32) {
    ensure_root_window_initialized();
    create_webview();

    if let Some(rect) = initial_rect {
        apply_surface_rect(rect, scale);
    }
}

/// Linux向け Native Surface 追従処理。
///
/// # 役割
///
/// - GTK Host Window を表示・非表示にする。
/// - WebView配置矩形が変化した場合のみ、位置・サイズを同期する。
/// - GTK / WebKitGTK のイベントを低頻度で処理する。
///
/// # 引数
///
/// * `ctx` - egui コンテキスト。
/// * `webview_rect` - WebView配置矩形。
/// * `should_show_native_surface` - Native Surface を表示する場合 true。
///
/// # 戻り値
///
/// なし。
pub fn sync_child_window(
    ctx: &egui::Context,
    webview_rect: Option<egui::Rect>,
    should_show_native_surface: bool,
) {
    sync_child_window_visibility(should_show_native_surface);

    if !should_show_native_surface {
        flush_gtk_events_throttled();
        return;
    }

    if let Some(rect) = webview_rect {
        let scale = ctx.pixels_per_point();
        apply_surface_rect(rect, scale);
    }

    flush_gtk_events_throttled();
}

/// Linux向け Native Surface の表示状態を同期する。
///
/// # 役割
///
/// - 表示状態が変化した場合のみ `show_all()` / `hide()` を実行する。
///
/// # 引数
///
/// * `visible` - 表示する場合 true。
///
/// # 戻り値
///
/// なし。
fn sync_child_window_visibility(visible: bool) {
    unsafe {
        if LAST_VISIBLE == Some(visible) {
            return;
        }

        LAST_VISIBLE = Some(visible);

        if let Some(window) = GTK_WINDOW.as_ref() {
            if visible {
                window.show_all();
            } else {
                window.hide();
            }
        }

        if let Some(webview) = WEBVIEW.as_ref() {
            if let Err(err) = webview.set_visible(visible) {
                println!("Linux WebView set_visible failed: {}", err);
            }
        }
    }
}

/// GTK Root Window が初期化済みであることを保証する。
///
/// # 役割
///
/// - GTK Window / GtkFixed を作成する。
/// - GTK Window の XID を取得する。
/// - `XReparentWindow()` で eframe / winit 側の X11 Window の子へ移す。
///
/// # 戻り値
///
/// なし。
fn ensure_root_window_initialized() {
    unsafe {
        if GTK_WINDOW.is_some() && ROOT_FIXED.is_some() {
            return;
        }
    }

    if let Err(err) = gtk::init() {
        println!("Linux WebView gtk::init failed: {}", err);
        return;
    }

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Linux WebView Host Window");
    window.set_default_size(1, 1);
    window.set_decorated(false);
    window.set_accept_focus(false);

    let root_fixed = gtk::Fixed::new();
    root_fixed.set_sensitive(false);
    window.add(&root_fixed);
    window.realize();

    let gtk_xid = gtk_window_xid(&window);

    unsafe {
        if let Some(xid) = gtk_xid {
            GTK_XID = Some(xid);
            println!("WV-11 GTK XID = 0x{:x}", xid);
            reparent_gtk_window(xid);
        } else {
            println!("WV-11 GTK XID not initialized");
        }
    }

    window.hide();

    unsafe {
        ROOT_FIXED = Some(root_fixed);
        GTK_WINDOW = Some(window);
        LAST_VISIBLE = Some(false);
    }

    flush_gtk_events_bounded();
}

/// GTK Window の XID を取得する。
///
/// # 役割
///
/// - GTK Window の GdkWindow を取得する。
/// - GDK X11 API で XID へ変換する。
///
/// # 引数
///
/// * `window` - GTK Window。
///
/// # 戻り値
///
/// XID。取得できない場合は None。
fn gtk_window_xid(window: &gtk::Window) -> Option<c_ulong> {
    let Some(gdk_window) = window.window() else {
        return None;
    };

    let xid = unsafe {
        let ptr = gdk_window.to_glib_none().0 as *mut c_void;
        gdk_x11_window_get_xid(ptr)
    };

    if xid == 0 {
        None
    } else {
        Some(xid)
    }
}

/// GTK Window を eframe / winit の X11 Window の子へ移す。
///
/// # 役割
///
/// - `XReparentWindow()` により X11 レベルの親子関係を作る。
///
/// # 引数
///
/// * `gtk_xid` - GTK Window の XID。
///
/// # 戻り値
///
/// なし。
unsafe fn reparent_gtk_window(gtk_xid: c_ulong) {
    let Some(display) = X_DISPLAY else {
        println!("WV-11 reparent skipped: X display is none");
        return;
    };

    let Some(parent_xid) = ROOT_XID else {
        println!("WV-11 reparent skipped: root xid is none");
        return;
    };

    let result = XReparentWindow(display, gtk_xid, parent_xid, 0, 0);
    XFlush(display);

    println!(
        "WV-11 XReparentWindow gtk=0x{:x} parent=0x{:x} result={}",
        gtk_xid, parent_xid, result
    );
}

/// WebKitGTK WebView を初回のみ生成する。
///
/// # 役割
///
/// - `ROOT_FIXED` を親として wry WebView を生成する。
/// - 生成した WebView を保持する。
///
/// # 戻り値
///
/// なし。
fn create_webview() {
    unsafe {
        if WEBVIEW.is_some() {
            return;
        }

        let Some(root_fixed) = ROOT_FIXED.as_ref() else {
            println!("Linux WebView create skipped: ROOT_FIXED is none");
            return;
        };

        match WebViewBuilder::new()
            .with_html(DEFAULT_HTML)
            .build_gtk(root_fixed)
        {
            Ok(webview) => {
                WEBVIEW = Some(webview);
                println!("Linux WebView create success");
            }
            Err(err) => {
                println!("Linux WebView create failed: {}", err);
            }
        }
    }

    flush_gtk_events_bounded();
}

/// Native Surface の位置・サイズを同期する。
///
/// # 役割
///
/// - egui矩形をNative Surface用の整数座標へ変換する。
/// - 前回状態と同一の場合は何もしない。
/// - GTK Host Window を X11 親Window内の子Windowとして移動・リサイズする。
/// - WV-10-08同様、WebView の `set_bounds()` は呼ばない。
///
/// # 引数
///
/// * `rect` - egui矩形。
/// * `scale` - egui の pixels_per_point。
///
/// # 戻り値
///
/// なし。
fn apply_surface_rect(rect: egui::Rect, scale: f32) {
    let (x, y, width, height) = rect_to_i32_bounds(rect, scale);
    let state = SurfaceState {
        x,
        y,
        width,
        height,
    };

    unsafe {
        if LAST_SURFACE_STATE == Some(state) {
            return;
        }

        LAST_SURFACE_STATE = Some(state);

        if let (Some(display), Some(gtk_xid)) = (X_DISPLAY, GTK_XID) {
            XMoveResizeWindow(
                display,
                gtk_xid,
                x,
                y,
                width as c_uint,
                height as c_uint,
            );
            XFlush(display);
        }

        if let Some(window) = GTK_WINDOW.as_ref() {
            window.resize(width, height);
        }

        if WEBVIEW.as_ref().is_some() {
            println!("WV-11 webview set_bounds skipped");
        }
    }
}

/// egui矩形を GTK / X11 用 i32 境界値へ変換する。
///
/// # 役割
///
/// - egui座標をスケール適用後の整数座標へ変換する。
/// - 極端な値を制限し、X11側へ不正に大きい値を渡さない。
///
/// # 引数
///
/// * `rect` - egui矩形。
/// * `scale` - スケール値。
///
/// # 戻り値
///
/// `(x, y, width, height)`
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
/// # 役割
///
/// - WebView生成直後など、明示的に GTK イベントを処理する。
/// - pending が残っていても上限回数で打ち切る。
///
/// # 戻り値
///
/// なし。
fn flush_gtk_events_bounded() {
    let mut count = 0usize;

    while gtk::events_pending() {
        count += 1;

        if count <= 10 {
            println!("WV-11 GTK event iteration={}", count);
        }

        gtk::main_iteration_do(false);

        if count >= GTK_FLUSH_MAX_ITERATIONS {
            println!(
                "WV-11 GTK event flush limit reached limit={}",
                GTK_FLUSH_MAX_ITERATIONS
            );
            break;
        }
    }

    if count > 0 {
        println!("WV-11 GTK events processed count={}", count);
    }
}

/// GTKイベントを低頻度で処理する。
///
/// # 役割
///
/// - eframe / winit を停止させずに GTK / WebKitGTK のイベントを継続処理する。
///
/// # 戻り値
///
/// なし。
fn flush_gtk_events_throttled() {
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

    flush_gtk_events_bounded();
}
