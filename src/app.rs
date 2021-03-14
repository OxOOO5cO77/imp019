use eframe::{egui, epi};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::data::Data;
use crate::league::League;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct Imp019App {
    rng: ThreadRng,
    pub leagues: Vec<League>,
    pub disp_league: usize,
}

impl Default for Imp019App {
    fn default() -> Self {
        Imp019App {
            rng: rand::thread_rng(),
            leagues: Vec::new(),
            disp_league: 0,
        }
    }
}

impl Imp019App {
    pub fn new() -> Self {
        let mut data = Data::new();

        let mut rng = rand::thread_rng();
        data.loc.shuffle(&mut rng);
        data.nick.shuffle(&mut rng);

        let mut leagues = Vec::new();
        leagues.push(League::new(&mut data, 28, &mut rng));
        leagues.push(League::new(&mut data, 28, &mut rng));
        leagues.push(League::new(&mut data, 28, &mut rng));

        // league::relegate_promote(&mut leagues, 4);
        //
        // for league in &mut leagues {
        //     league.reset();
        // }
        //}

        Imp019App {
            rng,
            leagues,
            disp_league: 0,
        }
    }

    pub fn update(&mut self) {
        for league in &mut self.leagues {
            league.sim(&mut self.rng);
        }
    }
}

impl epi::App for Imp019App {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {

        self.update();

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::SidePanel::left("side_panel", 200.0).show(ctx, |ui| {
            ui.heading("Side Panel");
            for cnt in 0..self.leagues.len() {
                if ui.button(format!("League {}", cnt + 1)).clicked() {
                    self.disp_league = cnt;
                }
            }
        });

        egui::TopPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("standings").show(ui, |ui| {
                ui.label("Rank");
                ui.label("Abbr");
                ui.label("Team");
                ui.label("Record");
                ui.end_row();

                let mut rank = 1;
                for team in &self.leagues[self.disp_league].teams {
                    ui.label(format!("{}", rank));
                    ui.label(team.abbr.as_str());
                    ui.label(team.name());
                    ui.label(format!("{}-{}", team.results.win, team.results.lose));
                    ui.end_row();
                    rank += 1;
                }
            });
        });

        ctx.request_repaint();
    }

    /// Called by the framework to load old app state (if any).
    #[cfg(feature = "persistence")]
    fn load(&mut self, storage: &dyn epi::Storage) {
        *self = epi::get_value(storage, epi::APP_KEY).unwrap()
    }

    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn name(&self) -> &str {
        "imp019"
    }
}
