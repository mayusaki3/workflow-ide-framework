//! Windows向け WebView / Child Window PoC処理。

use std::num::NonZeroIsize;

use eframe::egui;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, EnumChildWindows, GetClientRect,
    GetParent, IsWindow, MoveWindow, RegisterClassW, SetParent, ShowWindow, WNDCLASSW,
    CW_USEDEFAULT, SW_HIDE, SW_SHOW, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_VISIBLE,
};
use wry::raw_window_handle::{
    HasWindowHandle, RawWindowHandle, Win32WindowHandle, WindowHandle,
};
use wry::WebViewBuilder;

static mut ROOT_HWND: Option<HWND> = None;
static mut CHILD_HWND: Option<HWND> = None;
static mut WEBVIEW_CREATED: bool = false;
static mut WEBVIEW: Option<wry::WebView> = None;

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

pub fn set_root_hwnd(hwnd: HWND) {
    unsafe {
        ROOT_HWND = Some(hwnd);
        println!("WV-02 Root HWND = {:?}", hwnd);
    }
}

pub fn ensure_webview_initialized() {
    unsafe {
        if CHILD_HWND.is_none() {
            create_child_window();
        }

        if !WEBVIEW_CREATED {
            create_webview();
        }
    }
}

pub fn sync_child_window(
    ctx: &egui::Context,
    webview_rect: Option<egui::Rect>,
    should_show_native_surface: bool,
) {
    unsafe {
        if let Some(hwnd) = CHILD_HWND {
            if !should_show_native_surface {
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

                let _ = ShowWindow(hwnd, SW_SHOW);

                MoveWindow(
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

fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

extern "system" fn poc_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

unsafe extern "system" fn resize_child_proc(
    child_hwnd: HWND,
    _lparam: LPARAM,
) -> windows::core::BOOL {
    if let Ok(parent_hwnd) = GetParent(child_hwnd) {
        let mut rect = RECT::default();

        if GetClientRect(parent_hwnd, &mut rect).is_ok() {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

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

fn create_child_window() {
    unsafe {
        let class_name = to_wide("PoC1EWindowClass");
        let window_title = to_wide("PoC-1e Child Test Window");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(poc_window_proc),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        let _ = RegisterClassW(&wc);

        let Some(parent_hwnd) = ROOT_HWND else {
            println!("WV-02 Root HWND not initialized");
            return;
        };

        println!("WV-02 Parent HWND = {:?}", parent_hwnd);

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
                let _ = ShowWindow(hwnd, SW_SHOW);

                let _ = SetParent(hwnd, Some(parent_hwnd));

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

fn create_webview() {
    unsafe {
        let Some(hwnd) = CHILD_HWND else {
            println!("PoC-3a CHILD_HWND none");
            return;
        };

        let child_window = ChildWindowHandle { hwnd };

        if WEBVIEW_CREATED {
            return;
        }

        let result = WebViewBuilder::new()
            .with_url("https://example.com")
            .build_as_child(&child_window);

        match result {
            Ok(webview) => {
                WEBVIEW = Some(webview);
                WEBVIEW_CREATED = true;

                println!("WV-01 WebView create success");
            }
            Err(error) => {
                println!("WV-01 WebView create failed = {:?}", error);
            }
        }
    }
}
