#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use freya::prelude::*;

fn main() {
    launch(app);
}

static FERRIS: &[u8] = include_bytes!("./ferris.svg");

fn app(cx: Scope) -> Element {
    let ferris_a = bytes_to_data(cx, FERRIS);
    let ferris_b = bytes_to_data(cx, FERRIS);
    render!(
        svg { width: "100%", height: "50%", svg_data: ferris_a }
        svg { width: "100%", height: "50%", svg_data: ferris_b }
    )
}
