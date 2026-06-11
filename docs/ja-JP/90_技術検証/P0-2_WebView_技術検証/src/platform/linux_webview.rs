//! Linux向け WebView / GTK Fixed PoC処理。
//!
//! WV-08-17
//!
//! 役割:
//! - gtk::init()、GTK Window生成、Root Fixed生成、Child Fixed生成、Child Fixed move/resize、GTK Label生成、window.show_all() を実行する。
//! - Root Fixed を GTK Window に追加し、Child Fixed を Root Fixed に追加する。
//! - Child Fixed を固定位置へ移動し、固定サイズを設定する。
//! - GTK Label を Child Fixed に追加する。
//! - wry::WebViewBuilderExtUnix::build_gtk() により WebKitGTK WebView を Child Fixed へ最小追加する。
//! - build_gtk() 成功後、WebView::set_bounds() を1回だけ実行する。
//! - WebView を static に保持し、Dock矩形変化時に set_bounds() を継続実行する。
//! - Dock矩形変化時に set_bounds() を継続実行する。
//! - flush_gtk_events_throttled() を継続する。
//! - should_show_native_surface に連動して Native Surface 表示切替を実行する。
//! - GTK Window / Root Fixed / Child Fixed / WebView を static に保持する。
//! - show_all() 直後に上限付き GTKイベントflush を1回実行する。
//! - Dummy GTK Widget生成、継続的なGTKイベント処理は実行しない。
//! - WebKitGTK生成 + GtkFixed attach + WebView set_bounds 継続実行 + GTKイベントポンプ + Native Surface表示切替で応答なしが発生するか確認する。
//!
//! 注意:
//! - 技術検証用コード。
//! - WV-08-16では Dummy GTK Widget は使用しない。
//! - WV-08-16では show_all() 後に加え、Dock追従時に低頻度 GTKイベントflush を呼び出す。
//! - build_gtk() は WebKitGTK生成に加え、GtkFixed の set_size_request() と put() まで実行する。
//! - set_bounds() は GtkFixed 配下では WebView Widget への size_allocate() を実行する。
//! - Dock矩形が変化した場合のみ set_bounds() を実行する。
//! - should_show_native_surface に連動した Native Surface表示切替を検証する。

use eframe::{egui, CreationContext};
use gtk::prelude::*;
use std::time::{Duration, Instant};
use wry::{dpi, Rect, WebView, WebViewBuilder, WebViewBuilderExtUnix};

static mut GTK_WINDOW: Option<gtk::Window> = None;
static mut ROOT_FIXED: Option<gtk::Fixed> = None;
static mut CHILD_FIXED: Option<gtk::Fixed> = None;
static mut WEBVIEW: Option<WebView> = None;
static mut WEBVIEW_CREATED: bool = false;
static mut LAST_SURFACE_STATE: Option<SurfaceState> = None;
static mut LAST_GTK_FLUSH_AT: Option<Instant> = None;

/// GTKイベント flush の最大処理回数。
///
/// WV-08-13:
/// - show_all() 後に1回だけ使用する。
/// - pending が残る場合でも、上限回数で打ち切る。
const GTK_FLUSH_MAX_ITERATIONS: usize = 64;

/// GTKイベント flush の最小間隔。
///
/// WV-08-13:
/// - 継続的なGTKイベントポンプは使用しない。
/// - 後続検証で再利用する可能性があるため残置する。
const GTK_FLUSH_INTERVAL: Duration = Duration::from_millis(500);

/// Native Surface の同期状態。
///
/// 役割:
/// - 前回同期状態と比較し、不要な WebView set_bounds() / set_visible() を抑制する。
#[derive(Clone, Copy, PartialEq, Eq)]
struct SurfaceState {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    visible: bool,
}

