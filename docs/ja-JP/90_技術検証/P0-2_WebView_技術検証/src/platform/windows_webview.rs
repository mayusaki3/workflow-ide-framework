//! Windows向け WebView / Child Window PoC処理。
//!
//! 役割:
//! - Win32 API で Child Window を生成する。
//! - wry WebView を Child Window 上に生成する。
//! - Dock Panel 矩形に合わせて Child Window を追従させる。
//!
//! 注意:
//! - P0-2 WebView 技術検証用のPoCコード。
//! - `static mut` はPoC段階の暫定実装。
//! - 正式仕様化時は状態管理構造体または同期プリミティブへの置換を検討する。

use std::num::NonZeroIsize;

use eframe::egui;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, EnumChildWindows, GetClientRect, GetForegroundWindow,
    GetParent, IsWindow, MoveWindow, RegisterClassW, SetParent, ShowWindow, WNDCLASSW,
    CW_USEDEFAULT, SW_HIDE, SW_SHOW, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_VISIBLE,
};
use wry::raw_window_handle::{
    HasWindowHandle, RawWindowHandle, Win32WindowHandle, WindowHandle,
};
use wry::WebViewBuilder;

static mut CHILD_HWND: Option<HWND> = None;
static mut WEBVIEW_CREATED: bool = false;
static mut WEBVIEW: Option<wry::WebView> = None;
static mut WEBVIEW_VISIBLE: bool = false;

/// wry の `build_as_child()` に渡すための Win32 Window Handle ラッパー。
struct ChildWindowHandle {
    hwnd: HWND,
}

impl HasWindowHandle for ChildWindowHandle {
    fn window_handle(
        &self,
    ) -> Result<WindowHandle<'_>, wry::raw_window_handle::HandleError> {
        let hwnd = NonZeroIsize::new(self.hwnd.0 as isize)
            .ok_or(wry::raw_window_handle::HandleError::Unavailable)?;

        let handle = Win32WindowHandle::new(hwnd);

        unsafe {
            Ok(WindowHandle::borrow_raw(
                RawWindowHandle::Win32(handle),
            ))
        }
    }
}

/// WebView Placeholder が描画されたことを記録する。
pub fn mark_webview_visible() {
    unsafe {
        WEBVIEW_VISIBLE = true;
    }
}

/// フレーム先頭で WebView 表示状態を初期化する。
pub fn reset_webview_visible() {
    unsafe {
        WEBVIEW_VISIBLE = false;
    }
}

