//! P0-2 WebView 技術検証
//!
//! 目的:
//! - egui_dock による Dock Panel 矩形取得確認
//! - Dock 移動時の矩形変化確認
//! - Dock リサイズ時の矩形変化確認
//! - WebView Support Panel 方式の前提確認
//!
//! 注意:
//! - WV-00 の PoC 実装
//! - この段階では wry は未導入
//! - WebView は Placeholder
//! - PoC-1f では Child Window を Dock 矩形へ追従させる

use std::fs;
use std::path::Path;

use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
use windows::core::PCWSTR;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetForegroundWindow, GetParent, IsWindow, MoveWindow,
    RegisterClassW, SetParent, ShowWindow, WNDCLASSW, CW_USEDEFAULT, SW_SHOW, SW_HIDE,
    EnumChildWindows, GetClientRect, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_VISIBLE,
    WS_OVERLAPPEDWINDOW,
};

#[cfg(target_os = "windows")]
use wry::WebViewBuilder;

#[cfg(target_os = "windows")]
use wry::raw_window_handle::{
    HasWindowHandle,
    RawWindowHandle,
    WindowHandle,
    Win32WindowHandle,
};

use std::num::NonZeroIsize;

const LAYOUT_FILE_PATH: &str = "dock_layout.json";

#[cfg(target_os = "windows")]
static mut CHILD_HWND: Option<HWND> = None;

#[cfg(target_os = "windows")]
static mut WEBVIEW_CREATED: bool = false;

#[cfg(target_os = "windows")]
static mut WEBVIEW: Option<wry::WebView> = None;

#[cfg(target_os = "windows")]
static mut WEBVIEW_VISIBLE: bool = false;

#[derive(Clone, Serialize, Deserialize)]
enum PanelTab {
    Status,
    Viewport,
    Log,
    WebViewPlaceholder,
}

#[cfg(target_os = "windows")]
struct ChildWindowHandle {
    hwnd: HWND,
}

struct ValidationTabViewer<'a> {
    webview_rect: &'a mut Option<egui::Rect>,
}

#[cfg(target_os = "windows")]
impl HasWindowHandle for ChildWindowHandle {
    fn window_handle(
        &self,
    ) -> Result<WindowHandle<'_>,
        wry::raw_window_handle::HandleError
    > {
        let hwnd =
            NonZeroIsize::new(self.hwnd.0 as isize)
                .ok_or(
                    wry::raw_window_handle::HandleError::Unavailable
                )?;

        let mut handle =
            Win32WindowHandle::new(hwnd);

        unsafe {
            Ok(
                WindowHandle::borrow_raw(
                    RawWindowHandle::Win32(handle)
                )
            )
        }
    }
}

impl<'a> TabViewer for ValidationTabViewer<'a> {
    type Tab = PanelTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            PanelTab::Status => "Status".into(),
            PanelTab::Viewport => "Viewport".into(),
            PanelTab::Log => "Log".into(),
            PanelTab::WebViewPlaceholder => "WebView".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            PanelTab::Status => {
                ui.heading("Workflow IDE Framework");
                ui.separator();
                ui.label("Docking Validation");
            }
            PanelTab::Viewport => {
                ui.heading("GPU Viewport Placeholder");
                ui.label("GPU Viewport 予定領域");
            }
            PanelTab::Log => {
                ui.heading("Log Panel");
                ui.label("Runtime / Event Log 予定領域");
            }
            PanelTab::WebViewPlaceholder => {

                #[cfg(target_os = "windows")]
                unsafe {
                    WEBVIEW_VISIBLE = true;
                }

                ui.heading("WebView Placeholder");

                let rect = ui.max_rect();
                *self.webview_rect = Some(rect);

                ui.separator();
                ui.label(format!("x={:.1} y={:.1}", rect.min.x, rect.min.y));
                ui.label(format!("width={:.1} height={:.1}", rect.width(), rect.height()));
                ui.separator();
                ui.label("WV-00: Dock移動・Dockリサイズ時の矩形変化確認");
                ui.separator();
                ui.label("WV-00-01: Panel Rect取得 成功");
                ui.label("WV-00-02: Dock移動検知 成功");
                ui.label("WV-00-03: Dockリサイズ検知 成功");
                ui.label("PoC-1f: Child Window Dock追従確認");
            }
        }
    }
}

