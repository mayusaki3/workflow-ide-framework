//! P0-2 WebView 技術検証アプリ本体。
//!
//! 役割:
//! - eframe::App を実装する。
//! - Dock UI、デバッグUI、WebView Placeholder 矩形更新を統合する。
//!
//! 注意:
//! - P0-2 WebView 技術検証用のPoCコード。
//! - Platform固有処理は `platform` モジュールへ分離している。
//! - WV-03以降の検証結果により、責務分割は変更される可能性がある。

use eframe::egui;
use egui_dock::{DockArea, DockState};

use crate::layout_storage;
use crate::panel_tab::{PanelTab, ValidationTabViewer};
use crate::platform;

use raw_window_handle::{
    HasWindowHandle,
    RawWindowHandle,
};

/// P0-2 WebView 技術検証アプリ。
pub struct DockingValidationApp {
    dock_state: DockState<PanelTab>,
    webview_rect: Option<egui::Rect>,
    last_webview_rect: Option<egui::Rect>,
    debug_show_native_surface: bool,
}

impl DockingValidationApp {
    /// アプリケーション状態を初期化する。
    ///
    /// # 引数
    ///
    /// * `cc` - eframe の作成コンテキスト。
    ///
    /// # 戻り値
    ///
    /// 初期化済みの `DockingValidationApp`。
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Ok(window_handle) = cc.window_handle() {
            match window_handle.as_raw() {
                RawWindowHandle::Win32(handle) => {
                    let hwnd = windows::Win32::Foundation::HWND(
                        handle.hwnd.get() as *mut core::ffi::c_void
                    );

                    platform::set_root_hwnd(hwnd);
                }
                _ => {
                    println!("WV-02 non Win32 window handle");
                }
            }
        }

        println!("PoC-1d start");
        println!("egui viewport id = {:?}", egui::ViewportId::ROOT);
        println!("pixels_per_point = {}", cc.egui_ctx.pixels_per_point());

        let dock_state =
            layout_storage::load_layout().unwrap_or_else(layout_storage::create_default_layout);

        Self {
            dock_state,
            webview_rect: None,
            last_webview_rect: None,
            debug_show_native_surface: true,
        }
    }
}

impl Drop for DockingValidationApp {
    fn drop(&mut self) {
        layout_storage::save_layout(&self.dock_state);
    }
}

impl eframe::App for DockingValidationApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let _ = frame;
        let screen_rect = ctx.input(|i| i.content_rect());

        egui::TopBottomPanel::top("debug_panel").show(ctx, |ui| {
            ui.label(format!(
                "ContentRect: x={} y={} w={} h={}",
                screen_rect.min.x,
                screen_rect.min.y,
                screen_rect.width(),
                screen_rect.height()
            ));

            ui.label(format!("PixelsPerPoint={:.2}", ctx.pixels_per_point()));
            ui.label(format!("ViewportRect: {:?}", ctx.viewport_rect()));

            ui.separator();
            ui.label("PoC-1d");
            ui.label("Native Window Investigation");
            ui.label(format!("ViewportId = {:?}", egui::ViewportId::ROOT));

            ui.checkbox(
                &mut self.debug_show_native_surface,
                "Debug: Show Native Surface",
            );
        });

        egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
            if ui.button("Save Layout").clicked() {
                layout_storage::save_layout(&self.dock_state);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = ValidationTabViewer {
                webview_rect: &mut self.webview_rect,
            };

            DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);
        });

        if let Some(rect) = self.webview_rect {
            let changed = match self.last_webview_rect {
                Some(old) => old.min != rect.min || old.max != rect.max,
                None => true,
            };

            if changed {
                println!(
                    "PoC-2e DockRect min=({:.1},{:.1}) max=({:.1},{:.1})",
                    rect.min.x,
                    rect.min.y,
                    rect.max.x,
                    rect.max.y
                );

                self.last_webview_rect = Some(rect);
            }
        }

        if let Some(rect) = self.webview_rect {
            if rect.width() > 10.0 && rect.height() > 10.0 {
                platform::ensure_webview_initialized();
            }
        }

        let should_show_native_surface =
            self.debug_show_native_surface;

        if let Some(rect) = self.webview_rect {
            if rect.width() > 10.0 && rect.height() > 10.0 {
                platform::ensure_webview_initialized();
            }
        }

        platform::sync_child_window(
            ctx,
            self.webview_rect,
            should_show_native_surface,
        );
    }
}