/// Linux向け GTK Host Window を初期化する。
///
/// 役割:
/// - GTKを初期化する。
/// - GTK Host Window を Toplevel Window として生成する。
/// - Root Fixed を生成し、GTK Host Window に追加する。
/// - Child Fixed を生成し、Root Fixed に追加する。
/// - Child Fixed の位置とサイズを固定値で設定する。
/// - GTK Label を生成し、Child Fixed に追加する。
/// - WebKitGTK WebView を build_gtk(&child_fixed) で最小生成する。
/// - WebView::set_bounds() を1回だけ実行する。
/// - WebView を static に保持する。
/// - GTK Host Window を表示し、Window / Root Fixed / Child Fixed を static に保持する。
/// - show_all() 後に GTKイベントflush を1回だけ実行する。
///
/// 注意:
/// - WV-08-13では GDK_BACKEND=x11 での実行を前提とする。
/// - WV-08-13では Dummy GTK Widget、継続的な GTKイベントポンプは使用しない。
/// - build_gtk() は WebView生成 + GtkFixedへの追加までを実行する。
/// - 初期 set_bounds() 後、Dock矩形変化に応じて ensure_webview_initialized() 側で set_bounds() を継続実行する。
/// - WebView表示状態は ensure_webview_initialized() 側で set_visible() により制御する。
///
/// 引数:
/// - _cc: eframe生成コンテキスト。
///
/// 戻り値:
/// - なし。
pub fn initialize_root_window(_cc: &CreationContext<'_>) {
    println!("WV-08-13 gtk::init start");

    match gtk::init() {
        Ok(_) => {
            println!("WV-08-13 gtk::init success");
        }
        Err(err) => {
            println!("WV-08-13 gtk::init failed: {}", err);
            return;
        }
    }

    let window = gtk::Window::new(gtk::WindowType::Popup);

    println!("WV-08-13 gtk::Window created");

    let root_fixed = gtk::Fixed::new();

    println!("WV-08-13 root_fixed created");

    window.add(&root_fixed);

    println!("WV-08-13 root_fixed attached");

    let child_fixed = gtk::Fixed::new();

    println!("WV-08-13 child_fixed created");

    root_fixed.put(&child_fixed, 0, 0);

    println!("WV-08-13 child_fixed attached");

    root_fixed.move_(&child_fixed, 100, 100);

    println!("WV-08-13 child_fixed moved");

    child_fixed.set_size_request(300, 200);

    println!("WV-08-13 child_fixed resized");

    let label = gtk::Label::new(Some("WV-08-13"));

    println!("WV-08-13 label created");

    child_fixed.put(&label, 0, 0);

    println!("WV-08-13 label attached");

    let webview_bounds = Rect {
        position: dpi::LogicalPosition::new(0, 0).into(),
        size: dpi::LogicalSize::new(300, 200).into(),
    };

    println!("WV-08-13 webview build_gtk start");

    let webview_result = WebViewBuilder::new()
        .with_url("about:blank")
        .with_bounds(webview_bounds)
        .build_gtk(&child_fixed);

    let webview_created = match webview_result {
        Ok(webview) => {
            println!("WV-08-13 webview build_gtk success");

            let updated_bounds = Rect {
                position: dpi::LogicalPosition::new(20, 20).into(),
                size: dpi::LogicalSize::new(280, 180).into(),
            };

            println!("WV-08-13 webview initial set_bounds start");

            match webview.set_bounds(updated_bounds) {
                Ok(_) => {
                    println!("WV-08-13 webview initial set_bounds success");
                }
                Err(err) => {
                    println!("WV-08-13 webview initial set_bounds failed: {}", err);
                }
            }

            unsafe {
                WEBVIEW = Some(webview);
            }

            true
        }
        Err(err) => {
            println!("WV-08-13 webview build_gtk failed: {}", err);
            false
        }
    };

    window.show_all();

    println!("WV-08-13 window.show_all done");

    unsafe {
        GTK_WINDOW = Some(window);
        ROOT_FIXED = Some(root_fixed);
        CHILD_FIXED = Some(child_fixed);
        WEBVIEW_CREATED = webview_created;
    }

    println!("WV-08-13 GTK_WINDOW stored");
    println!("WV-08-13 ROOT_FIXED stored");
    println!("WV-08-13 CHILD_FIXED stored");
    println!("WV-08-13 WEBVIEW_CREATED={}", unsafe { WEBVIEW_CREATED });

    flush_gtk_events_bounded("WV-08-13");

    println!("WV-08-13 GTK flush done");
}