struct DockingValidationApp {
    dock_state: DockState<PanelTab>,
    webview_rect: Option<egui::Rect>,
    last_webview_rect: Option<egui::Rect>,
}

#[cfg(target_os = "windows")]
fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(target_os = "windows")]
extern "system" fn poc_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn resize_child_proc(
    child_hwnd: HWND,
    _lparam: LPARAM,
) -> windows::core::BOOL {
    println!(
        "PoC-3c enum child = {:?}",
        child_hwnd
    );

    if let Ok(parent_hwnd) = GetParent(child_hwnd) {
        let mut rect = RECT::default();

        if GetClientRect(parent_hwnd, &mut rect).is_ok() {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            println!(
                "PoC-3c resize child={:?} parent={:?} w={} h={}",
                child_hwnd,
                parent_hwnd,
                width,
                height
            );

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

impl DockingValidationApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        println!("PoC-1d start");
        println!("egui viewport id = {:?}", egui::ViewportId::ROOT);
        println!("pixels_per_point = {}", cc.egui_ctx.pixels_per_point());

        let dock_state = Self::load_layout().unwrap_or_else(Self::create_default_layout);

        Self {
            dock_state,
            webview_rect: None,
            last_webview_rect: None,
        }
    }

    fn create_default_layout() -> DockState<PanelTab> {
        let mut dock_state = DockState::new(vec![PanelTab::Status]);

        let surface = dock_state.main_surface_mut();

        let [_, right] = surface.split_right(
            NodeIndex::root(),
            0.7,
            vec![PanelTab::Viewport],
        );

        let [_, bottom] = surface.split_below(
            right,
            0.8,
            vec![PanelTab::Log],
        );

        surface.split_right(
            bottom,
            0.5,
            vec![PanelTab::WebViewPlaceholder],
        );

        dock_state
    }

    fn save_layout(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.dock_state) {
            let _ = fs::write(LAYOUT_FILE_PATH, json);
        }
    }

    fn load_layout() -> Option<DockState<PanelTab>> {
        if !Path::new(LAYOUT_FILE_PATH).exists() {
            return None;
        }

        let json = fs::read_to_string(LAYOUT_FILE_PATH).ok()?;
        serde_json::from_str(&json).ok()
    }
}

impl Drop for DockingValidationApp {
    fn drop(&mut self) {
        self.save_layout();
    }
}

