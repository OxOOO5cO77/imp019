#![forbid(unsafe_code)]
//#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

pub use app::Imp019App;

mod app;
mod data;
mod league;
mod player;
mod schedule;
mod stat;
mod team;
mod util;

// ----------------------------------------------------------------------------
// When compiling for web:

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    let app = Imp019App::new();
    eframe::start_web(canvas_id, Box::new(app))
}
