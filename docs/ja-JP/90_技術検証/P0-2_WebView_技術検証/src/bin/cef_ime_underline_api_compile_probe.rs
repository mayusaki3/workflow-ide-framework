//! CEF IME CompositionUnderline の Rust バインディング形状を確認する Probe。
//!
//! 役割:
//! - `cef 151.8.1` の `CompositionUnderline` の実際のフィールド型をコンパイルで確認する。
//! - CEF 公式 Windows OSR 実装に合わせた selection range + underline Probe を作る前に、
//!   推測した型を実行 Probe へ持ち込まない。
//!
//! 注意点:
//! - 実行時検証は行わない。`cargo check --bin cef_ime_underline_api_compile_probe` 用。
//! - 正式 Surface API ではない。

use cef::*;

/// Composition 全体を覆う underline と selection range を生成し、
/// `ime_set_composition` へ渡せることを型検査する。
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
fn compile_ime_set_composition_with_underline(browser: &Browser, text_value: &str) {
    let Some(host) = browser.host() else {
        return;
    };

    let text = CefString::from(text_value);
    let utf16_len = text_value.encode_utf16().count() as u32;

    let selection = Range {
        from: 0,
        to: utf16_len,
    };

    let underline = CompositionUnderline {
        range: Range {
            from: 0,
            to: utf16_len,
        },
        color: 0xFF000000,
        background_color: 0x00000000,
        thick: 0,
        ..Default::default()
    };
    let underlines = [underline];

    host.ime_set_composition(
        Some(&text),
        Some(&underlines),
        None,
        Some(&selection),
    );
}

fn main() {
    println!("CEF IME underline API compile probe");
}
