//! Linux向け WebView / GTK Fixed PoC処理。
//!
//! WV-05-06
//!
//! 役割:
//! - WV-04で選定した build_gtk() + gtk::Fixed 方式を維持する。
//! - WebKitGTK と eframe のイベントループ共存状態をログで確認する。
//! - eframe / winit を止めずに GTK / WebKitGTK のイベントを低頻度で処理する。
//!
//! 注意:
//! - 技術検証用コード。
//! - build_gtk(), move_(), set_size_request() の検証済み経路を維持する。
//! - GTKイベント処理は上限付き、かつスロットリング付きで実行する。

use eframe::{egui, CreationContext};
use gtk::glib::{self, ControlFlow};
use gtk::prelude::*;
use std::time::{Duration, Instant};
use wry::{
    dpi::{LogicalPosition, LogicalSize},
    Rect, WebViewBuilder, WebViewBuilderExtUnix,
};

static mut GTK_WINDOW: Option<gtk::Window> = None;
static mut ROOT_FIXED: Option<gtk::Fixed> = None;
static mut CHILD_FIXED: Option<gtk::Fixed> = None;
static mut WEBVIEW_CREATED: bool = false;
static mut WEBVIEW: Option<wry::WebView> = None;
static mut GTK_TIMER_LOG_INSTALLED: bool = false;
static mut LAST_SURFACE_STATE: Option<SurfaceState> = None;
static mut LAST_EFRAME_ALIVE_LOG_AT: Option<Instant> = None;
static mut LAST_GTK_FLUSH_AT: Option<Instant> = None;

/// GTKイベント flush の最大処理回数。
const GTK_FLUSH_MAX_ITERATIONS: usize = 64;

/// GTKイベント flush の最小間隔。
const GTK_FLUSH_INTERVAL: Duration = Duration::from_millis(100);

/// Native Surface の同期状態。
///
/// 役割:
/// - 前回同期状態と比較し、不要な GTK Widget 操作を抑制する。
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
/// - WebViewを配置するための GTK Window と gtk::Fixed を生成する。
/// - MainContext 所有状態をログ出力する。
///
/// 引数:
/// - _cc: eframe生成コンテキスト。
///
/// 戻り値:
/// - なし。
pub fn initialize_root_window(_cc: &CreationContext<'_>) {
    unsafe {
        if GTK_WINDOW.is_some() {
            println!("WV-05 Linux GTK root window already initialized");
            return;
        }

        println!("WV-05 Linux initialize_root_window begin");

        if let Err(error) = gtk::init() {
            println!("WV-05 Linux gtk::init failed = {:?}", error);
            return;
        }

        log_main_context_state("after gtk::init");

        install_gtk_timer_logger();

        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_title("WV-05 Linux GTK WebView Host");
        window.set_default_size(800, 600);

        let root_fixed = gtk::Fixed::new();
        root_fixed.set_size_request(800, 600);

        window.add(&root_fixed);
        window.show_all();

        GTK_WINDOW = Some(window);
        ROOT_FIXED = Some(root_fixed);

        println!("WV-05 Linux GTK root window initialized");
        log_main_context_state("after GTK root window initialized");
    }
}

/// Linux向け WebView を初期化する。
///
/// 役割:
/// - gtk::Fixed 上に Child Surface 用 gtk::Fixed を配置する。
/// - wry build_gtk() で WebView を生成する。
/// - WebView生成前後の MainContext 所有状態をログ出力する。
///
/// 引数:
/// - initial_rect: 初期配置矩形。
/// - scale: egui のスケール値。
///
/// 戻り値:
/// - なし。
pub fn ensure_webview_initialized(
    initial_rect: Option<egui::Rect>,
    scale: f32,
) {
    unsafe {
        if WEBVIEW_CREATED {
            return;
        }

        println!("WV-05 Linux ensure_webview_initialized begin");
        log_main_context_state("before WebView initialize");

        let Some(root_fixed) = ROOT_FIXED.as_ref() else {
            println!("WV-05 Linux ROOT_FIXED not initialized");
            return;
        };

        let (x, y, width, height) = initial_rect
            .map(|rect| rect_to_i32_bounds(rect, scale))
            .unwrap_or((0, 0, 800, 600));

        println!(
            "WV-05 Linux initial child surface bounds x={} y={} w={} h={} scale={}",
            x, y, width, height, scale
        );

        let child_fixed = gtk::Fixed::new();
        child_fixed.set_size_request(width, height);

        root_fixed.put(&child_fixed, x, y);
        child_fixed.show_all();
        root_fixed.show_all();

        log_main_context_state("before build_gtk");

        let bounds = Rect {
            position: LogicalPosition::new(0, 0).into(),
            size: LogicalSize::new(width as u32, height as u32).into(),
        };

        println!("WV-05 Linux build_gtk begin");

        let result = WebViewBuilder::new()
            .with_bounds(bounds)
            .with_url("https://example.com")
            .build_gtk(&child_fixed);

        println!("WV-05 Linux build_gtk returned");

        match result {
            Ok(webview) => {
                CHILD_FIXED = Some(child_fixed);
                WEBVIEW = Some(webview);
                WEBVIEW_CREATED = true;

                println!("WV-05 Linux WebView create success");
                log_main_context_state("after WebView create success");
            }
            Err(error) => {
                println!("WV-05 Linux WebView create failed = {:?}", error);
                log_main_context_state("after WebView create failed");
            }
        }

        flush_gtk_events_bounded("after WebView initialize");
        println!("WV-05 Linux ensure_webview_initialized end");
    }
}

