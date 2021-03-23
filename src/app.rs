use eframe::{egui, epi};
use eframe::egui::Button;
use ordinal::Ordinal;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::data::Data;
use crate::league::{end_of_season, League};
use crate::player::Stat;
use crate::team::Team;

#[derive(Copy, Clone)]
enum Mode {
    Schedule,
    Standings,
    Team(u64),
    Player(u64, u64),
    Leaders(Stat),
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct Imp019App {
    rng: ThreadRng,
    pub leagues: Vec<League>,
    year: u32,
    pub disp_league: usize,
    disp_mode: Mode,
}

impl Default for Imp019App {
    fn default() -> Self {
        Imp019App {
            rng: rand::thread_rng(),
            leagues: Vec::new(),
            year: 2030,
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

        let year = 2030;
        let mut team_id = 0;
        let mut player_id = 0;
        let mut leagues = Vec::new();
        leagues.push(League::new(1, &mut data, 20, year, &mut team_id, &mut player_id, &mut rng));
        leagues.push(League::new(2, &mut data, 20, year, &mut team_id, &mut player_id, &mut rng));
        leagues.push(League::new(3, &mut data, 20, year, &mut team_id, &mut player_id, &mut rng));

        Imp019App {
            rng,
            leagues,
            year,
            ..Default::default()
        }
    }

    pub fn update(&mut self) -> bool {
        let mut result = false;
        for league in &mut self.leagues {
            result = league.sim(&mut self.rng) || result;
        }
        result
    }
}

fn as_league(value: Option<u32>) -> String {
    if let Some(pos) = value {
        format!("{} in League {}", Ordinal(pos % 100), pos / 100)
    } else {
        "---".to_string()
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
                    let result = self.update();
                    if !result {
                        end_of_season(&mut self.leagues, 4, self.year, &mut self.rng);
                        self.year += 1;
                    }
                };
                if ui.button("Sim All").clicked() {
                    while self.update() {}
                }
            });
        });

        egui::SidePanel::left("side_panel", 200.0).show(ctx, |ui| {
            ui.heading("Leagues");
            for cnt in 0..self.leagues.len() {
                ui.horizontal(|ui| {
                    ui.label(format!("League {}", cnt + 1));
                    if ui.button("Sche").clicked() {
                        self.disp_mode = Mode::Schedule;
                        self.disp_league = cnt;
                    }
                    if ui.button("Stan").clicked() {
                        self.disp_mode = Mode::Standings;
                        self.disp_league = cnt;
                    }
                    if ui.button("Lead").clicked() {
                        self.disp_mode = Mode::Leaders(Stat::HR);
                        self.disp_league = cnt;
                    }
                });
            }
            ui.separator();
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let league = &self.leagues[self.disp_league];
            self.disp_mode = match &self.disp_mode {
                Mode::Schedule => {
                    let total_games = league.schedule.games.len();

                    let cur_idx = league.schedule.games.iter().position(|o| o.home.r == o.away.r).unwrap_or(total_games);
                    let teams = league.teams.len();

                    if cur_idx < total_games {
                        ui.heading(format!("Today ({})", cur_idx / (teams / 2)));
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
                        let start = end - ((teams / 2) as i32);
                        for past_idx in start..end {
                            if past_idx >= 0 {
                                let game = &league.schedule.games[past_idx as usize];
                                let home_team = &league.teams[game.home.team];
                                let away_team = &league.teams[game.away.team];
                                ui.label(format!("{} {:2} @ {:2} {}", away_team.abbr, game.away.r, game.home.r, home_team.abbr));
                            }
                        }
                    }
                    Mode::Schedule
                }
                Mode::Standings => {
                    let mut mode = Mode::Standings;
                    egui::Grid::new("standings").show(ui, |ui| {
                        ui.label("Rank");
                        ui.label("Abbr");
                        ui.label("Team");
                        ui.label("Record");
                        ui.end_row();

                        let teams = &mut league.teams.iter().collect::<Vec<&Team>>();
                        teams.sort_by_key(|o| {
                            let denom = o.results.win + o.results.lose;
                            if denom > 0 {
                                (o.results.win * 1000 / denom) + 1
                            } else {
                                0
                            }
                        });
                        teams.reverse();


                        let mut rank = 1;
                        for team in teams.iter() {
                            ui.label(format!("{}", rank));
                            ui.label(team.abbr.as_str());
                            if ui.add(Button::new(team.name()).frame(false)).clicked() {
                                mode = Mode::Team(team.id);
                            }
                            ui.label(format!("{}-{}", team.results.win, team.results.lose));
                            ui.end_row();
                            rank += 1;
                        }
                    });
                    mode
                }
                Mode::Team(id) => {
                    let mut mode = Mode::Team(*id);
                    if ui.button("Close").clicked() {
                        mode = Mode::Standings;
                    }

                    let team = &mut league.teams.iter().find(|o| o.id == *id).unwrap();
                    ui.label(team.name());
                    ui.label(format!("Founded: {}", team.history.founded));
                    ui.label(format!("Best: {}", as_league(team.history.best)));
                    ui.label(format!("Worst: {}", as_league(team.history.worst)));
                    ui.label(format!("Wins: {}", team.history.wins));
                    ui.label(format!("Losses: {}", team.history.losses));

                    ui.horizontal(|ui| {
                        if !team.history.results.is_empty() {
                            ui.vertical(|ui| {
                                ui.heading("History");
                                egui::Grid::new("history").striped(true).show(ui, |ui| {
                                    ui.label("Year");
                                    ui.label("League");
                                    ui.label("Rank");
                                    ui.label("W");
                                    ui.label("L");
                                    ui.end_row();

                                    ui.end_row();

                                    for result in &team.history.results {
                                        ui.label(format!("{}", result.year));
                                        ui.label(format!("League {}", result.league));
                                        ui.label(format!("{}", Ordinal(result.rank)));
                                        ui.label(format!("{}", result.win));
                                        ui.label(format!("{}", result.lose));
                                        ui.end_row();
                                    }
                                });
                            });
                        }


                        if !team.players.is_empty() {
                            ui.vertical(|ui| {
                                ui.heading("Players");
                                egui::Grid::new("players").striped(true).show(ui, |ui| {
                                    ui.label("Name");
                                    ui.label("Pos");
                                    ui.label("PA");
                                    ui.label("AB");
                                    ui.label("H");
                                    ui.label("2B");
                                    ui.label("3B");
                                    ui.label("HR");
                                    ui.label("BB");
                                    ui.label("HBP");
                                    ui.label("AVG");
                                    ui.label("OBP");
                                    ui.label("SLG");
                                    ui.end_row();


                                    for player in &team.players {
                                        let stats = player.get_stats();

                                        if ui.add(Button::new(&player.fullname()).frame(false)).clicked() {
                                            mode = Mode::Player(*id, player.id);
                                        }
                                        ui.label(player.display_position());
                                        ui.label(format!("{}", stats.pa));
                                        ui.label(format!("{}", stats.ab));
                                        ui.label(format!("{}", stats.h));
                                        ui.label(format!("{}", stats.h2b));
                                        ui.label(format!("{}", stats.h3b));
                                        ui.label(format!("{}", stats.hr));
                                        ui.label(format!("{}", stats.bb));
                                        ui.label(format!("{}", stats.hbp));
                                        ui.label(format!("{}.{:03}", stats.avg / 1000, stats.avg % 1000));
                                        ui.label(format!("{}.{:03}", stats.obp / 1000, stats.obp % 1000));
                                        ui.label(format!("{}.{:03}", stats.slg / 1000, stats.slg % 1000));
                                        ui.end_row();
                                    }
                                });
                            });
                        }
                    });


                    mode
                }
                Mode::Player(team_id, player_id) => {
                    let mut mode = Mode::Player(*team_id, *player_id);
                    if ui.button("Close").clicked() {
                        mode = Mode::Team(*team_id);
                    }

                    let team = &mut league.teams.iter().find(|o| o.id == *team_id).unwrap();
                    let player = &team.players.iter().find(|o| o.id == *player_id).unwrap();

                    egui::Grid::new("history").striped(true).show(ui, |ui| {
                        ui.label("Year");
                        ui.label("League");
                        ui.label("Team");
                        ui.label("PA");
                        ui.label("AB");
                        ui.label("H");
                        ui.label("2B");
                        ui.label("3B");
                        ui.label("HR");
                        ui.label("BB");
                        ui.label("HBP");
                        ui.label("AVG");
                        ui.label("OBP");
                        ui.label("SLG");
                        ui.end_row();


                        for history in &player.historical {
                            let stats = history.get_stats();

                            ui.label(format!("{}", history.year));
                            ui.label(format!("{}", history.league));
                            ui.label(format!("{}", history.team));
                            ui.label(format!("{}", stats.pa));
                            ui.label(format!("{}", stats.ab));
                            ui.label(format!("{}", stats.h));
                            ui.label(format!("{}", stats.h2b));
                            ui.label(format!("{}", stats.h3b));
                            ui.label(format!("{}", stats.hr));
                            ui.label(format!("{}", stats.bb));
                            ui.label(format!("{}", stats.hbp));
                            ui.label(format!("{}.{:03}", stats.avg / 1000, stats.avg % 1000));
                            ui.label(format!("{}.{:03}", stats.obp / 1000, stats.obp % 1000));
                            ui.label(format!("{}.{:03}", stats.slg / 1000, stats.slg % 1000));
                            ui.end_row();
                        }
                    });

                    mode
                }
                Mode::Leaders(result) => {
                    let mut mode = Mode::Leaders(*result);

                    egui::Grid::new("leaders").striped(true).show(ui, |ui| {
                        ui.label("#");
                        ui.label("Name");
                        ui.label("Team");
                        if ui.button("PA").clicked() {
                            mode = Mode::Leaders(Stat::PA);
                        }
                        if ui.button("AB").clicked() {
                            mode = Mode::Leaders(Stat::AB);
                        }
                        if ui.button("H").clicked() {
                            mode = Mode::Leaders(Stat::H);
                        }
                        if ui.button("2B").clicked() {
                            mode = Mode::Leaders(Stat::H2b);
                        }
                        if ui.button("3B").clicked() {
                            mode = Mode::Leaders(Stat::H3b);
                        }
                        if ui.button("HR").clicked() {
                            mode = Mode::Leaders(Stat::HR);
                        }
                        if ui.button("BB").clicked() {
                            mode = Mode::Leaders(Stat::BB);
                        }
                        if ui.button("HBP").clicked() {
                            mode = Mode::Leaders(Stat::HBP);
                        }
                        if ui.button("AVG").clicked() {
                            mode = Mode::Leaders(Stat::AVG);
                        }
                        if ui.button("OBP").clicked() {
                            mode = Mode::Leaders(Stat::OBP);
                        }
                        if ui.button("SLG").clicked() {
                            mode = Mode::Leaders(Stat::SLG);
                        }
                        ui.end_row();

                        let mut all_players = Vec::new();

                        for team in league.teams.iter() {
                            for player in team.players.iter() {
                                all_players.push((&team.abbr, player));
                            }
                        }

                        all_players.sort_by_key(|o| o.1.get_stat(*result));
                        all_players.reverse();

                        for (rank, ap) in all_players[0..30].iter().enumerate() {
                            let player = ap.1;

                            ui.label(format!("{}", rank + 1));
                            ui.label(player.fullname());
                            ui.label(ap.0);


                            let stats = player.get_stats();

                            ui.label(format!("{}", stats.pa));
                            ui.label(format!("{}", stats.ab));
                            ui.label(format!("{}", stats.h));
                            ui.label(format!("{}", stats.h2b));
                            ui.label(format!("{}", stats.h3b));
                            ui.label(format!("{}", stats.hr));
                            ui.label(format!("{}", stats.bb));
                            ui.label(format!("{}", stats.hbp));
                            ui.label(format!("{}.{:03}", stats.avg / 1000, stats.avg % 1000));
                            ui.label(format!("{}.{:03}", stats.obp / 1000, stats.obp % 1000));
                            ui.label(format!("{}.{:03}", stats.slg / 1000, stats.slg % 1000));
                            ui.end_row();
                        }
                    });

                    mode
                }
            }
        });
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
