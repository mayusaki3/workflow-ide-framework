//! Linux向け WebView / Child Surface PoC処理。
//!
//! 役割:
//! - Linux(Wayland/X11)環境で wry の `build_as_child()` を使用して WebView を生成する。
//! - egui_dock の WebView Panel 矩形に合わせて WebView の bounds を更新する。
//!
//! 注意:
//! - P0-2 WebView 技術検証用のPoCコード。
//! - WV-03-04 では Windows版と同じ platform API を維持し、上位層へOS差異を見せない。
//! - Linuxでは `WebView::set_bounds()` によりDock矩形追従を検証する。

use eframe::{egui, CreationContext};
use wry::dpi::{LogicalPosition, LogicalSize};
use wry::raw_window_handle::{
    HasWindowHandle, RawWindowHandle, WindowHandle,
};
use wry::{Rect, WebViewBuilder};

static mut ROOT_HANDLE: Option<RawWindowHandle> = None;
static mut WEBVIEW_CREATED: bool = false;
static mut WEBVIEW: Option<wry::WebView> = None;

/// 保存済みの RawWindowHandle を wry の親Windowとして渡すためのラッパー。
struct RootWindowHandle {
    raw_window_handle: RawWindowHandle,
}

impl HasWindowHandle for RootWindowHandle {
    fn window_handle(
        &self,
    ) -> Result<WindowHandle<'_>, wry::raw_window_handle::HandleError> {
        unsafe {
            Ok(WindowHandle::borrow_raw(self.raw_window_handle))
        }
    }
}

/// eframe の作成コンテキストから Root Window Handle を取得する。
///
/// # 役割
///
/// - App層から Linux 固有の Window Handle 取得処理を分離する。
/// - 取得した Root Window Handle を WebView の親として保持する。
///
/// # 注意点
///
/// - Linux向けPoC処理である。
/// - Wayland / X11 の差異は wry / raw-window-handle 側に委ねる。
pub fn initialize_root_window(cc: &CreationContext<'_>) {
    if let Ok(window_handle) = cc.window_handle() {
        let raw = window_handle.as_raw();

        unsafe {
            ROOT_HANDLE = Some(raw);
        }

        println!("WV-03 Linux Root Window Handle = {:?}", raw);
    } else {
        println!("WV-03 Linux Root Window Handle unavailable");
    }
}

/// WebView が未生成であれば生成する。
///
/// # 引数
///
/// * `initial_rect` - WebView Panel の初期矩形。
/// * `scale` - egui の pixels_per_point。
///
/// # 注意点
///
/// - Linux版の初期PoCでは、Child Surface相当として wry の `build_as_child()` を使用する。
/// - 初期表示時の左上フラッシュを避けるため、生成時に `with_bounds()` を設定する。
pub fn ensure_webview_initialized(
    initial_rect: Option<egui::Rect>,
    scale: f32,
) {
    unsafe {
        if WEBVIEW_CREATED {
            return;
        }

        let Some(root_handle) = ROOT_HANDLE else {
            println!("WV-03 Linux Root Window Handle not initialized");
            return;
        };

        let Some(rect) = initial_rect else {
            println!("WV-03 Linux initial rect none");
            return;
        };

        let parent = RootWindowHandle {
            raw_window_handle: root_handle,
        };

        let bounds = rect_to_wry_bounds(rect, scale);

        let result = WebViewBuilder::new()
            .with_bounds(bounds)
            .with_url("https://example.com")
            .build_as_child(&parent);

        match result {
            Ok(webview) => {
                WEBVIEW = Some(webview);
                WEBVIEW_CREATED = true;

                println!("WV-03 Linux WebView create success");
            }
            Err(error) => {
                println!("WV-03 Linux WebView create failed = {:?}", error);
            }
        }
    }
}

/// WebView の表示位置・サイズ・表示状態を同期する。
///
/// # 引数
///
/// * `ctx` - egui Context。
/// * `webview_rect` - WebView Panel の現在矩形。
/// * `should_show_native_surface` - Native Surface を表示すべきか。
///
/// # 注意点
///
/// - 初期PoCでは Hide/Show の完全制御よりも、WebView生成とbounds追従を優先する。
/// - wry 0.53.5 の Linux では `set_bounds()` による手動追従が必要である。
pub fn sync_child_window(
    ctx: &egui::Context,
    webview_rect: Option<egui::Rect>,
    should_show_native_surface: bool,
) {
    unsafe {
        let Some(webview) = WEBVIEW.as_ref() else {
            return;
        };

        if !should_show_native_surface {
            let _ = webview.set_bounds(Rect {
                position: LogicalPosition::new(0.0, 0.0).into(),
                size: LogicalSize::new(1.0, 1.0).into(),
            });
            return;
        }

        if let Some(rect) = webview_rect {
            let bounds = rect_to_wry_bounds(rect, ctx.pixels_per_point());

            if let Err(error) = webview.set_bounds(bounds) {
                println!("WV-03 Linux WebView set_bounds failed = {:?}", error);
            }
        }
    }
}

/// egui のRectを wry のRectへ変換する。
fn rect_to_wry_bounds(rect: egui::Rect, scale: f32) -> Rect {
    let x = (rect.min.x * scale) as f64;
    let y = (rect.min.y * scale) as f64;
    let width = (rect.width() * scale) as f64;
    let height = (rect.height() * scale) as f64;

    Rect {
        position: LogicalPosition::new(x, y).into(),
        size: LogicalSize::new(width, height).into(),
    }
}
