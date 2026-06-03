//! WV-04-02
//! tao + GTK Fixed + wry(build_gtk)
//!
//! 目的:
//! Linux(Wayland/X11)環境において
//! GTK Fixed を Child Window 相当として利用できるか確認する。
//!
//! 確認項目:
//! - Window生成
//! - default_vbox取得
//! - gtk::Fixed生成
//! - build_gtk成功
//! - URL表示成功
//! - set_bounds成功
//! - リサイズ追従成功

use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::unix::WindowExtUnix,
    window::WindowBuilder,
};

use wry::{
    dpi::{LogicalPosition, LogicalSize},
    Rect,
    WebViewBuilder,
    WebViewBuilderExtUnix,
};

fn main() -> wry::Result<()> {
    // EventLoop生成
    let event_loop = EventLoop::new();

    // Window生成
    let window = WindowBuilder::new()
        .with_title("WV-04-02 tao + GTK Fixed + wry")
        .build(&event_loop)
        .expect("failed to create window");

    // GTKコンテナ取得
    let fixed = {
        use gtk::prelude::*;

        let vbox = window
            .default_vbox()
            .expect("default_vbox not available");

        let fixed = gtk::Fixed::new();

        vbox.pack_start(&fixed, true, true, 0);

        fixed.show_all();

        fixed
    };

    let size = window
        .inner_size()
        .to_logical::<u32>(window.scale_factor());

    let bounds = Rect {
        position: LogicalPosition::new(0, 0).into(),
        size: LogicalSize::new(size.width, size.height).into(),
    };

    // WebView生成
    let webview = WebViewBuilder::new()
        .with_bounds(bounds)
        .with_url("https://tauri.app")
        .build_gtk(&fixed)?;

    println!("WV-04-02: build_gtk succeeded");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let size =
                    size.to_logical::<u32>(window.scale_factor());

                let _ = webview.set_bounds(Rect {
                    position: LogicalPosition::new(0, 0).into(),
                    size: LogicalSize::new(
                        size.width,
                        size.height,
                    )
                    .into(),
                });

                println!(
                    "WV-04-02: resized {}x{}",
                    size.width,
                    size.height
                );
            }

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
