//! WV-04-02 / WV-04-03
//! tao + GTK Fixed + wry(build_gtk) 検証。
//!
//! 役割:
//! - Wayland 上で tao + GTK Fixed + build_gtk() が成立するか確認する。
//! - WebView::set_bounds() による位置変更・サイズ変更を確認する。
//!
//! 注意点:
//! - 本ファイルは P0-2 WebView 技術検証用 PoC。
//! - egui / egui_dock 統合前の Linux WebView 方式確認を目的とする。

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::unix::WindowExtUnix,
    window::WindowBuilder,
};

use wry::{
    dpi::{LogicalPosition, LogicalSize},
    Rect, WebViewBuilder, WebViewBuilderExtUnix,
};

/// WebView制御イベント。
enum WebViewCommand {
    /// WebViewを右下へ移動し、サイズを縮小する。
    MoveAndResize,
    /// WebViewを左上へ戻し、Window全体へ広げる。
    Restore,
}

fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("WV-04-03 tao + GTK Fixed + set_bounds")
        .build(&event_loop)
        .expect("failed to create window");

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

    let initial_bounds = Rect {
        position: LogicalPosition::new(0, 0).into(),
        size: LogicalSize::new(size.width, size.height).into(),
    };

    let webview = WebViewBuilder::new()
        .with_bounds(initial_bounds)
        .with_url("https://tauri.app")
        .build_gtk(&fixed)?;

    println!("WV-04-03: build_gtk succeeded");
    println!("WV-04-03: initial bounds {}x{}", size.width, size.height);

    let (tx, rx) = mpsc::channel::<WebViewCommand>();

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(3));
        let _ = tx.send(WebViewCommand::MoveAndResize);

        thread::sleep(Duration::from_secs(3));
        let _ = tx.send(WebViewCommand::Restore);
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        while let Ok(command) = rx.try_recv() {
            match command {
                WebViewCommand::MoveAndResize => {
                    println!("WV-04-03: set_bounds move and resize");

                    let _ = webview.set_bounds(Rect {
                        position: LogicalPosition::new(200, 100).into(),
                        size: LogicalSize::new(400, 300).into(),
                    });
                }
                WebViewCommand::Restore => {
                    let size = window
                        .inner_size()
                        .to_logical::<u32>(window.scale_factor());

                    println!("WV-04-03: set_bounds restore {}x{}", size.width, size.height);

                    let _ = webview.set_bounds(Rect {
                        position: LogicalPosition::new(0, 0).into(),
                        size: LogicalSize::new(size.width, size.height).into(),
                    });
                }
            }
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let size = size.to_logical::<u32>(window.scale_factor());

                let _ = webview.set_bounds(Rect {
                    position: LogicalPosition::new(0, 0).into(),
                    size: LogicalSize::new(size.width, size.height).into(),
                });

                println!("WV-04-03: resized {}x{}", size.width, size.height);
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
