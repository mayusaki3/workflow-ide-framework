//! Linux向け WebView / GTK Host Window PoC処理。
//!
//! # 役割
//!
//! - Windows版 `windows_webview.rs` と同等の責務で Linux 向け WebView を管理する。
//! - eframe / egui 側から Linux 固有処理を分離する。
//! - GTK Host Window を Native Surface 相当として生成し、Dock Panel の矩形へ追従させる。
//! - WebKitGTK WebView の生成、位置同期、表示同期を扱う。
//!
//! # 注意点
//!
//! - 技術検証用コードである。
//! - Linux / X11 環境での実行を前提とする。
//! - GTK / WebKitGTK / wry 固有の処理は本ファイル内に閉じ込める。
//! - 応答なし再現確認を目的とするため、検証番号依存の分岐や一時無効化コードは持ち込まない。

use eframe::{egui, CreationContext};
use gtk::prelude::*;
use std::time::{Duration, Instant};
use wry::dpi::{PhysicalPosition, PhysicalSize};
use wry::{Rect, WebView, WebViewBuilder, WebViewBuilderExtUnix};

static mut GTK_WINDOW: Option<gtk::Window> = None;
static mut ROOT_FIXED: Option<gtk::Fixed> = None;
static mut WEBVIEW: Option<WebView> = None;
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
    <p>Final implementation candidate for GTK / WebKitGTK validation.</p>
  </main>
</body>
</html>
"#;

/// Native Surface の同期状態。
///
/// # 役割
///
/// - 前回同期状態と比較し、不要な GTK / WebView 操作を抑制する。
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
/// - GTKを初期化する。
/// - WebViewを配置するための GTK Window と gtk::Fixed を生成する。
/// - 生成した Root Window / Fixed を後続処理で利用できるよう保持する。
///
/// # 注意点
///
/// - WebView生成は行わない。
/// - Windows版の `initialize_root_window()` と同じく、Root取得・保持のみを担当する。
/// - すでに初期化済みの場合は何もしない。
///
/// # 引数
///
/// * `_cc` - eframe生成コンテキスト。Linux版では GTK Host Window を別途生成するため使用しない。
///
/// # 戻り値
///
/// なし。
pub fn initialize_root_window(_cc: &CreationContext<'_>) {
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

    let root_fixed = gtk::Fixed::new();
    window.add(&root_fixed);
    window.hide();

    unsafe {
        ROOT_FIXED = Some(root_fixed);
        GTK_WINDOW = Some(window);
        LAST_VISIBLE = Some(false);
    }

    flush_gtk_events_bounded();
}

/// Linux向け WebView を初期化する。
///
/// # 役割
///
/// - GTK Host Window が未生成の場合は初期化する。
/// - WebKitGTK WebView を初回のみ生成する。
/// - 初期矩形が存在する場合は Host Window / WebView へ適用する。
///
/// # 注意点
///
/// - Windows版の `ensure_webview_initialized()` と同じく、Child Surface 相当と WebView の生成を担当する。
/// - 2回目以降は WebView を再生成しない。
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
/// - Native Surface 相当の GTK Host Window を表示・非表示にする。
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
/// - 毎フレームの show / hide 呼び出しを避ける。
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
/// - App初期化順序によって `initialize_root_window()` が未実行の場合に備える。
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

    let root_fixed = gtk::Fixed::new();
    window.add(&root_fixed);
    window.hide();

    unsafe {
        ROOT_FIXED = Some(root_fixed);
        GTK_WINDOW = Some(window);
        LAST_VISIBLE = Some(false);
    }

    flush_gtk_events_bounded();
}

/// WebKitGTK WebView を初回のみ生成する。
///
/// # 役割
///
/// - `ROOT_FIXED` を親として wry WebView を生成する。
/// - 生成した WebView を保持する。
///
/// # 注意点
///
/// - 既に生成済みの場合は何もしない。
/// - Rect同期は行わない。
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
/// - GTK Host Window と WebView bounds を同期する。
/// - WV-10-08では WebView の `set_bounds()` のみを停止し、入力イベントとの相互作用を切り分ける。
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

        if let Some(window) = GTK_WINDOW.as_ref() {
            window.move_(x, y);
            window.resize(width, height);
        }

        if WEBVIEW.as_ref().is_some() {
            println!("WV-10-08 webview set_bounds skipped");
        }
    }
}

/// egui矩形を GTK / wry 用 i32 境界値へ変換する。
///
/// # 役割
///
/// - egui座標をスケール適用後の整数座標へ変換する。
/// - 極端な値を制限し、GTK側へ不正に大きい値を渡さない。
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
/// - WV-10-06 のため、処理件数と上限到達有無をログ出力する。
///
/// # 戻り値
///
/// なし。
fn flush_gtk_events_bounded() {
    let mut count = 0usize;

    while gtk::events_pending() {
        count += 1;

        if count <= 10 {
            println!("WV-10-06 GTK event iteration={}", count);
        }

        gtk::main_iteration_do(false);

        if count >= GTK_FLUSH_MAX_ITERATIONS {
            println!(
                "WV-10-06 GTK event flush limit reached limit={}",
                GTK_FLUSH_MAX_ITERATIONS
            );
            break;
        }
    }

    if count > 0 {
        println!("WV-10-06 GTK events processed count={}", count);
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