/// Linux向け Child Surface 追従処理。
///
/// 役割:
/// - eframe側の update() から呼ばれることで eframe alive を確認する。
/// - WebView用 Child Surface の表示、移動、サイズ変更を行う。
/// - GTK / WebKitGTK 側イベントを低頻度・上限付きで処理する。
///
/// 引数:
/// - _ctx: egui コンテキスト。
/// - webview_rect: WebView配置矩形。
/// - should_show_native_surface: Native Surface を表示する場合 true。
///
/// 戻り値:
/// - なし。
pub fn sync_child_window(
    _ctx: &egui::Context,
    webview_rect: Option<egui::Rect>,
    should_show_native_surface: bool,
) {
    unsafe {
        log_eframe_alive();

        let Some(root_fixed) = ROOT_FIXED.as_ref() else {
            flush_gtk_events_throttled("sync_child_window no root");
            return;
        };

        let Some(child_fixed) = CHILD_FIXED.as_ref() else {
            flush_gtk_events_throttled("sync_child_window no child");
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
            flush_gtk_events_throttled("sync_child_window unchanged");
            return;
        }

        LAST_SURFACE_STATE = Some(new_state);

        if !new_state.visible {
            child_fixed.hide();
            println!("WV-05 Linux hide child surface");
            flush_gtk_events_throttled("sync_child_window hidden");
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
            "WV-05 Linux sync child surface x={} y={} w={} h={}",
            new_state.x,
            new_state.y,
            new_state.width,
            new_state.height
        );

        root_fixed.show_all();

        flush_gtk_events_throttled("sync_child_window changed");
    }
}

/// egui矩形を GTK / wry 用 i32 境界値へ変換する。
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
/// - WebView生成直後など、明示的に GTK イベントを処理する。
/// - pending が残っていても上限回数で打ち切る。
///
/// 引数:
/// - label: ログ識別名。
///
/// 戻り値:
/// - なし。
fn flush_gtk_events_bounded(label: &str) {
    let mut count = 0;

    while gtk::events_pending() && count < GTK_FLUSH_MAX_ITERATIONS {
        gtk::main_iteration_do(false);
        count += 1;
    }

    let remaining = gtk::events_pending();

    println!(
        "WV-05 Linux flush_gtk_events_bounded label={} processed={} remaining={}",
        label,
        count,
        remaining
    );
}

/// GTKイベントを低頻度で処理する。
///
/// 役割:
/// - eframe / winit を停止させずに GTK / WebKitGTK のイベントを継続処理する。
/// - GTK Host Window の応答停止を防げるか確認する。
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

/// GTK MainContext の所有状態をログ出力する。
///
/// 引数:
/// - label: ログ識別名。
///
/// 戻り値:
/// - なし。
fn log_main_context_state(label: &str) {
    let context = glib::MainContext::default();
    let is_owner = context.is_owner();

    println!(
        "WV-05 Linux MainContext label={} is_owner={}",
        label,
        is_owner
    );
}

/// GTK timer ロガーを登録する。
///
/// 役割:
/// - GTK MainContext が継続して処理されているか確認する。
///
/// 注意:
/// - idle_add_local() は使用しない。
/// - timeout_add_local() により、GTK側イベント処理が進む場合だけ timer alive が出る。
///
/// 戻り値:
/// - なし。
fn install_gtk_timer_logger() {
    unsafe {
        if GTK_TIMER_LOG_INSTALLED {
            return;
        }

        GTK_TIMER_LOG_INSTALLED = true;
    }

    glib::timeout_add_local(Duration::from_secs(1), || {
        println!("WV-05 Linux GTK timer alive");
        ControlFlow::Continue
    });

    println!("WV-05 Linux GTK timer logger installed");
}

/// eframe 側 update 継続状態をログ出力する。
///
/// 役割:
/// - sync_child_window() が継続して呼ばれているか確認する。
///
/// 戻り値:
/// - なし。
fn log_eframe_alive() {
    unsafe {
        let now = Instant::now();

        let should_log = LAST_EFRAME_ALIVE_LOG_AT
            .map(|last| now.duration_since(last) >= Duration::from_secs(1))
            .unwrap_or(true);

        if should_log {
            LAST_EFRAME_ALIVE_LOG_AT = Some(now);
            println!("WV-05 Linux eframe alive");
        }
    }
}
