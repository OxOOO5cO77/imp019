use eframe::epi::App;
use eframe::NativeOptions;

pub use app::Imp019App;

mod app;
mod data;
mod game;
mod league;
mod player;
mod schedule;
mod stat;
mod team;
mod util;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = app::Imp019App::new();
    let options = NativeOptions {
        always_on_top: false,
        decorated: true,
        drag_and_drop_support: false,
        icon_data: None,
        initial_window_size: Some(app.max_size_points()),
        resizable: true,
        transparent: false,
    };
    eframe::run_native(Box::new(app), options);
}
