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
fn main() -> eframe::Result<()> {
    let options = NativeOptions::default();
    eframe::run_native("imp019", options, Box::new(|cc| Box::new(Imp019App::new(cc))))
}
