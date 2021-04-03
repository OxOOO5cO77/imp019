use std::collections::{HashMap, HashSet};

use eframe::{egui, epi};
use eframe::egui::Button;
use ordinal::Ordinal;
use rand::rngs::ThreadRng;

use crate::data::Data;
use crate::league::{end_of_season, League};
use crate::player::{Player, Position, Stat};
use crate::team::Team;

#[derive(Copy, Clone, PartialEq)]
enum Mode {
    Schedule,
    Standings,
    Team(u64),
    Player(Option<u64>, u64),
    BatLeaders(Stat, bool),
    PitLeaders(Stat, bool),
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct Imp019App {
    rng: ThreadRng,
    players: HashMap<u64, Player>,
    teams: HashMap<u64, Team>,
    leagues: Vec<League>,
    year: u32,
    disp_league: usize,
    disp_mode: Mode,
    sim_all: bool,
}

impl Default for Imp019App {
    fn default() -> Self {
        Imp019App {
            rng: rand::thread_rng(),
            players: HashMap::new(),
            teams: HashMap::new(),
            leagues: Vec::new(),
            year: 2030,
            disp_league: 0,
            disp_mode: Mode::Schedule,
            sim_all: false,
        }
    }
}

struct DraftPlayer {
    id: u64,
    pos: Position,
    taken: bool,
}

impl Imp019App {
    fn pick_player(players: &mut Vec<DraftPlayer>, pos: Position) -> u64 {
        if let Some(player) = players.iter_mut().find(|o| !o.taken && o.pos == pos) {
            player.taken = true;
            player.id
        } else {
            0
        }
    }

    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let data = Data::new();
        let year = 2030;

        let mut pos_gen = vec![
            Position::Catcher,
            Position::FirstBase,
            Position::SecondBase,
            Position::ThirdBase,
            Position::ShortStop,
            Position::LeftField,
            Position::CenterField,
            Position::RightField,
            Position::DesignatedHitter,
        ];

        for _ in 0..9 {
            pos_gen.push(Position::Pitcher);
        }

        let mut player_id = 1;
        let mut players = HashMap::new();
        players.reserve(2000);
        for pos in pos_gen {
            for _ in 1..=200 {
                let name_first = data.choose_name_first(&mut rng);
                let name_last = data.choose_name_last(&mut rng);
                players.insert(player_id, Player::new(name_first, name_last, &pos, &mut rng));
                player_id += 1;
            }
        }

        let mut unused_players = players.iter().map(|(k, v)| DraftPlayer { id: *k, pos: v.pos, taken: false }).collect::<Vec<_>>();

        let locs = data.get_locs(&mut HashSet::new(), &mut rng, 60);
        let nicks = data.get_nicks(&mut HashSet::new(), &mut rng, 60);

