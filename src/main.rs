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
    eframe::run_native(Box::new(app));
}
