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

use raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// egui_dock 0.18.0 の既定タブバー高さ。
///
/// egui_dock の内部状態 `State` / `DragDropState` は `pub(super)` のため、
/// 外部アプリから正確なタブドラッグ状態は参照できない。
/// 本PoCでは、各Panelのコンテンツ矩形の直上 24px をタブバー候補領域として扱い、
/// その領域でドラッグを開始した場合のみ「Dockタブドラッグ候補」とみなす。
const EGUI_DOCK_TAB_BAR_HEIGHT: f32 = 24.0;

/// P0-2 WebView 技術検証アプリ。
pub struct DockingValidationApp {
    dock_state: DockState<PanelTab>,
    webview_rect: Option<egui::Rect>,
    last_webview_rect: Option<egui::Rect>,
    active_panel_rects: Vec<egui::Rect>,
    dock_tab_drag_candidate: bool,
    webview_tab_visible: bool,
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
                        handle.hwnd.get() as *mut core::ffi::c_void,
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
            active_panel_rects: Vec::new(),
            dock_tab_drag_candidate: false,
            webview_tab_visible: false,
            debug_show_native_surface: true,
        }
    }

    /// WebView Panel のタブバー候補矩形を返す。
    ///
    /// WebView のコンテンツ矩形は `ValidationTabViewer::ui()` 内で `ui.max_rect()` として取得する。
    /// egui_dock はコンテンツ領域の直上にタブバーを描画するため、既定高さ 24px を使って
    /// WebViewタブバー候補矩形を近似する。
    fn webview_tab_bar_rect(&self) -> Option<egui::Rect> {
        self.webview_rect.map(|rect| {
            egui::Rect::from_min_max(
                egui::pos2(rect.min.x, rect.min.y - EGUI_DOCK_TAB_BAR_HEIGHT),
                egui::pos2(rect.max.x, rect.min.y),
            )
        })
    }

    /// 指定位置が、現在表示されているDock Panelのコンテンツ領域内か判定する。
    fn is_inside_any_active_panel_content(&self, pos: egui::Pos2) -> bool {
        self.active_panel_rects.iter().any(|rect| rect.contains(pos))
    }

    /// Dockタブドラッグ候補状態を更新する。
    ///
    /// egui_dock 0.18.0 は内部の正確な DragDropState を外部公開していないため、
    /// 本PoCでは「左ボタン押下開始位置がPanelコンテンツ外、またはWebViewタブバー候補付近」
    /// をDockタブドラッグ候補として扱う。
    ///
    /// これにより、WebViewコンテンツ領域から単にドラッグしてWebView上へ移動した場合の
    /// 誤Hideを避ける。
    fn update_dock_tab_drag_candidate(&mut self, ctx: &egui::Context) {
        let primary_down = ctx.input(|i| i.pointer.primary_down());

        if !primary_down {
            if self.dock_tab_drag_candidate {
                println!("WV-02 Dock tab drag candidate end");
            }
            self.dock_tab_drag_candidate = false;
            return;
        }

        let primary_pressed = ctx.input(|i| i.pointer.primary_pressed());

        if primary_pressed {
            let press_origin = ctx.input(|i| i.pointer.press_origin());

            let started_from_panel_content = press_origin
                .map(|pos| self.is_inside_any_active_panel_content(pos))
                .unwrap_or(false);

            let started_from_webview_tab_bar = match (press_origin, self.webview_tab_bar_rect()) {
                (Some(pos), Some(rect)) => rect.contains(pos),
                _ => false,
            };

            self.dock_tab_drag_candidate =
                !started_from_panel_content || started_from_webview_tab_bar;

            println!(
                "WV-02 drag start origin={:?} started_from_panel_content={} started_from_webview_tab_bar={} candidate={}",
                press_origin,
                started_from_panel_content,
                started_from_webview_tab_bar,
                self.dock_tab_drag_candidate
            );
        }
    }

    /// Native Surface を表示すべきか判定する。
    ///
    /// 非表示にするのは、Dockタブドラッグ候補であり、ポインタがWebView Panel上にあり、
    /// egui_dock がタブドラッグ時に設定する `CursorIcon::Grabbing` が出ている場合のみ。
    fn should_show_native_surface(&self, ctx: &egui::Context) -> bool {
        if !self.debug_show_native_surface {
            return false;
        }

        if !self.webview_tab_visible {
            return false;
        }

        let pointer_decidedly_dragging = ctx.input(|i| i.pointer.is_decidedly_dragging());

        let pointer_on_webview_panel = ctx.input(|i| {
            i.pointer
                .interact_pos()
                .map(|pos| {
                    self.webview_rect
                        .map(|rect| rect.contains(pos))
                        .unwrap_or(false)
                })
                .unwrap_or(false)
        });

        let cursor_icon_is_grabbing =
            ctx.output(|o| o.cursor_icon == egui::CursorIcon::Grabbing);

        let should_hide = self.dock_tab_drag_candidate
            && pointer_decidedly_dragging
            && pointer_on_webview_panel
            && cursor_icon_is_grabbing;

        if should_hide {
            println!(
                "WV-02 Hide Native Surface: dock_tab_drag_candidate={} pointer_decidedly_dragging={} pointer_on_webview_panel={} cursor_icon_is_grabbing={}",
                self.dock_tab_drag_candidate,
                pointer_decidedly_dragging,
                pointer_on_webview_panel,
                cursor_icon_is_grabbing
            );
        }

        !should_hide
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

            ui.separator();
            ui.label(format!(
                "Dock tab drag candidate = {}",
                self.dock_tab_drag_candidate
            ));

            if let Some(rect) = self.webview_tab_bar_rect() {
                ui.label(format!(
                    "WebView tab bar candidate: x={:.1} y={:.1} w={:.1} h={:.1}",
                    rect.min.x,
                    rect.min.y,
                    rect.width(),
                    rect.height()
                ));
            }
        });

        egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
            if ui.button("Save Layout").clicked() {
                layout_storage::save_layout(&self.dock_state);
            }
        });

        self.active_panel_rects.clear();
        self.webview_tab_visible = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = ValidationTabViewer {
                webview_rect: &mut self.webview_rect,
                active_panel_rects: &mut self.active_panel_rects,
                webview_tab_visible: &mut self.webview_tab_visible,
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

                if let Some(tab_rect) = self.webview_tab_bar_rect() {
                    println!(
                        "WV-02 WebViewTabBarCandidate min=({:.1},{:.1}) max=({:.1},{:.1})",
                        tab_rect.min.x,
                        tab_rect.min.y,
                        tab_rect.max.x,
                        tab_rect.max.y
                    );
                }

                self.last_webview_rect = Some(rect);
            }

            if rect.width() > 10.0 && rect.height() > 10.0 {
                platform::ensure_webview_initialized(Some(rect), ctx.pixels_per_point());
            }
        }

        self.update_dock_tab_drag_candidate(ctx);

        let should_show_native_surface = self.should_show_native_surface(ctx);

        platform::sync_child_window(ctx, self.webview_rect, should_show_native_surface);
    }
}
