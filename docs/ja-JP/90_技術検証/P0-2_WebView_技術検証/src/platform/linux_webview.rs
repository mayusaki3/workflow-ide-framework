//! Linux向け WebView / GTK Fixed PoC処理。
//!
//! 役割:
//! - Windows版 `windows_webview.rs` と同じ公開I/Fで Linux 側の実装可否を検証する。
//! - Linux(Wayland/X11)では `build_as_child()` を使用せず、`build_gtk()` を使用する。
//! - GTK Window / root_fixed / child_fixed / WebView の構成で、Child Window相当の
//!   表示・非表示・移動・リサイズを検証する。
//!
//! 注意:
//! - P0-2 WebView 技術検証用のPoCコード。
//! - 現行 eframe Root Window から GTK Container を取得する経路は確認できていない。
//! - そのため本ファイルでは、Linux側の同一I/F実装可能性を確認するため、
//!   GTK側に検証用Windowを生成して `build_gtk()` を検証する。
//! - 本番仕様化時には、本検証結果をもとに Platform 仕様・設計・テストへ戻す。

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

/// Linux側の Root Window 相当を初期化する。
///
/// # 役割
///
/// - Windows版の `initialize_root_window()` と同じ呼び出し口を維持する。
/// - Linux版では GTK を初期化し、検証用 GTK Window と root_fixed を生成する。
///
/// # 注意点
///
/// - eframe の CreationContext は、現時点では GTK Container 取得に使用しない。
/// - GTK初期化済みでない場合、`build_gtk()` が panic するため、ここで `gtk::init()` を行う。
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

/// WebView が未生成であれば生成する。
///
/// # 引数
///
/// * `initial_rect` - WebView Panel の初期矩形。
/// * `scale` - egui の pixels_per_point。
///
/// # 役割
///
/// - Windows版の `ensure_webview_initialized()` と同じ呼び出し口を維持する。
/// - Linux版では root_fixed 配下に child_fixed を作成し、その配下へ `build_gtk()` で
///   WebView を生成する。
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

/// Child Surface 相当の表示位置・サイズ・表示状態を同期する。
///
/// WV-04-05
///
/// GTK Fixed の move_() と set_size_request() が
/// 実際に WebView へ反映されるかを確認するため、
/// egui座標を無視して強制的に
///
/// x=0
/// y=0
/// w=200
/// h=100
///
/// へ移動する。
pub fn sync_child_window(
    _ctx: &egui::Context,
    _webview_rect: Option<egui::Rect>,
    should_show_native_surface: bool,
) {
    unsafe {
        let Some(root_fixed) = ROOT_FIXED.as_ref() else {
            return;
        };

        let Some(child_fixed) = CHILD_FIXED.as_ref() else {
            return;
        };

        if !should_show_native_surface {
            child_fixed.hide();
            flush_gtk_events();
            return;
        }

        child_fixed.show();

        //
        // WV-04-05 強制移動テスト
        //
        root_fixed.move_(child_fixed, 0, 0);

        //
        // WV-04-05 強制サイズ変更
        //
        child_fixed.set_size_request(200, 100);

        println!(
            "WV-04-05 TEST move child surface x=0 y=0 w=200 h=100"
        );

        root_fixed.show_all();

        flush_gtk_events();
    }
}

/// egui の Rect を GTK 用の整数座標へ変換する。
///
/// # 注意点
///
/// - Wayland環境で異常に大きいサイズが GTK/GDK クラッシュを誘発したため、
///   技術検証用に最小・最大値を制限する。
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

/// GTKイベントを処理する。
///
/// # 役割
///
/// - eframe/winit 側のイベントループ内で GTK 側の表示更新を進める。
/// - 技術検証用の簡易処理。
fn flush_gtk_events() {
    while gtk::events_pending() {
        gtk::main_iteration_do(false);
    }
}
