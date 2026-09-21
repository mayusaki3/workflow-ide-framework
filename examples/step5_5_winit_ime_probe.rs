//! Direct winit IME cursor-area probe.
//!
//! Move the mouse horizontally. The IME candidate window should follow the
//! cursor position while composing text. This intentionally bypasses egui,
//! eframe, Dock and TextEditor.

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Ime, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    ime_pos: PhysicalPosition<f64>,
    preedit: String,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(
                        WindowAttributes::default()
                            .with_title("WFIDE - Direct winit IME Cursor Area Probe")
                            .with_inner_size(PhysicalSize::new(1000, 700)),
                    )
                    .expect("create window"),
            );
            window.set_ime_allowed(true);
            self.ime_pos = PhysicalPosition::new(100.0, 200.0);
            window.set_ime_cursor_area(self.ime_pos, PhysicalSize::new(2, 24));
            println!("IME AREA x={:.1} y={:.1}", self.ime_pos.x, self.ime_pos.y);
            self.window = Some(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else { return };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::CursorMoved { position, .. } => {
                self.ime_pos = position;
                window.set_ime_cursor_area(position, PhysicalSize::new(2, 24));
                window.set_title(&format!("WFIDE IME Probe | cursor=({:.0},{:.0}) | composing: {}", position.x, position.y, self.preedit));
            }
            WindowEvent::Ime(event) => {
                println!("IME EVENT {event:?}");
                match &event {
                    Ime::Preedit(text, _) => self.preedit = text.clone(),
                    Ime::Commit(text) => self.preedit = format!("COMMIT: {text}"),
                    Ime::Enabled => self.preedit = "IME enabled".to_owned(),
                    Ime::Disabled => self.preedit = "IME disabled".to_owned(),
                }
                window.set_title(&format!(
                    "WFIDE IME Probe | cursor=({:.0},{:.0}) | composing: {}",
                    self.ime_pos.x, self.ime_pos.y, self.preedit
                ));
                if !matches!(event, Ime::Disabled) {
                    window.set_ime_cursor_area(self.ime_pos, PhysicalSize::new(2, 24));
                }
            }
            _ => {}
        }
    }
}

fn main() {
    println!("Direct winit IME probe");
    println!("1. Enable Japanese IME.");
    println!("2. Move the mouse to the left or right side of this window.");
    println!("3. Start composition. The current preedit text is shown in the window title.");\n    println!("4. Candidate UI should appear near the mouse position.");
    let event_loop = EventLoop::new().expect("create event loop");
    event_loop.run_app(&mut App::default()).expect("run app");
}