        let mut teams = HashMap::new();
        teams.reserve(60);
        for team_id in 0..60 {
            let (abbr, city, state) = locs[team_id].clone();
            let nick = nicks[team_id].clone();
            let mut team = Team::new(abbr, city, state, nick, year);

            team.players.push(Imp019App::pick_player(&mut unused_players, Position::Catcher));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::FirstBase));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::SecondBase));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::ThirdBase));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::ShortStop));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::LeftField));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::CenterField));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::RightField));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::DesignatedHitter));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::Catcher));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::FirstBase));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::SecondBase));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::ThirdBase));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::ShortStop));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::LeftField));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::CenterField));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::RightField));
            team.players.push(Imp019App::pick_player(&mut unused_players, Position::DesignatedHitter));

            for rot in 0..5 {
                let p = Imp019App::pick_player(&mut unused_players, Position::Pitcher);
                team.rotation[rot] = p;
                team.players.push(p);
            }

            let team_id = (team_id + 1) as u64;
            teams.insert(team_id, team);
        }

        let mut remaining_teams = teams.keys().copied().collect();

        let leagues = vec![
            League::new(1, 20, &mut remaining_teams, &mut rng),
            League::new(2, 20, &mut remaining_teams, &mut rng),
            League::new(3, 20, &mut remaining_teams, &mut rng),
        ];

        Imp019App {
            rng,
            players,
            teams,
            leagues,
            year,
            disp_league: 0,
            disp_mode: Mode::Schedule,
            sim_all: false,
        }
    }

    pub fn update(&mut self) -> bool {
        let mut result = false;
        for league in &mut self.leagues {
            result = league.sim(&mut self.teams, &mut self.players, &mut self.rng) || result;
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

fn select_bat_stat(new_stat: Stat, cur_stat: Stat, reverse: bool, default: bool) -> Mode {
    let flip = if new_stat == cur_stat { !reverse } else { default };
    Mode::BatLeaders(new_stat, flip)
}

fn select_pit_stat(new_stat: Stat, cur_stat: Stat, reverse: bool, default: bool) -> Mode {
    let flip = if new_stat == cur_stat { !reverse } else { default };
    Mode::PitLeaders(new_stat, flip)
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
                        end_of_season(&mut self.leagues, &mut self.teams, &mut self.players, 4, self.year, &mut self.rng);
                        self.year += 1;
                    }
                };
                if ui.button("Sim All").clicked() {
                    self.sim_all = true;
                }
            });
        });

        if self.sim_all {
            self.sim_all = self.update();
            ctx.request_repaint();
        }

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
                    if ui.button("Bat").clicked() {
                        self.disp_mode = Mode::BatLeaders(Stat::Bhr, true);
                        self.disp_league = cnt;
                    }
                    if ui.button("Pit").clicked() {
                        self.disp_mode = Mode::PitLeaders(Stat::Pera, false);
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
                            let home_team = self.teams.get(&game.home.id).unwrap();
                            let away_team = self.teams.get(&game.away.id).unwrap();
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
                                let home_team = self.teams.get(&game.home.id).unwrap();
                                let away_team = self.teams.get(&game.away.id).unwrap();
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

                        let teams = &mut league.teams.iter().collect::<Vec<_>>();
                        teams.sort_by_key(|o| {
                            let team = self.teams.get(*o).unwrap();
                            team.win_pct()
                        });
                        teams.reverse();


                        let mut rank = 1;
                        for team_id in teams.iter() {
                            let team = self.teams.get(*team_id).unwrap();
                            ui.label(format!("{}", rank));
                            ui.label(team.abbr.as_str());
                            if ui.add(Button::new(team.name()).frame(false)).clicked() {
                                mode = Mode::Team(**team_id);
                            }
                            ui.label(format!("{}-{}", team.get_wins(), team.get_losses()));
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

                    let team = self.teams.get(id).unwrap();
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
                                ui.heading("Batting");
                                egui::Grid::new("batting").striped(true).show(ui, |ui| {
                                    ui.label("Name");
                                    ui.label("Pos");
                                    ui.label("G");
                                    ui.label("GS");
                                    ui.label("PA");
                                    ui.label("AB");
                                    ui.label("H");
                                    ui.label("2B");
                                    ui.label("3B");
                                    ui.label("HR");
                                    ui.label("BB");
                                    ui.label("HBP");
                                    ui.label("SO");
                                    ui.label("R");
                                    ui.label("RBI");
                                    ui.label("AVG");
                                    ui.label("OBP");
                                    ui.label("SLG");
                                    ui.end_row();


                                    for player_id in &team.players {
                                        let player = self.players.get(player_id).unwrap();
                                        if player.pos == Position::Pitcher {
                                            continue;
                                        }
                                        let stats = player.get_stats();

                                        if ui.add(Button::new(&player.fullname()).frame(false)).clicked() {
                                            mode = Mode::Player(Some(*id), *player_id);
                                        }
                                        ui.label(player.pos.to_str());
                                        ui.label(format!("{}", stats.g));
                                        ui.label(format!("{}", stats.gs));
                                        ui.label(format!("{}", stats.b_pa));
                                        ui.label(format!("{}", stats.b_ab));
                                        ui.label(format!("{}", stats.b_h));
                                        ui.label(format!("{}", stats.b_2b));
                                        ui.label(format!("{}", stats.b_3b));
                                        ui.label(format!("{}", stats.b_hr));
                                        ui.label(format!("{}", stats.b_bb));
                                        ui.label(format!("{}", stats.b_hbp));
                                        ui.label(format!("{}", stats.b_so));
                                        ui.label(format!("{}", stats.b_r));
                                        ui.label(format!("{}", stats.b_rbi));
                                        ui.label(format!("{}.{:03}", stats.b_avg / 1000, stats.b_avg % 1000));
                                        ui.label(format!("{}.{:03}", stats.b_obp / 1000, stats.b_obp % 1000));
                                        ui.label(format!("{}.{:03}", stats.b_slg / 1000, stats.b_slg % 1000));
                                        ui.end_row();
                                    }
                                });
                                ui.heading("Pitching");
                                egui::Grid::new("pitching").striped(true).show(ui, |ui| {
                                    ui.label("Name");
                                    ui.label("G");
                                    ui.label("GS");
                                    ui.label("IP");
                                    ui.label("BF");
                                    ui.label("H");
                                    ui.label("2B");
                                    ui.label("3B");
                                    ui.label("HR");
                                    ui.label("BB");
                                    ui.label("HBP");
                                    ui.label("SO");
                                    ui.label("R");
                                    ui.label("ER");
                                    ui.label("ERA");
                                    ui.label("WHIP");
                                    ui.label("AVG");
                                    ui.label("OBP");
                                    ui.label("SLG");
                                    ui.end_row();


                                    for player_id in &team.players {
                                        let player = self.players.get(player_id).unwrap();
                                        if player.pos != Position::Pitcher {
                                            continue;
                                        }
                                        let stats = player.get_stats();

                                        if ui.add(Button::new(&player.fullname()).frame(false)).clicked() {
                                            mode = Mode::Player(Some(*id), *player_id);
                                        }
                                        ui.label(format!("{}", stats.g));
                                        ui.label(format!("{}", stats.gs));
                                        ui.label(format!("{}.{}", stats.p_o / 3, stats.p_o % 3));
                                        ui.label(format!("{}", stats.p_bf));
                                        ui.label(format!("{}", stats.p_h));
                                        ui.label(format!("{}", stats.p_2b));
                                        ui.label(format!("{}", stats.p_3b));
                                        ui.label(format!("{}", stats.p_hr));
                                        ui.label(format!("{}", stats.p_bb));
                                        ui.label(format!("{}", stats.p_hbp));
                                        ui.label(format!("{}", stats.p_so));
                                        ui.label(format!("{}", stats.p_r));
                                        ui.label(format!("{}", stats.p_er));
                                        ui.label(format!("{}.{:03}", stats.p_era / 1000, stats.p_era % 1000));
                                        ui.label(format!("{}.{:03}", stats.p_whip / 1000, stats.p_whip % 1000));
                                        ui.label(format!("{}.{:03}", stats.p_avg / 1000, stats.p_avg % 1000));
                                        ui.label(format!("{}.{:03}", stats.p_obp / 1000, stats.p_obp % 1000));
                                        ui.label(format!("{}.{:03}", stats.p_slg / 1000, stats.p_slg % 1000));
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

                    let player = self.players.get(player_id).unwrap();

                    if ui.button("Close").clicked() {
                        if let Some(team_id) = team_id {
                            mode = Mode::Team(*team_id);
                        } else if player.pos == Position::Pitcher {
                            mode = Mode::PitLeaders(Stat::Pera, false);
                        } else {
                            mode = Mode::BatLeaders(Stat::Bhr, true);
                        }
                    }
                    ui.label(format!("Name: {}", player.fullname()));
                    ui.label(format!("Age: {}", player.age));
                    ui.label(format!("Pos: {}", player.pos.to_str()));
                    ui.label(format!("Bats: {}", player.bats.to_str()));
                    ui.label(format!("Throws: {}", player.throws.to_str()));

                    ui.heading("History");
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
                        ui.label("SO");
                        ui.label("R");
                        ui.label("RBI");
                        ui.label("AVG");
                        ui.label("OBP");
                        ui.label("SLG");
                        ui.end_row();

                        for history in &player.historical {
                            let stats = history.get_stats();

                            ui.label(format!("{}", history.year));
                            ui.label(format!("{}", history.league));
                            ui.label(format!("{}", history.team));
                            ui.label(format!("{}", stats.b_pa));
                            ui.label(format!("{}", stats.b_ab));
                            ui.label(format!("{}", stats.b_h));
                            ui.label(format!("{}", stats.b_2b));
                            ui.label(format!("{}", stats.b_3b));
                            ui.label(format!("{}", stats.b_hr));
                            ui.label(format!("{}", stats.b_bb));
                            ui.label(format!("{}", stats.b_hbp));
                            ui.label(format!("{}", stats.b_so));
                            ui.label(format!("{}", stats.b_r));
                            ui.label(format!("{}", stats.b_rbi));
                            ui.label(format!("{}.{:03}", stats.b_avg / 1000, stats.b_avg % 1000));
                            ui.label(format!("{}.{:03}", stats.b_obp / 1000, stats.b_obp % 1000));
                            ui.label(format!("{}.{:03}", stats.b_slg / 1000, stats.b_slg % 1000));
                            ui.end_row();
                        }
                    });

                    mode
                }
                Mode::BatLeaders(result, reverse) => {
                    let mut mode = Mode::BatLeaders(*result, *reverse);

                    egui::Grid::new("bleaders").striped(true).show(ui, |ui| {
                        ui.label("#");
                        ui.label("Name");
                        ui.label("Team");
                        if ui.button("G").clicked() {
                            mode = select_bat_stat(Stat::G, *result, *reverse, true);
                        }
                        if ui.button("GS").clicked() {
                            mode = select_bat_stat(Stat::Gs, *result, *reverse, true);
                        }
                        if ui.button("PA").clicked() {
                            mode = select_bat_stat(Stat::Bpa, *result, *reverse, true);
                        }
                        if ui.button("AB").clicked() {
                            mode = select_bat_stat(Stat::Bab, *result, *reverse, true);
                        }
                        if ui.button("H").clicked() {
                            mode = select_bat_stat(Stat::Bh, *result, *reverse, true);
                        }
                        if ui.button("2B").clicked() {
                            mode = select_bat_stat(Stat::B2b, *result, *reverse, true);
                        }
                        if ui.button("3B").clicked() {
                            mode = select_bat_stat(Stat::B3b, *result, *reverse, true);
                        }
                        if ui.button("HR").clicked() {
                            mode = select_bat_stat(Stat::Bhr, *result, *reverse, true);
                        }
                        if ui.button("BB").clicked() {
                            mode = select_bat_stat(Stat::Bbb, *result, *reverse, true);
                        }
                        if ui.button("HBP").clicked() {
                            mode = select_bat_stat(Stat::Bhbp, *result, *reverse, true);
                        }
                        if ui.button("SO").clicked() {
                            mode = select_bat_stat(Stat::Bso, *result, *reverse, true);
                        }
                        if ui.button("R").clicked() {
                            mode = select_bat_stat(Stat::Br, *result, *reverse, true);
                        }
                        if ui.button("RBI").clicked() {
                            mode = select_bat_stat(Stat::Brbi, *result, *reverse, true);
                        }
                        if ui.button("AVG").clicked() {
                            mode = select_bat_stat(Stat::Bavg, *result, *reverse, true);
                        }
                        if ui.button("OBP").clicked() {
                            mode = select_bat_stat(Stat::Bobp, *result, *reverse, true);
                        }
                        if ui.button("SLG").clicked() {
                            mode = select_bat_stat(Stat::Bslg, *result, *reverse, true);
                        }
                        ui.end_row();

                        let mut all_players = Vec::new();

                        for team_id in league.teams.iter() {
                            let team = &self.teams.get(team_id).unwrap();
                            for player_id in team.players.iter() {
                                let player = self.players.get(player_id).unwrap();
                                if player.pos != Position::Pitcher {
                                    all_players.push((&team.abbr, player, player.get_stats(), player_id));
                                }
                            }
                        }

                        all_players.sort_by_key(|o| o.2.get_stat(*result));
                        if *reverse {
                            all_players.reverse()
                        };

                        for (rank, ap) in all_players[0..30].iter().enumerate() {
                            let player = ap.1;

                            ui.label(format!("{}", rank + 1));
                            if ui.add(Button::new(player.fullname()).frame(false)).clicked() {
                                mode = Mode::Player(None, *ap.3);
                            }
                            ui.label(ap.0);


                            let stats = &ap.2;

                            ui.label(format!("{}", stats.g));
                            ui.label(format!("{}", stats.gs));
                            ui.label(format!("{}", stats.b_pa));
                            ui.label(format!("{}", stats.b_ab));
                            ui.label(format!("{}", stats.b_h));
                            ui.label(format!("{}", stats.b_2b));
                            ui.label(format!("{}", stats.b_3b));
                            ui.label(format!("{}", stats.b_hr));
                            ui.label(format!("{}", stats.b_bb));
                            ui.label(format!("{}", stats.b_hbp));
                            ui.label(format!("{}", stats.b_so));
                            ui.label(format!("{}", stats.b_r));
                            ui.label(format!("{}", stats.b_rbi));
                            ui.label(format!("{}.{:03}", stats.b_avg / 1000, stats.b_avg % 1000));
                            ui.label(format!("{}.{:03}", stats.b_obp / 1000, stats.b_obp % 1000));
                            ui.label(format!("{}.{:03}", stats.b_slg / 1000, stats.b_slg % 1000));
                            ui.end_row();
                        }
                    });

                    mode
                }
                Mode::PitLeaders(result, reverse) => {
                    let mut mode = Mode::PitLeaders(*result, *reverse);

                    egui::Grid::new("pleaders").striped(true).show(ui, |ui| {
                        ui.label("#");
                        ui.label("Name");
                        ui.label("Team");
                        if ui.button("G").clicked() {
                            mode = select_pit_stat(Stat::G, *result, *reverse, true);
                        }
                        if ui.button("GS").clicked() {
                            mode = select_pit_stat(Stat::Gs, *result, *reverse, true);
                        }
                        if ui.button("IP").clicked() {
                            mode = select_pit_stat(Stat::Po, *result, *reverse, true);
                        }
                        if ui.button("BF").clicked() {
                            mode = select_pit_stat(Stat::Pbf, *result, *reverse, true);
                        }
                        if ui.button("H").clicked() {
                            mode = select_pit_stat(Stat::Ph, *result, *reverse, true);
                        }
                        if ui.button("2B").clicked() {
                            mode = select_pit_stat(Stat::P2b, *result, *reverse, true);
                        }
                        if ui.button("3B").clicked() {
                            mode = select_pit_stat(Stat::P3b, *result, *reverse, true);
                        }
                        if ui.button("HR").clicked() {
                            mode = select_pit_stat(Stat::Phr, *result, *reverse, true);
                        }
                        if ui.button("BB").clicked() {
                            mode = select_pit_stat(Stat::Pbb, *result, *reverse, true);
                        }
                        if ui.button("HBP").clicked() {
                            mode = select_pit_stat(Stat::Phbp, *result, *reverse, true);
                        }
                        if ui.button("SO").clicked() {
                            mode = select_pit_stat(Stat::Pso, *result, *reverse, true);
                        }
                        if ui.button("R").clicked() {
                            mode = select_pit_stat(Stat::Pr, *result, *reverse, true);
                        }
                        if ui.button("ER").clicked() {
                            mode = select_pit_stat(Stat::Per, *result, *reverse, true);
                        }
                        if ui.button("ERA").clicked() {
                            mode = select_pit_stat(Stat::Pera, *result, *reverse, false);
                        }
                        if ui.button("WHIP").clicked() {
                            mode = select_pit_stat(Stat::Pwhip, *result, *reverse, false);
                        }
                        if ui.button("AVG").clicked() {
                            mode = select_pit_stat(Stat::Pavg, *result, *reverse, false);
                        }
                        if ui.button("OBP").clicked() {
                            mode = select_pit_stat(Stat::Pobp, *result, *reverse, false);
                        }
                        if ui.button("SLG").clicked() {
                            mode = select_pit_stat(Stat::Pslg, *result, *reverse, false);
                        }
                        ui.end_row();

                        let mut all_players = Vec::new();

                        for team_id in league.teams.iter() {
                            let team = &self.teams.get(team_id).unwrap();
                            for player_id in team.players.iter() {
                                let player = self.players.get(player_id).unwrap();
                                if player.pos == Position::Pitcher {
                                    all_players.push((&team.abbr, player, player.get_stats(), player_id));
                                }
                            }
                        }

                        all_players.sort_by_key(|o| o.2.get_stat(*result));
                        if *reverse {
                            all_players.reverse()
                        };

                        for (rank, ap) in all_players[0..30].iter().enumerate() {
                            let player = ap.1;

                            ui.label(format!("{}", rank + 1));
                            if ui.add(Button::new(player.fullname()).frame(false)).clicked() {
                                mode = Mode::Player(None, *ap.3);
                            }
                            ui.label(ap.0);


                            let stats = &ap.2;

                            ui.label(format!("{}", stats.g));
                            ui.label(format!("{}", stats.gs));
                            ui.label(format!("{}.{}", stats.p_o / 3, stats.p_o % 3));
                            ui.label(format!("{}", stats.p_bf));
                            ui.label(format!("{}", stats.p_h));
                            ui.label(format!("{}", stats.p_2b));
                            ui.label(format!("{}", stats.p_3b));
                            ui.label(format!("{}", stats.p_hr));
                            ui.label(format!("{}", stats.p_bb));
                            ui.label(format!("{}", stats.p_hbp));
                            ui.label(format!("{}", stats.p_so));
                            ui.label(format!("{}", stats.p_r));
                            ui.label(format!("{}", stats.p_er));
                            ui.label(format!("{}.{:03}", stats.p_era / 1000, stats.p_era % 1000));
                            ui.label(format!("{}.{:03}", stats.p_whip / 1000, stats.p_whip % 1000));
                            ui.label(format!("{}.{:03}", stats.p_avg / 1000, stats.p_avg % 1000));
                            ui.label(format!("{}.{:03}", stats.p_obp / 1000, stats.p_obp % 1000));
                            ui.label(format!("{}.{:03}", stats.p_slg / 1000, stats.p_slg % 1000));
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
