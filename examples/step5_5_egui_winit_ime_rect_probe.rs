//! Step 5.5 diagnostic: egui-winit IME rect semantics.
//!
//! This probe does not render egui. It records the distinction between
//! the TextEdit rectangle and the primary cursor rectangle using plain egui types.

use egui_winit::egui::{pos2, vec2, Rect};

fn main() {
    let editor_rect = Rect::from_min_size(pos2(0.0, 168.0), vec2(995.0, 118.0));
    let cursor_rect = Rect::from_min_size(pos2(896.6, 175.5), vec2(3.0, 22.0));

    println!("TextEdit rect = {:?}", editor_rect);
    println!("Cursor rect   = {:?}", cursor_rect);
    println!("Expected IME candidate anchor = cursor rect, not TextEdit rect");
}
