//! Step 5.5 diagnostic: egui-winit IME rect semantics.
//!
//! This does not render egui. It compares the two rectangles carried by
//! egui::IMEOutput and documents which one must be used for candidate placement.

use egui_winit::egui::{pos2, vec2, IMEOutput, Rect};

fn main() {
    let editor_rect = Rect::from_min_size(pos2(0.0, 168.0), vec2(995.0, 118.0));
    let cursor_rect = Rect::from_min_size(pos2(896.6, 175.5), vec2(3.0, 22.0));
    let ime = IMEOutput {
        rect: editor_rect,
        cursor_rect,
        should_interrupt_composition: false,
        ..Default::default()
    };

    println!("IMEOutput.rect        = {:?}", ime.rect);
    println!("IMEOutput.cursor_rect = {:?}", ime.cursor_rect);
    println!("candidate anchor must follow cursor_rect, not the TextEdit widget rect");
}
