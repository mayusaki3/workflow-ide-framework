//! WV-11-04-02 Windows Native IME Context Probe。
//!
//! 役割:
//! - eframe が作成した Windows HWND を `raw-window-handle` 経由で取得できることを確認する。
//! - その HWND に IMM32 の IME context (HIMC) が関連付いているか確認する。
//! - IME の open / conversion / sentence 状態と egui IME Event を同時観測する。
//! - CEF OSR 公式実装の `WM_IME_*` / HIMC 経路へ進む前に、Native IME context の成立を切り分ける。
//!
//! 注意点:
//! - Windows 専用の技術検証 Probe。
//! - 本 Probe は HWND の WndProc を変更しない。
//! - `WM_IME_*` の直接捕捉は Native IME context の成立確認後に別 Probe で行う。
//! - 正式 Surface API ではない。

use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::ffi::c_void;
use std::time::Duration;

const MESSAGE_PUMP_INTERVAL: Duration = Duration::from_millis(50);

/// Win32 HWND。
#[cfg(target_os = "windows")]
type Hwnd = *mut c_void;

/// IMM32 input context handle。
#[cfg(target_os = "windows")]
type Himc = *mut c_void;

#[cfg(target_os = "windows")]
#[link(name = "imm32")]
unsafe extern "system" {
    fn ImmGetContext(hwnd: Hwnd) -> Himc;
    fn ImmReleaseContext(hwnd: Hwnd, himc: Himc) -> i32;
    fn ImmGetOpenStatus(himc: Himc) -> i32;
    fn ImmGetConversionStatus(
        himc: Himc,
        conversion: *mut u32,
        sentence: *mut u32,
    ) -> i32;
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
unsafe extern "system" {
    fn GetFocus() -> Hwnd;
}

/// HWND と IMM32 状態を保持する表示用 snapshot。
#[derive(Debug, Clone, Default)]
struct NativeImeSnapshot {
    hwnd: usize,
    focused_hwnd: usize,
    himc: usize,
    has_context: bool,
    ime_open: bool,
    conversion_mode: u32,
    sentence_mode: u32,
    conversion_status_valid: bool,
}

/// `eframe::Frame` から Win32 HWND を取得する。
///
/// # 引数
/// - `frame`: 現在の eframe Frame。
///
/// # 戻り値
/// - Windows の場合は HWND。取得できない場合はエラー文字列。
///
/// @hldocs.ref doc-20260912-104801Z-WV16#sec_n4q8c2v7m1kt
#[cfg(target_os = "windows")]
fn hwnd_from_frame(frame: &eframe::Frame) -> Result<Hwnd, String> {
    let handle = frame
        .window_handle()
        .map_err(|error| format!("window_handle failed: {error}"))?;

    match handle.as_raw() {
        RawWindowHandle::Win32(win32) => Ok(win32.hwnd.get() as Hwnd),
        other => Err(format!("unexpected raw window handle: {other:?}")),
    }
}

/// HWND に関連付いた IMM32 context と現在状態を取得する。
///
/// # 引数
/// - `hwnd`: eframe native window handle。
///
/// # 戻り値
/// - 表示用 snapshot。
///
/// @hldocs.ref doc-20260912-104801Z-WV16#sec_n4q8c2v7m1kt
#[cfg(target_os = "windows")]
fn query_native_ime(hwnd: Hwnd) -> NativeImeSnapshot {
    let focused_hwnd = unsafe { GetFocus() };
    let himc = unsafe { ImmGetContext(hwnd) };

    if himc.is_null() {
        return NativeImeSnapshot {
            hwnd: hwnd as usize,
            focused_hwnd: focused_hwnd as usize,
            ..Default::default()
        };
    }

    let ime_open = unsafe { ImmGetOpenStatus(himc) != 0 };
    let mut conversion_mode = 0_u32;
    let mut sentence_mode = 0_u32;
    let conversion_status_valid = unsafe {
        ImmGetConversionStatus(himc, &mut conversion_mode, &mut sentence_mode) != 0
    };

    unsafe {
        ImmReleaseContext(hwnd, himc);
    }

    NativeImeSnapshot {
        hwnd: hwnd as usize,
        focused_hwnd: focused_hwnd as usize,
        himc: himc as usize,
        has_context: true,
        ime_open,
        conversion_mode,
        sentence_mode,
        conversion_status_valid,
    }
}

struct NativeImeProbeApp {
    snapshot: NativeImeSnapshot,
    last_error: Option<String>,
    preedit_count: u64,
    commit_count: u64,
    last_preedit: String,
    last_commit: String,
}

impl NativeImeProbeApp {
    fn new() -> Self {
        Self {
            snapshot: NativeImeSnapshot::default(),
            last_error: None,
            preedit_count: 0,
            commit_count: 0,
            last_preedit: String::new(),
            last_commit: String::new(),
        }
    }

    fn collect_egui_ime_events(&mut self, ctx: &egui::Context) {
        let events = ctx.input(|input| input.events.clone());
        for event in events {
            let egui::Event::Ime(ime_event) = event else {
                continue;
            };

            match ime_event {
                egui::ImeEvent::Preedit(text) => {
                    self.preedit_count = self.preedit_count.saturating_add(1);
                    self.last_preedit = text.clone();
                    println!("egui IME preedit: {:?}", text);
                }
                egui::ImeEvent::Commit(text) => {
                    self.commit_count = self.commit_count.saturating_add(1);
                    self.last_commit = text.clone();
                    println!("egui IME commit: {:?}", text);
                }
                _ => {}
            }
        }
    }
}

impl eframe::App for NativeImeProbeApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        #[cfg(target_os = "windows")]
        match hwnd_from_frame(frame) {
            Ok(hwnd) => {
                let previous = self.snapshot.clone();
                self.snapshot = query_native_ime(hwnd);
                self.last_error = None;

                if previous.hwnd != self.snapshot.hwnd
                    || previous.himc != self.snapshot.himc
                    || previous.ime_open != self.snapshot.ime_open
                    || previous.conversion_mode != self.snapshot.conversion_mode
                    || previous.sentence_mode != self.snapshot.sentence_mode
                {
                    println!(
                        "Native IME: hwnd=0x{:X} focus=0x{:X} himc=0x{:X} context={} open={} conversion=0x{:08X} sentence=0x{:08X} valid={}",
                        self.snapshot.hwnd,
                        self.snapshot.focused_hwnd,
                        self.snapshot.himc,
                        self.snapshot.has_context,
                        self.snapshot.ime_open,
                        self.snapshot.conversion_mode,
                        self.snapshot.sentence_mode,
                        self.snapshot.conversion_status_valid,
                    );
                }
            }
            Err(error) => {
                self.last_error = Some(error);
            }
        }

        self.collect_egui_ime_events(ctx);

        ctx.send_viewport_cmd(egui::ViewportCommand::IMEAllowed(true));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("WV-11-04-02 Windows Native IME Context Probe");
            ui.label("Click this window, enable Japanese IME, then type without confirming.");
            ui.separator();

            ui.monospace(format!("HWND          : 0x{:X}", self.snapshot.hwnd));
            ui.monospace(format!(
                "Focused HWND  : 0x{:X} ({})",
                self.snapshot.focused_hwnd,
                if self.snapshot.hwnd != 0 && self.snapshot.hwnd == self.snapshot.focused_hwnd {
                    "THIS WINDOW"
                } else {
                    "OTHER"
                }
            ));
            ui.monospace(format!("HIMC          : 0x{:X}", self.snapshot.himc));
            ui.monospace(format!("Has context   : {}", self.snapshot.has_context));
            ui.monospace(format!("IME open      : {}", self.snapshot.ime_open));
            ui.monospace(format!(
                "Conversion    : 0x{:08X}",
                self.snapshot.conversion_mode
            ));
            ui.monospace(format!(
                "Sentence      : 0x{:08X}",
                self.snapshot.sentence_mode
            ));
            ui.monospace(format!(
                "Status valid  : {}",
                self.snapshot.conversion_status_valid
            ));

            ui.separator();
            ui.label("egui IME events");
            ui.monospace(format!("Preedit count : {}", self.preedit_count));
            ui.monospace(format!("Last preedit  : {:?}", self.last_preedit));
            ui.monospace(format!("Commit count  : {}", self.commit_count));
            ui.monospace(format!("Last commit   : {:?}", self.last_commit));

            if let Some(error) = &self.last_error {
                ui.separator();
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
        });

        ctx.request_repaint_after(MESSAGE_PUMP_INTERVAL);
    }
}

fn main() -> eframe::Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("win_ime_native_context_probe is Windows-only");
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        println!("WV-11-04-02 Windows native IME context probe start");
        println!("Enable Japanese IME and type without confirming.");

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("WV-11-04-02 Windows Native IME Context Probe")
                .with_inner_size([760.0, 560.0]),
            ..Default::default()
        };

        eframe::run_native(
            "WV-11-04-02 Windows Native IME Context Probe",
            native_options,
            Box::new(|_cc| Ok(Box::new(NativeImeProbeApp::new()))),
        )
    }
}
