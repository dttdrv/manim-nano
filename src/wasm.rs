//! WebAssembly entry points for manim-nano.
//!
//! This module is only compiled with the `wasm` feature enabled (see
//! `Cargo.toml`). It exposes the natural-language renderer to JavaScript so the
//! engine runs entirely in the browser — no server, no network, no API key.
//!
//! Build it with either of:
//!
//! ```text
//! wasm-pack build --target web --features wasm
//! cargo build --lib --features wasm --target wasm32-unknown-unknown
//! ```

use crate::nl::interpret_into;
use crate::scene::Scene;
use wasm_bindgen::prelude::*;

/// Render a plain-English description to an animated GIF and return its bytes.
///
/// The returned `Vec<u8>` is handed to JavaScript as a `Uint8Array`, ready to
/// wrap in a `Blob` of type `image/gif`. Everything runs locally in the
/// browser via WebAssembly.
///
/// - `text`: the animation description (e.g. `"Draw a red circle, then move it
///   right while a blue square fades in"`).
/// - `width` / `height`: output resolution in pixels (clamped to sane bounds).
/// - `fps`: frames per second (clamped to at least 1).
///
/// On any failure the function returns an empty buffer rather than trapping, so
/// the JS side can simply check for an empty result.
#[wasm_bindgen]
pub fn render_gif(text: &str, width: u32, height: u32, fps: u32) -> Vec<u8> {
    // Clamp inputs so a bad request can never allocate an absurd pixmap.
    let width = width.clamp(16, 4096);
    let height = height.clamp(16, 4096);
    let fps = fps.clamp(1, 60);

    let scene = Scene::new(width, height).fps(fps);
    let mut scene = interpret_into(text, scene);

    scene.encode_gif().unwrap_or_default()
}

/// Returns the crate version string, useful for a small "about" line in the UI.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