impl eframe::App for DockingValidationApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let _ = frame;
        let screen_rect = ctx.input(|i| i.content_rect());

        #[cfg(target_os = "windows")]
        unsafe {
            WEBVIEW_VISIBLE = false;
        }

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

            #[cfg(target_os = "windows")]
            {
                if ui.button("PoC-1e GetForegroundWindow").clicked() {
                    unsafe {
                        let hwnd = GetForegroundWindow();
                        println!("PoC-1e foreground hwnd = {:?}", hwnd);
                    }
                }

                if ui.button("PoC-1e CreateWindowEx").clicked() {
                    unsafe {
                        let class_name = to_wide("PoC1EWindowClass");
                        let window_title = to_wide("PoC-1e Child Test Window");

                        let wc = WNDCLASSW {
                            lpfnWndProc: Some(poc_window_proc),
                            lpszClassName: PCWSTR(class_name.as_ptr()),
                            ..Default::default()
                        };

                        let atom = RegisterClassW(&wc);
                        println!("PoC-1e RegisterClassW = {}", atom);

                        let parent_hwnd = GetForegroundWindow();
                        println!("PoC-1e Parent HWND = {:?}", parent_hwnd);

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
                                println!("PoC-1e Parent == Child ? {}", parent_hwnd == hwnd);
                                println!("PoC-1e CreateWindowExW = {:?}", hwnd);

                                let _ = ShowWindow(hwnd, SW_SHOW);

                                let old_parent = SetParent(hwnd, Some(parent_hwnd));
                                println!("PoC-1e Old Parent = {:?}", old_parent);

                                let current_parent = GetParent(hwnd);
                                println!("PoC-1e Current Parent = {:?}", current_parent);

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

                if ui.button("WV-01 Create WebView Window").clicked() {
                    
                    unsafe {

                        if CHILD_HWND.is_none() {
                            println!("PoC-3a CHILD_HWND none");
                            return;
                        }

                        let hwnd = CHILD_HWND.unwrap();

                        let child_window =
                            ChildWindowHandle {
                                hwnd,
                            };

                        println!(
                            "PoC-3a CHILD_HWND = {:?}",
                            hwnd
                        );

                        if WEBVIEW_CREATED {

                            println!("WV-01 already created");

                        } else {

                            println!("WV-01 create start");

                            let result =
                                WebViewBuilder::new()
                                    .with_url("https://example.com")
                                    .build_as_child(&child_window);

                            match result {

                                Ok(webview) => {

                                    WEBVIEW = Some(webview);

                                    WEBVIEW_CREATED = true;

                                    println!("WV-01 WebView create success");

                                    println!(
                                        "WV-01 webview ptr = {:p}",
                                        WEBVIEW.as_ref().unwrap()
                                    );
                                }

                                Err(error) => {

                                    println!(
                                        "WV-01 WebView create failed = {:?}",
                                        error
                                    );
                                }
                            }
                        }
                    }
                }

                if ui.button("Hide Child Window").clicked() {
                    unsafe {
                        if let Some(hwnd) = CHILD_HWND {
                            let _ = ShowWindow(hwnd, SW_HIDE);
                        }
                    }
                }

                if ui.button("Show Child Window").clicked() {
                    unsafe {
                        if let Some(hwnd) = CHILD_HWND {
                            let _ = ShowWindow(hwnd, SW_SHOW);
                        }
                    }
                }

                if ui.button("PoC-2f WebView Status").clicked() {
                    unsafe {
                        println!(
                            "WEBVIEW_CREATED = {}",
                            WEBVIEW_CREATED
                        );

                        println!(
                            "WEBVIEW exists = {}",
                            WEBVIEW.is_some()
                        );
                    }
                }

                if ui.button("PoC-2f Dump WebView").clicked() {
                    unsafe {
                        if let Some(webview) = WEBVIEW.as_ref() {

                            println!(
                                "PoC-2f type = {}",
                                std::any::type_name_of_val(webview)
                            );

                            // ↓ここで補完確認用
                            let _ = webview.url();
                        }
                    }
                }
            }

            #[cfg(not(target_os = "windows"))]
            {
                ui.label("PoC-1e Windows Only");
            }
        });

        egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
            if ui.button("Save Layout").clicked() {
                self.save_layout();
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
                Some(old) => {
                    old.min != rect.min
                        || old.max != rect.max
                }
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

        #[cfg(target_os = "windows")]
        unsafe {
            if let Some(hwnd) = CHILD_HWND {

                if !WEBVIEW_VISIBLE {
                    let _ = ShowWindow(hwnd, SW_HIDE);
                    return;
                }

                let _ = ShowWindow(hwnd, SW_SHOW);

                if let Some(rect) = self.webview_rect {
                    let scale = ctx.pixels_per_point();

                    let x = (rect.min.x * scale) as i32;
                    let y = (rect.min.y * scale) as i32;
                    let width = (rect.width() * scale) as i32;
                    let height = (rect.height() * scale) as i32;

                    let _ = MoveWindow(
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
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "P0-2 WebView Validation",
        options,
        Box::new(|cc| Ok(Box::new(DockingValidationApp::new(cc)))),
    )
}