/// Linux向け WebView を初期化・追従する。
///
/// 役割:
/// - WV-08-13では WebViewの生成は initialize_root_window() 内で行う。
/// - WebView配置矩形がある場合は WebView::set_visible(true) を実行する。
/// - WebView配置矩形がない場合は WebView::set_visible(false) を実行する。
/// - Dock矩形が変化した場合のみ WebView::set_bounds() を実行する。
///
/// 注意:
/// - WV-08-13では継続的な GTKイベントポンプを実行しない。
/// - WV-08-13では Native Surface表示切替のうち WebView::set_visible() のみを検証する。
///
/// 引数:
/// - initial_rect: WebView配置矩形。
/// - _scale: egui のスケール値。
///
/// 戻り値:
/// - なし。
pub fn ensure_webview_initialized(
    initial_rect: Option<egui::Rect>,
    _scale: f32,
) {
    let rect = match initial_rect {
        Some(rect) => rect,
        None => {
            let state = SurfaceState {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
                visible: false,
            };

            unsafe {
                if LAST_SURFACE_STATE == Some(state) {
                    return;
                }

                LAST_SURFACE_STATE = Some(state);

                if let Some(webview) = WEBVIEW.as_ref() {
                    println!("WV-08-13 set_visible(false) start");

                    match webview.set_visible(false) {
                        Ok(_) => {
                            println!("WV-08-13 set_visible(false) success");
                        }
                        Err(err) => {
                            println!("WV-08-13 set_visible(false) failed: {}", err);
                        }
                    }
                }
            }

            return;
        }
    };

    let (x, y, width, height) = rect_to_i32_bounds(rect, 1.0);

    let state = SurfaceState {
        x,
        y,
        width,
        height,
        visible: true,
    };

    unsafe {
        if LAST_SURFACE_STATE == Some(state) {
            return;
        }

        LAST_SURFACE_STATE = Some(state);

        if let Some(webview) = WEBVIEW.as_ref() {
            println!("WV-08-13 set_visible(true) start");

            match webview.set_visible(true) {
                Ok(_) => {
                    println!("WV-08-13 set_visible(true) success");
                }
                Err(err) => {
                    println!("WV-08-13 set_visible(true) failed: {}", err);
                }
            }

            println!(
                "WV-08-13 set_bounds start x={} y={} w={} h={}",
                x, y, width, height
            );

            let bounds = Rect {
                position: dpi::LogicalPosition::new(x, y).into(),
                size: dpi::LogicalSize::new(width, height).into(),
            };

            match webview.set_bounds(bounds) {
                Ok(_) => {
                    println!("WV-08-13 set_bounds success");
                }
                Err(err) => {
                    println!("WV-08-13 set_bounds failed: {}", err);
                }
            }

            println!("WV-08-16 throttled GTK flush request");

            flush_gtk_events_throttled("WV-08-16");
        }
    }
}

/// Linux向け Child Surface 追従処理。
///
/// 役割:
/// - WV-08-17では should_show_native_surface に連動して WebView 表示状態を切り替える。
/// - Native Surface 表示切替経路で応答なしが発生するか確認する。
///
/// 引数:
/// - _ctx: egui コンテキスト。
/// - _webview_rect: WebView配置矩形。
/// - should_show_native_surface: Native Surface を表示する場合 true。
///
/// 戻り値:
/// - なし。
pub fn sync_child_window(
    _ctx: &egui::Context,
    _webview_rect: Option<egui::Rect>,
    should_show_native_surface: bool,
) {
    unsafe {
        if let Some(webview) = WEBVIEW.as_ref() {
            if should_show_native_surface {
                println!("WV-08-17 native surface show start");

                match webview.set_visible(true) {
                    Ok(_) => {
                        println!("WV-08-17 native surface show success");
                    }
                    Err(err) => {
                        println!("WV-08-17 native surface show failed: {}", err);
                    }
                }
            } else {
                println!("WV-08-17 native surface hide start");

                match webview.set_visible(false) {
                    Ok(_) => {
                        println!("WV-08-17 native surface hide success");
                    }
                    Err(err) => {
                        println!("WV-08-17 native surface hide failed: {}", err);
                    }
                }
            }
        }
    }
}

/// egui矩形を GTK / wry 用 i32 境界値へ変換する。
///
/// 役割:
/// - egui座標をスケール適用後の整数座標へ変換する。
/// - 極端な値を制限し、GTK側へ不正に大きい値を渡さない。
///
/// 引数:
/// - rect: egui矩形。
/// - scale: スケール値。
///
/// 戻り値:
/// - (x, y, width, height)
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

/// GTKイベントを上限付きで処理する。
///
/// 役割:
/// - show_all() 直後など、明示的に GTK イベントを処理する。
/// - pending が残っていても上限回数で打ち切る。
///
/// 注意:
/// - WV-08-13では show_all() 後に1回だけ呼び出す。
///
/// 引数:
/// - label: ログ識別名。
///
/// 戻り値:
/// - なし。
fn flush_gtk_events_bounded(label: &str) {
    println!("WV-08-13 GTK event flush start label={}", label);

    for iteration in 0..GTK_FLUSH_MAX_ITERATIONS {
        if !gtk::events_pending() {
            println!(
                "WV-08-13 GTK event flush completed label={} iterations={}",
                label,
                iteration
            );
            return;
        }

        gtk::main_iteration_do(false);
    }

    println!(
        "WV-08-13 GTK event flush stopped by limit label={} limit={}",
        label,
        GTK_FLUSH_MAX_ITERATIONS
    );
}

/// GTKイベントを低頻度で処理する。
///
/// 役割:
/// - eframe / winit を停止させずに GTK / WebKitGTK のイベントを継続処理する。
/// - GTK Host Window の応答停止を防げるか確認する。
///
/// 注意:
/// - WV-08-16では Dock追従時に呼び出す。
///
/// 引数:
/// - label: ログ識別名。
///
/// 戻り値:
/// - なし。
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
