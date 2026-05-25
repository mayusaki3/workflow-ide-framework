//! P0-2 WebView 技術検証
//!
//! 目的:
//! - wry による WebView 表示確認
//! - Window coexist 確認
//! - JavaScript 実行確認
//!
//! 注意:
//! - 技術検証用コード
//! - egui 統合前の最小構成

use wry::application::event::{Event, WindowEvent};
use wry::application::event_loop::{ControlFlow, EventLoop};
use wry::application::window::WindowBuilder;
use wry::webview::WebViewBuilder;

/// エントリーポイント
fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("P0-2 WebView 技術検証")
        .build(&event_loop)?;

    let _webview = WebViewBuilder::new(window)?
        .with_html(
            r#"
            <html>
            <body>
                <h1>P0-2 WebView 技術検証</h1>
                <p>wry WebView 表示確認</p>
                <button onclick=\"alert('WebView OK')\">Test</button>
            </body>
            </html>
            "#,
        )?
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
