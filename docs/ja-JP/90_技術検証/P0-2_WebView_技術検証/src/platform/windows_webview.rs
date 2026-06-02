windows_webview.rs 修正1

削除

if !WEBVIEW_VISIBLE {
    let _ = ShowWindow(hwnd, SW_HIDE);
    return;
}
windows_webview.rs 修正2

現在

let _ = let _ = ShowWindow(hwnd, SW_SHOW);

↓

差し替え

let _ = ShowWindow(hwnd, SW_SHOW);
windows_webview.rs 修正3

現在

if !should_show_native_surface {
    let _ = ShowWindow(hwnd, SW_HIDE);
    return;
}

↓

差し替え

if !should_show_native_surface {
    return;
}
sync_child_window 完成形
pub fn sync_child_window(
    ctx: &egui::Context,
    webview_rect: Option<egui::Rect>,
    should_show_native_surface: bool,
) {
    unsafe {
        if let Some(hwnd) = CHILD_HWND {

            if !should_show_native_surface {
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

この修正で消えるエラー

cannot find value WEBVIEW_VISIBLE
expected expression, found let statement

さらに警告対策。

削除

use crate::platform;

（panel_tab.rs）

現在 mark_webview_visible() を削除済みなので不要です。

この状態ならコンパイルは通るはずです。