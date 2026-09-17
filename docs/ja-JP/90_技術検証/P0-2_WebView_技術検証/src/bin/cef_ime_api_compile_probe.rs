//! CEF IME API の Rust バインディング形状をコンパイルで確認する Probe。
//!
//! 役割:
//! - `cef 151.8.1` の `Range` と `ime_set_composition` の実シグネチャを確認する。
//! - WV-11-04-02 の実装変更前に、推測した API 形状を本体 Probe へ持ち込まない。
//!
//! 注意点:
//! - 実行時検証は行わない。`cargo check --bin cef_ime_api_compile_probe` 用。
//! - 正式 Surface API ではない。

use cef::*;

/// CEF BrowserHost へ selection range 付き Composition を送る形を型検査する。
///
/// # 引数
/// - `browser`: 対象 CEF Browser。
/// - `text_value`: 未確定文字列。
///
/// # 戻り値
/// - なし。
///
/// @hldocs.ref doc-20260912-104801Z-WV16#sec_b6m1r9p3x7da
#[allow(dead_code)]
fn compile_ime_set_composition(browser: &Browser, text_value: &str) {
    let Some(host) = browser.host() else {
        return;
    };

    let text = CefString::from(text_value);
    let utf16_len = text_value.encode_utf16().count() as u32;
    let selection = Range {
        from: utf16_len,
        to: utf16_len,
    };

    host.ime_set_composition(Some(&text), None, None, Some(&selection));
}

/// CEF RenderHandler の IME range callback の Rust バインディング形状を型検査する。
#[derive(Clone)]
struct ImeApiRenderHandler;

wrap_render_handler! {
    struct ImeApiRenderHandlerBuilder {
        handler: ImeApiRenderHandler,
    }

    impl RenderHandler {
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            if let Some(rect) = rect {
                rect.width = 1;
                rect.height = 1;
            }
        }

        fn on_ime_composition_range_changed(
            &self,
            _browser: Option<&mut Browser>,
            selected_range: Option<&Range>,
            character_bounds: Option<&[Rect]>,
        ) {
            let _ = selected_range;
            let _ = character_bounds;
        }
    }
}

fn main() {
    println!("CEF IME API compile probe");
}
