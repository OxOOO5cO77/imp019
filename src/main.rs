pub use app::Imp019App;

mod app;
mod team;
mod player;
mod league;
mod data;
mod schedule;
mod util;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = app::Imp019App::new();
    eframe::run_native(Box::new(app));
}
