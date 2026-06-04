//! Linux向け WebView / GTK Fixed PoC処理。
//!
//! WV-06-05
//!
//! 役割:
//! - WV-04で選定した build_gtk() + gtk::Fixed 方式を維持する。
//! - GTK Host Window を gtk::WindowType::Popup として生成し、GNOME / Mutter の「応答なし」判定を回避できるか確認する。
//! - GTKイベント処理を維持したまま、Host Window の Window Manager 管理対象性を切り分ける。
//!
//! 注意:
//! - 技術検証用コード。
//! - build_gtk(), move_(), set_size_request() の検証済み経路を維持する。
//! - GTKイベント処理は上限付き、かつスロットリング付きで実行する。

use eframe::{egui, CreationContext};
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
static mut LAST_SURFACE_STATE: Option<SurfaceState> = None;
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
/// - GTK Host Window を Popup Window として生成する。
///
/// 注意:
/// - WV-06-05では Host Window を表示するが、通常の Toplevel Window ではなく Popup Window を使用する。
///
/// 引数:
/// - _cc: eframe生成コンテキスト。
///
/// 戻り値:
/// - なし。
pub fn initialize_root_window(_cc: &CreationContext<'_>) {
    unsafe {
        if GTK_WINDOW.is_some() {
            return;
        }

        if let Err(error) = gtk::init() {
            println!("WV-06-05 Linux gtk::init failed = {:?}", error);
            return;
        }

        let window = gtk::Window::new(gtk::WindowType::Popup);
        window.set_title("WV-06-05 Linux GTK WebView Host Popup");
        window.set_default_size(800, 600);

        let root_fixed = gtk::Fixed::new();
        root_fixed.set_size_request(800, 600);

        window.add(&root_fixed);
        window.show_all();

        GTK_WINDOW = Some(window);
        ROOT_FIXED = Some(root_fixed);

        println!("WV-06-05 Linux GTK popup root window initialized");
    }
}

/// Linux向け WebView を初期化する。
///
/// 役割:
/// - gtk::Fixed 上に Child Surface 用 gtk::Fixed を配置する。
/// - wry build_gtk() で WebView を生成する。
///
/// 注意:
/// - WebView生成済みの場合は再生成しない。
/// - WV-06-00では build_gtk() の成立経路を変更しない。
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

        let Some(root_fixed) = ROOT_FIXED.as_ref() else {
            println!("WV-06-05 Linux ROOT_FIXED not initialized");
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

                println!("WV-06-05 Linux WebView create success");
            }
            Err(error) => {
                println!("WV-06-05 Linux WebView create failed = {:?}", error);
            }
        }

        flush_gtk_events_bounded("after WebView initialize");
    }
}

/// Linux向け Child Surface 追従処理。
///
/// 役割:
/// - WebView用 Child Surface の表示、移動、サイズ変更を行う。
/// - 状態変化がない場合は GTK Widget 操作を抑制する。
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

        root_fixed.show_all();

        flush_gtk_events_throttled("sync_child_window changed");
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

    if count > 0 || remaining {
        println!(
            "WV-06-05 Linux flush_gtk_events_bounded label={} processed={} remaining={}",
            label,
            count,
            remaining
        );
    }
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
