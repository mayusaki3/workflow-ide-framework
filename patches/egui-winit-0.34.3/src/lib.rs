pub use egui;
pub use winit;

use raw_window_handle::HasDisplayHandle;
use winit::window::Window;

#[must_use]
#[derive(Clone, Copy, Debug, Default)]
pub struct EventResponse {
    pub consumed: bool,
    pub repaint: bool,
}

pub struct State {
    egui_ctx: egui::Context,
    allow_ime: bool,
    ime_rect_px: Option<egui::Rect>,
}

impl State {
    pub fn new(
        egui_ctx: egui::Context,
        _viewport_id: egui::ViewportId,
        _display_target: &dyn HasDisplayHandle,
        _native_pixels_per_point: Option<f32>,
        _theme: Option<winit::window::Theme>,
        _max_texture_side: Option<usize>,
    ) -> Self {
        Self {
            egui_ctx,
            allow_ime: false,
            ime_rect_px: None,
        }
    }

    pub fn allow_ime(&self) -> bool {
        self.allow_ime
    }

    pub fn set_allow_ime(&mut self, allow: bool) {
        self.allow_ime = allow;
    }

    pub fn egui_ctx(&self) -> &egui::Context {
        &self.egui_ctx
    }

    pub fn handle_platform_output(
        &mut self,
        window: &Window,
        platform_output: egui::PlatformOutput,
    ) {
        let ime = platform_output.ime;
        let allow_ime = ime.is_some();
        if self.allow_ime != allow_ime {
            self.allow_ime = allow_ime;
            eprintln!("WFIDE IME TRACE set_ime_allowed={allow_ime}");
            window.set_ime_allowed(allow_ime);
        }

        if let Some(ime) = ime {
            let ppp = self.egui_ctx.pixels_per_point();
            let rect_px = ppp * ime.rect;
            let cursor_px = ppp * ime.cursor_rect;
            eprintln!(
                "WFIDE IME TRACE rect=({:.1},{:.1},{:.1},{:.1}) cursor=({:.1},{:.1},{:.1},{:.1}) ppp={:.3}",
                rect_px.min.x,
                rect_px.min.y,
                rect_px.width(),
                rect_px.height(),
                cursor_px.min.x,
                cursor_px.min.y,
                cursor_px.width(),
                cursor_px.height(),
                ppp,
            );

            // Diagnostic fix: anchor the OS IME candidate area to the primary cursor.
            if self.ime_rect_px != Some(cursor_px)
                || self.egui_ctx.input(|i| !i.events.is_empty())
            {
                self.ime_rect_px = Some(cursor_px);
                eprintln!(
                    "WFIDE IME TRACE set_ime_cursor_area=({:.1},{:.1},{:.1},{:.1})",
                    cursor_px.min.x,
                    cursor_px.min.y,
                    cursor_px.width(),
                    cursor_px.height(),
                );
                window.set_ime_cursor_area(
                    winit::dpi::PhysicalPosition {
                        x: cursor_px.min.x,
                        y: cursor_px.min.y,
                    },
                    winit::dpi::PhysicalSize {
                        width: cursor_px.width(),
                        height: cursor_px.height(),
                    },
                );
            }
        } else {
            self.ime_rect_px = None;
        }
    }
}
