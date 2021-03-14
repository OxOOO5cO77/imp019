use eframe::{egui, epi};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::data::Data;
use crate::league::League;
use crate::team::Team;

enum Mode {
    Schedule,
    Standings,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct Imp019App {
    rng: ThreadRng,
    pub leagues: Vec<League>,
    pub disp_league: usize,
    disp_mode: Mode,
}

impl Default for Imp019App {
    fn default() -> Self {
        Imp019App {
            rng: rand::thread_rng(),
            leagues: Vec::new(),
            disp_league: 0,
            disp_mode: Mode::Schedule,
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
        leagues.push(League::new(&mut data, 20, &mut rng));
        leagues.push(League::new(&mut data, 20, &mut rng));
        leagues.push(League::new(&mut data, 20, &mut rng));

        // league::relegate_promote(&mut leagues, 4);
        //
        // for league in &mut leagues {
        //     league.reset();
        // }
        //}

        Imp019App {
            rng,
            leagues,
            ..Default::default()
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

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::SidePanel::left("side_panel", 200.0).show(ctx, |ui| {
            ui.heading("Leagues");
            for cnt in 0..self.leagues.len() {
                ui.horizontal(|ui| {
                    ui.label(format!("League {}", cnt + 1));
                    if ui.button("Schedule").clicked() {
                        self.disp_mode = Mode::Schedule;
                        self.disp_league = cnt;
                    }
                    if ui.button("Standings").clicked() {
                        self.disp_mode = Mode::Standings;
                        self.disp_league = cnt;
                    }
                });
            }
            ui.separator();
        });

        egui::TopPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                ui.separator();
                if ui.button("Sim").clicked() {
                    self.update();
                };
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let league = &self.leagues[self.disp_league];
            match self.disp_mode {
                Mode::Schedule => {
                    let total_games = league.schedule.games.len();

                    let cur_idx = league.schedule.games.iter().position(|o| o.home.r == o.away.r).or(Some(total_games)).unwrap();
                    let teams = league.teams.len();

                    if cur_idx < total_games {
                        ui.heading(format!("Today ({})",cur_idx/(teams/2)));
                        for idx in cur_idx..(cur_idx + (teams / 2)) {
                            let game = &league.schedule.games[idx];
                            let home_team = &league.teams[game.home.team];
                            let away_team = &league.teams[game.away.team];
                            ui.label(format!("{} @ {}", away_team.abbr, home_team.abbr));
                        }
                    }

                    if cur_idx > 0 {
                        ui.heading("Yesterday");
                        let end = cur_idx as i32;
                        let start = end - ((teams/2) as i32);
                        for past_idx in start..end {
                            if past_idx >= 0 {
                                let game = &league.schedule.games[past_idx as usize];
                                let home_team = &league.teams[game.home.team];
                                let away_team = &league.teams[game.away.team];
                                ui.label(format!("{} {:2} @ {:2} {}", away_team.abbr, game.away.r, game.home.r, home_team.abbr));
                            }
                        }
                    }

                }
                Mode::Standings => {
                    egui::Grid::new("standings").show(ui, |ui| {
                        ui.label("Rank");
                        ui.label("Abbr");
                        ui.label("Team");
                        ui.label("Record");
                        ui.end_row();

                        let mut cloned_teams = league.teams.iter().collect::<Vec<&Team>>();
                        cloned_teams.sort_by_key(|o| {
                            let denom = o.results.win + o.results.lose;
                            if denom > 0 {
                                (o.results.win * 1000 / denom) + 1
                            } else {
                                0
                            }
                        });
                        cloned_teams.reverse();


                        let mut rank = 1;
                        for team in &cloned_teams {
                            ui.label(format!("{}", rank));
                            ui.label(team.abbr.as_str());
                            ui.label(team.name());
                            ui.label(format!("{}-{}", team.results.win, team.results.lose));
                            ui.end_row();
                            rank += 1;
                        }
                    });
                }
            }
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
