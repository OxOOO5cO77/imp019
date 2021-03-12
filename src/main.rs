pub use app::Imp019App;

mod app;
mod team;
mod player;
mod results;
mod league;
mod data;
mod schedule;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = app::Imp019App::new();
    eframe::run_native(Box::new(app));
}
