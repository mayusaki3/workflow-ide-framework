//! WV-04-04
//! tao + GTK Fixed 二重構成 + wry(build_gtk) 検証。
//!
//! 役割:
//! - Wayland 上で GTK Fixed を Child Window 相当として扱えるか確認する。
//! - root_fixed 配下に child_fixed を配置し、child_fixed 配下に WebView を生成する。
//! - root_fixed.move_() と child_fixed.set_size_request() により、WebView を含む
//!   Child Surface 相当領域が移動・リサイズできるか確認する。
//!
//! 注意点:
//! - 本ファイルは P0-2 WebView 技術検証用 PoC。
//! - egui / egui_dock 統合前の Linux WebView 方式確認を目的とする。
//! - WV-04-03 で WebView::set_bounds() の後続反映が確認できなかったため、
//!   本検証では GTK Container 側の移動・リサイズを確認する。

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

/// WebView を含む Child Surface 相当領域の制御イベント。
enum WebViewCommand {
    /// child_fixed を右下へ移動し、サイズを縮小する。
    MoveAndResize,
    /// child_fixed を左上へ戻し、Window全体へ広げる。
    Restore,
}

fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("WV-04-04 tao + GTK Fixed child surface")
        .build(&event_loop)
        .expect("failed to create window");

    let window_size = window
        .inner_size()
        .to_logical::<u32>(window.scale_factor());

    let (root_fixed, child_fixed) = {
        use gtk::prelude::*;

        let vbox = window
            .default_vbox()
            .expect("default_vbox not available");

        let root_fixed = gtk::Fixed::new();
        let child_fixed = gtk::Fixed::new();

        root_fixed.set_size_request(
            window_size.width as i32,
            window_size.height as i32,
        );

        child_fixed.set_size_request(
            window_size.width as i32,
            window_size.height as i32,
        );

        vbox.pack_start(&root_fixed, true, true, 0);
        root_fixed.put(&child_fixed, 0, 0);

        root_fixed.show_all();
        child_fixed.show_all();

        (root_fixed, child_fixed)
    };

    let initial_bounds = Rect {
        position: LogicalPosition::new(0, 0).into(),
        size: LogicalSize::new(window_size.width, window_size.height).into(),
    };

    let _webview = WebViewBuilder::new()
        .with_bounds(initial_bounds)
        .with_url("https://tauri.app")
        .build_gtk(&child_fixed)?;

    println!("WV-04-04: build_gtk succeeded");
    println!(
        "WV-04-04: initial child surface bounds 0,0 {}x{}",
        window_size.width, window_size.height
    );

    let (tx, rx) = mpsc::channel::<WebViewCommand>();

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(3));
        let _ = tx.send(WebViewCommand::MoveAndResize);

        thread::sleep(Duration::from_secs(3));
        let _ = tx.send(WebViewCommand::Restore);
    });

    event_loop.run(move |event, _, control_flow| {
        use gtk::prelude::*;

        *control_flow = ControlFlow::Poll;

        while let Ok(command) = rx.try_recv() {
            match command {
                WebViewCommand::MoveAndResize => {
                    println!("WV-04-04: move child_fixed to 200,100 400x300");

                    root_fixed.move_(&child_fixed, 200, 100);
                    child_fixed.set_size_request(400, 300);
                    root_fixed.show_all();
                    child_fixed.show_all();
                }

                WebViewCommand::Restore => {
                    let size = window
                        .inner_size()
                        .to_logical::<u32>(window.scale_factor());

                    println!(
                        "WV-04-04: restore child_fixed to 0,0 {}x{}",
                        size.width, size.height
                    );

                    root_fixed.move_(&child_fixed, 0, 0);
                    child_fixed.set_size_request(
                        size.width as i32,
                        size.height as i32,
                    );
                    root_fixed.set_size_request(
                        size.width as i32,
                        size.height as i32,
                    );

                    root_fixed.show_all();
                    child_fixed.show_all();
                }
            }
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let size = size.to_logical::<u32>(window.scale_factor());

                root_fixed.set_size_request(
                    size.width as i32,
                    size.height as i32,
                );

                println!("WV-04-04: resized {}x{}", size.width, size.height);
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