/// Windows向けデバッグ操作UIを描画する。
///
/// # 引数
///
/// * `ui` - 描画先の egui UI。
pub fn render_debug_controls(ui: &mut egui::Ui) {
    if ui.button("PoC-1e GetForegroundWindow").clicked() {
        unsafe {
            let hwnd = GetForegroundWindow();
            println!("PoC-1e foreground hwnd = {:?}", hwnd);
        }
    }

    if ui.button("PoC-1e CreateWindowEx").clicked() {
        create_child_window();
    }

    if ui.button("WV-01 Create WebView Window").clicked() {
        create_webview_window();
    }

    if ui.button("Hide Child Window").clicked() {
        unsafe {
            if let Some(hwnd) = CHILD_HWND {
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
        }
    }

    if ui.button("Show Child Window").clicked() {
        unsafe {
            if let Some(hwnd) = CHILD_HWND {
                let _ = ShowWindow(hwnd, SW_SHOW);
            }
        }
    }

    if ui.button("PoC-2f WebView Status").clicked() {
        unsafe {
            println!("WEBVIEW_CREATED = {}", WEBVIEW_CREATED);
            println!("WEBVIEW exists = {}", WEBVIEW.is_some());
        }
    }

    if ui.button("PoC-2f Dump WebView").clicked() {
        unsafe {
            if let Some(webview) = WEBVIEW.as_ref() {
                println!("PoC-2f type = {}", std::any::type_name_of_val(webview));

                // 補完確認用。
                let _ = webview.url();
            }
        }
    }
}

/// Child Window を Dock Panel 矩形へ追従させる。
///
/// # 引数
///
/// * `ctx` - egui Context。
/// * `webview_rect` - WebView Placeholder の現在矩形。
pub fn sync_child_window(
    ctx: &egui::Context,
    webview_rect: Option<egui::Rect>,
) {
    unsafe {
        if let Some(hwnd) = CHILD_HWND {
            if !WEBVIEW_VISIBLE {
                let _ = ShowWindow(hwnd, SW_HIDE);
                return;
            }

            let _ = ShowWindow(hwnd, SW_SHOW);

            if let Some(rect) = webview_rect {
                let scale = ctx.pixels_per_point();

                let x = (rect.min.x * scale) as i32;
                let y = (rect.min.y * scale) as i32;
                let width = (rect.width() * scale) as i32;
                let height = (rect.height() * scale) as i32;

                let _ = MoveWindow(
                    hwnd,
                    x,
                    y,
                    width,
                    height,
                    true,
                );

                let _ = EnumChildWindows(
                    Some(hwnd),
                    Some(resize_child_proc),
                    LPARAM(0),
                );
            }
        }
    }
}

/// UTF-16 NUL終端文字列へ変換する。
fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// PoC用 Window Procedure。
extern "system" fn poc_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// 子Window内の WebView 子要素を親Windowサイズへ合わせる。
unsafe extern "system" fn resize_child_proc(
    child_hwnd: HWND,
    _lparam: LPARAM,
) -> windows::core::BOOL {
    println!("PoC-3c enum child = {:?}", child_hwnd);

    if let Ok(parent_hwnd) = GetParent(child_hwnd) {
        let mut rect = RECT::default();

        if GetClientRect(parent_hwnd, &mut rect).is_ok() {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            println!(
                "PoC-3c resize child={:?} parent={:?} w={} h={}",
                child_hwnd,
                parent_hwnd,
                width,
                height
            );

            let _ = MoveWindow(
                child_hwnd,
                0,
                0,
                width,
                height,
                true,
            );
        }
    }

    windows::core::BOOL(1)
}

/// Child Window を作成する。
fn create_child_window() {
    unsafe {
        let class_name = to_wide("PoC1EWindowClass");
        let window_title = to_wide("PoC-1e Child Test Window");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(poc_window_proc),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        let atom = RegisterClassW(&wc);
        println!("PoC-1e RegisterClassW = {}", atom);

        let parent_hwnd = GetForegroundWindow();
        println!("PoC-1e Parent HWND = {:?}", parent_hwnd);

        let hwnd = CreateWindowExW(
            Default::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            600,
            400,
            Some(parent_hwnd),
            None,
            None,
            None,
        );

        match hwnd {
            Ok(hwnd) => {
                println!("PoC-1e Parent == Child ? {}", parent_hwnd == hwnd);
                println!("PoC-1e CreateWindowExW = {:?}", hwnd);

                let _ = ShowWindow(hwnd, SW_SHOW);

                let old_parent = SetParent(hwnd, Some(parent_hwnd));
                println!("PoC-1e Old Parent = {:?}", old_parent);

                let current_parent = GetParent(hwnd);
                println!("PoC-1e Current Parent = {:?}", current_parent);

                println!(
                    "PoC-1e IsWindow = {}",
                    IsWindow(Some(hwnd)).as_bool()
                );

                CHILD_HWND = Some(hwnd);
            }
            Err(error) => {
                println!("PoC-1e CreateWindowExW failed = {:?}", error);
            }
        }
    }
}

/// Child Window 上へ WebView を作成する。
fn create_webview_window() {
    unsafe {
        if CHILD_HWND.is_none() {
            println!("PoC-3a CHILD_HWND none");
            return;
        }

        let hwnd = CHILD_HWND.unwrap();

        let child_window = ChildWindowHandle {
            hwnd,
        };

        println!("PoC-3a CHILD_HWND = {:?}", hwnd);

        if WEBVIEW_CREATED {
            println!("WV-01 already created");
        } else {
            println!("WV-01 create start");

            let result = WebViewBuilder::new()
                .with_url("https://example.com")
                .build_as_child(&child_window);

            match result {
                Ok(webview) => {
                    WEBVIEW = Some(webview);
                    WEBVIEW_CREATED = true;

                    println!("WV-01 WebView create success");

                    println!(
                        "WV-01 webview ptr = {:p}",
                        WEBVIEW.as_ref().unwrap()
                    );
                }
                Err(error) => {
                    println!("WV-01 WebView create failed = {:?}", error);
                }
            }
        }
    }
}
