use std::collections::{HashMap, HashSet};

use eframe::{egui, epi};
use eframe::egui::{Button, ScrollArea, Ui};
use ordinal::Ordinal;
use rand::rngs::ThreadRng;

use crate::data::Data;
use crate::league::{end_of_season, League};
use crate::player::{collect_all_active, generate_players, PlayerId, PlayerMap};
use crate::schedule::{Game, GameLogEvent, Scoreboard};
use crate::stat::{Stat, Stats, HistoricalStats};
use crate::team::{Team, TeamId, TeamMap};

#[derive(Copy, Clone, PartialEq)]
enum Mode {
    Schedule(usize),
    BoxScore(usize, usize),
    GameLog(usize, usize),
    Standings(usize),
    Team(usize, TeamId),
    Player(usize, PlayerId, Option<TeamId>),
    BatLeaders(usize, Stat, bool),
    PitLeaders(usize, Stat, bool),
    LeagueRecords(usize),
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct Imp019App {
    rng: ThreadRng,
    data: Data,
    players: PlayerMap,
    teams: TeamMap,
    leagues: Vec<League>,
    year: u32,
    disp_mode: Mode,
    sim_all: bool,
}

impl Default for Imp019App {
    fn default() -> Self {
        Imp019App {
            rng: rand::thread_rng(),
            data: Data::new(),
            players: HashMap::new(),
            teams: HashMap::new(),
            leagues: Vec::new(),
            year: 2030,
            disp_mode: Mode::Schedule(0),
            sim_all: false,
        }
    }
}

impl Imp019App {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let data = Data::new();
        let year = 2049;

        let mut players = HashMap::new();
        generate_players(&mut players, 3600, year, &data, &mut rng);

        let mut available = collect_all_active(&players);

        let locs = data.get_locs(&mut HashSet::new(), &mut rng, 60);
        let nicks = data.get_nicks(&mut HashSet::new(), &mut rng, 60);

        let mut teams = HashMap::new();
        teams.reserve(60);
        for team_id in 0..60 {
            let (abbr, city, state) = locs[team_id].clone();
            let nick = nicks[team_id].clone();
            let mut team = Team::new(abbr, city, state, nick, year);

            team.populate(&mut available, &players);

            let team_id = (team_id + 1) as TeamId;
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
            data,
            players,
            teams,
            leagues,
            year,
            disp_mode: Mode::Schedule(0),
            sim_all: false,
        }
    }

    pub fn update(&mut self) -> bool {
        let mut result = false;
        for league in &mut self.leagues {
            result = league.sim(&mut self.teams, &mut self.players, self.year, &mut self.rng) || result;
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

fn display_game(ui: &mut Ui, game: &Game, teams: &TeamMap) -> bool {
    let home_team = teams.get(&game.home.id).unwrap();
    let away_team = teams.get(&game.away.id).unwrap();

    let complete = game.away.r != game.home.r;

    let mut clicked = false;

    ui.group(|ui| {
        ui.vertical(|ui| {
            if complete {
                ui.horizontal(|ui| {
                    ui.monospace("   ");
                    ui.monospace("  R");
                    ui.monospace("  H");
                    ui.monospace("  E");
                });
            }
            ui.horizontal(|ui| {
                if complete {
                    ui.monospace(&away_team.abbr);
                    ui.monospace(format!("{:3}", game.away.r));
                    ui.monospace(format!("{:3}", game.away.h));
                    ui.monospace(format!("{:3}", game.away.e));
                } else {
                    ui.monospace(format!("  {}", away_team.abbr));
                }
            });
            ui.horizontal(|ui| {
                if complete {
                    ui.monospace(&home_team.abbr);
                    ui.monospace(format!("{:3}", game.home.r));
                    ui.monospace(format!("{:3}", game.home.h));
                    ui.monospace(format!("{:3}", game.home.e));
                } else {
                    ui.monospace(format!("@ {}", home_team.abbr));
                }
            });
            clicked = complete && ui.button("Box Score").clicked();
        });
    });

    clicked
}

fn display_bo(ui: &mut Ui, scoreboard: &Scoreboard, team: &Team, players: &PlayerMap, stat_map: &HashMap<PlayerId, Vec<Stat>>) {
    ui.label(format!("{} {} Batters", team.abbr, team.nickname));

    const HEADERS: [Stat; 6] = [
        Stat::Bab,
        Stat::Br,
        Stat::Bh,
        Stat::Brbi,
        Stat::Bbb,
        Stat::Bso,
    ];

    for header in HEADERS.iter() {
        ui.monospace(header.to_string());
    }
    ui.monospace(Stat::Bavg.to_string());
    ui.monospace("OPS");
    ui.end_row();

    for (idx, def) in scoreboard.bo.iter().enumerate() {
        let batter = players.get(&def.player).unwrap();

        let stats = Stats::compile_stats(stat_map.get(&def.player).unwrap_or(&Vec::new()));
        let full_stats = batter.get_stats();

        ui.label(format!("{}. {} {}", idx + 1, batter.fname(), def.pos));

        for header in HEADERS.iter() {
            ui.monospace(header.value(stats.get_stat(*header)).to_string());
        }
        ui.monospace(Stat::Bavg.value(full_stats.get_stat(Stat::Bavg)).to_string());
        let ops = full_stats.b_obp + full_stats.b_slg;
        ui.monospace(Stat::Bobp.value(ops));
        ui.end_row();
    }
}

fn display_pitching(ui: &mut Ui, scoreboard: &Scoreboard, team: &Team, players: &PlayerMap, stat_map: &HashMap<PlayerId, Vec<Stat>>) {
    ui.label(format!("{} {} Pitchers", team.abbr, team.nickname));

    const HEADERS: [Stat; 7] = [
        Stat::Po,
        Stat::Ph,
        Stat::Pr,
        Stat::Per,
        Stat::Pbb,
        Stat::Pso,
        Stat::Phr,
    ];

    for header in HEADERS.iter() {
        ui.monospace(header.to_string());
    }
    ui.monospace(Stat::Pera.to_string());
    ui.end_row();

    for rec in scoreboard.pitcher_record.iter() {
        let pitcher = players.get(&rec.pitcher).unwrap();

        let stats = Stats::compile_stats(stat_map.get(&rec.pitcher).unwrap_or(&Vec::new()));
        let full_stats = pitcher.get_stats();

        ui.label(pitcher.fname());
        for header in HEADERS.iter() {
            ui.monospace(header.value(stats.get_stat(*header)).to_string());
        }

        ui.monospace(Stat::Pera.value(full_stats.p_era));
        ui.end_row();
    }
}

fn for_each_event<T>(game: &Game, mut action: T) where T: FnMut(usize, bool, &GameLogEvent, bool) {
    let mut inning = 1;
    let mut tophalf = true;
    let mut outs = 0;

    let mut error = false;
    for event in game.playbyplay.iter() {
        action(inning, tophalf, event, error);

        if event.event == Stat::Fe {
            error = true;
        }

        if event.event == Stat::Bo || event.event == Stat::Bso {
            if !error {
                outs += 1;
            }
            error = false;
            if outs == 3 {
                if !tophalf {
                    inning += 1;
                }
                tophalf = !tophalf;
                outs = 0;
            }
        }
    }
}

fn display_team_stats(ui: &mut Ui, is_batter: bool, headers: &[Stat], team_players: &[PlayerId], players: &PlayerMap) -> Option<PlayerId> {
    ui.label("Name");
    ui.label("Pos");


    for header in headers {
        ui.label(header.to_string());
    }
    ui.end_row();


    let mut ret = None;
    for player_id in team_players {
        let player = players.get(player_id).unwrap();
        if player.pos.is_pitcher() == is_batter {
            continue;
        }
        let stats = player.get_stats();

        if ui.add(Button::new(&player.fullname()).frame(false)).clicked() {
            ret = Some(*player_id);
        }
        ui.label(player.pos.to_string());

        for header in headers {
            ui.label(header.value(stats.get_stat(*header)));
        }
        ui.end_row();
    }

    ret
}

fn display_historical_stats(ui: &mut Ui, headers: &[Stat], historical: &[HistoricalStats], teams: &TeamMap ) {
    ui.label("Year");
    ui.label("League");
    ui.label("Team");
    for header in headers {
        ui.label(header.to_string());
    }
    ui.end_row();

    for history in historical {
        let stats = history.get_stats();
        let team = teams.get(&history.team).unwrap();

        ui.label(format!("{}", history.year));
        ui.label(format!("{}", history.league));
        ui.label(&team.abbr);
        for header in headers {
            ui.label(header.value(stats.get_stat(*header)));
        }
        ui.end_row();
    }
}

fn display_leaders(ui: &mut Ui, is_batter: bool, headers: &[Stat], league: &League, teams: &TeamMap, players: &PlayerMap, mut mode: Mode) -> Mode {

    let (disp_league,result, reverse) = match mode {
        Mode::BatLeaders(disp_league,result,reverse) => (disp_league,result,reverse),
        Mode::PitLeaders(disp_league,result,reverse) => (disp_league,result,reverse),
        _ => panic!(),
    };

    ui.label("#");
    ui.label("Name");
    ui.label("Team");
    ui.label("Pos");

    for header in headers {
        if ui.button(header.to_string()).clicked() {
            let flip = if *header == result { !reverse } else { !header.is_reverse_sort() };
            mode = match mode {
                Mode::BatLeaders(disp_league,_,_) => Mode::BatLeaders(disp_league,*header,flip),
                Mode::PitLeaders(disp_league,_,_) => Mode::BatLeaders(disp_league,*header,flip),
                _ => panic!(),
            }
        }
    }

    ui.end_row();

    let mut all_players = Vec::new();

    for team_id in &league.teams {
        let team = &teams.get(team_id).unwrap();
        for player_id in &team.players {
            let player = players.get(player_id).unwrap();
            if player.pos.is_pitcher() != is_batter {
                all_players.push((&team.abbr, player, player.get_stats(), player_id));
            }
        }
    }

    all_players.sort_by_key(|o| o.2.get_stat(result));
    if reverse {
        all_players.reverse()
    };

    for (rank, ap) in all_players[0..30].iter().enumerate() {
        let player = ap.1;

        ui.label(format!("{}", rank + 1));
        if ui.add(Button::new(player.fullname()).frame(false)).clicked() {
            mode = Mode::Player(disp_league, *ap.3, None);
        }
        ui.label(ap.0);
        ui.label(ap.1.pos.to_string());

        let stats = &ap.2;

        for header in headers {
            ui.label(header.value(stats.get_stat(*header)));
        }
        ui.end_row();
    }
    mode
}

const BATTING_HEADERS: [Stat; 16] = [
    Stat::G,
    Stat::Gs,
    Stat::Bpa,
    Stat::Bab,
    Stat::Bh,
    Stat::B2b,
    Stat::B3b,
    Stat::Bhr,
    Stat::Bbb,
    Stat::Bhbp,
    Stat::Bso,
    Stat::Br,
    Stat::Brbi,
    Stat::Bavg,
    Stat::Bobp,
    Stat::Bslg,
];

const PITCHING_HEADERS: [Stat; 23] = [
    Stat::G,
    Stat::Pw,
    Stat::Pl,
    Stat::Psv,
    Stat::Phld,
    Stat::Pcg,
    Stat::Psho,
    Stat::Po,
    Stat::Pbf,
    Stat::Ph,
    Stat::P2b,
    Stat::P3b,
    Stat::Phr,
    Stat::Pbb,
    Stat::Phbp,
    Stat::Pso,
    Stat::Pr,
    Stat::Per,
    Stat::Pera,
    Stat::Pwhip,
    Stat::Pavg,
    Stat::Pobp,
    Stat::Pslg,
];



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
                        end_of_season(&mut self.leagues, &mut self.teams, &mut self.players, 4, self.year, &self.data, &mut self.rng);
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
            for league_idx in 0..self.leagues.len() {
                ui.horizontal(|ui| {
                    ui.label(format!("League {}", league_idx + 1));
                    if ui.button("Sche").clicked() {
                        self.disp_mode = Mode::Schedule(league_idx);
                    }
                    if ui.button("Stan").clicked() {
                        self.disp_mode = Mode::Standings(league_idx);
                    }
                    if ui.button("Bat").clicked() {
                        self.disp_mode = Mode::BatLeaders(league_idx, Stat::Bhr, true);
                    }
                    if ui.button("Pit").clicked() {
                        self.disp_mode = Mode::PitLeaders(league_idx, Stat::Pw, true);
                    }
                    if ui.button("Rec").clicked() {
                        self.disp_mode = Mode::LeagueRecords(league_idx);
                    }
                });
            }
            ui.separator();
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.disp_mode = match &self.disp_mode {
                Mode::Schedule(disp_league) => {
                    let league = &self.leagues[*disp_league];
                    let total_games = league.schedule.games.len();

                    let cur_idx = league.schedule.games.iter().position(|o| o.home.r == o.away.r).unwrap_or(total_games);
                    let teams = league.teams.len();

                    let half_teams = teams / 2;

                    if cur_idx < total_games {
                        ui.heading(format!("Today ({})", cur_idx / half_teams));
                        ui.group(|ui| {
                            ui.horizontal_wrapped(|ui| {
                                for idx in cur_idx..(cur_idx + half_teams) {
                                    let game = &league.schedule.games[idx];
                                    display_game(ui, game, &self.teams);
                                }
                            });
                        });
                    }

                    let mut mode = Mode::Schedule(*disp_league);
                    if cur_idx > 0 {
                        ui.heading("Yesterday");
                        let end = cur_idx as i32;
                        let start = end - (half_teams as i32);
                        ui.group(|ui| {
                            ui.horizontal_wrapped(|ui| {
                                for past_idx in start..end {
                                    if past_idx >= 0 {
                                        let game = &league.schedule.games[past_idx as usize];
                                        if display_game(ui, game, &self.teams) {
                                            mode = Mode::BoxScore(*disp_league, past_idx as usize)
                                        }
                                        if ((past_idx - start + 1) % 5) == 0 {
                                            ui.end_row();
                                        }
                                    }
                                }
                            });
                        });
                    }
                    mode
                }
                Mode::BoxScore(disp_league, game_idx) => {
                    let league = &self.leagues[*disp_league];
                    let mut mode = Mode::BoxScore(*disp_league, *game_idx);
                    let game = &league.schedule.games[*game_idx];

                    let awayteam = self.teams.get(&game.away.id).unwrap();
                    let hometeam = self.teams.get(&game.home.id).unwrap();

                    let mut awayruns = Vec::new();
                    let mut homeruns = Vec::new();

                    ui.horizontal(|ui| {
                        if ui.button("Back").clicked() {
                            mode = Mode::Schedule(*disp_league);
                        }
                        if ui.button("Game Log").clicked() {
                            mode = Mode::GameLog(*disp_league, *game_idx);
                        }
                    });


                    let mut winner = None;
                    let mut loser = None;
                    let mut save = None;

                    let mut stat_map = HashMap::new();

                    for_each_event(game, |inning, tophalf, event, _| {
                        let player_stats = stat_map.entry(event.player).or_insert_with(Vec::new);
                        player_stats.push(event.event);

                        match event.event {
                            Stat::Pw => winner = Some(event.player),
                            Stat::Pl => loser = Some(event.player),
                            Stat::Psv => save = Some(event.player),
                            _ => {}
                        };

                        let skip = matches!(event.event, Stat::Pw|Stat::Pcg|Stat::Psho|Stat::Pl|Stat::Psv|Stat::Phld|Stat::Po|Stat::Pso);

                        if !skip {
                            let runs = if tophalf { &mut awayruns } else { &mut homeruns };
                            if runs.len() < inning {
                                runs.push(0);
                            }
                            if event.event == Stat::Br {
                                runs[inning - 1] += 1;
                            }
                        }
                    });

                    egui::Grid::new("Innings").show(ui, |ui| {
                        ui.monospace("   ");
                        for inning in 1..=awayruns.len().max(homeruns.len()) {
                            ui.monospace(format!("{}", inning));
                        }
                        ui.monospace("  R");
                        ui.monospace("  H");
                        ui.monospace("  E");
                        ui.end_row();
                        ui.monospace(&awayteam.abbr);
                        for awayrun in awayruns.iter() {
                            ui.monospace(format!("{}", awayrun));
                        }
                        ui.monospace(format!("{:3}", game.away.r));
                        ui.monospace(format!("{:3}", game.away.h));
                        ui.monospace(format!("{:3}", game.away.e));
                        ui.end_row();
                        ui.monospace(&hometeam.abbr);
                        for homerun in homeruns.iter() {
                            ui.monospace(format!("{}", homerun));
                        }
                        if awayruns.len() > homeruns.len() {
                            ui.monospace("X");
                        }
                        ui.monospace(format!("{:3}", game.home.r));
                        ui.monospace(format!("{:3}", game.home.h));
                        ui.monospace(format!("{:3}", game.home.e));
                        ui.end_row();
                    });

                    ui.horizontal(|ui| {
                        if let Some(w) = winner {
                            let pitcher = self.players.get(&w).unwrap();
                            ui.label(format!("W: {}", pitcher.fname()));
                        }
                        if let Some(l) = loser {
                            let pitcher = self.players.get(&l).unwrap();
                            ui.label(format!("L: {}", pitcher.fname()));
                        }
                        if let Some(sv) = save {
                            let pitcher = self.players.get(&sv).unwrap();
                            ui.label(format!("SV: {}", pitcher.fname()));
                        }
                    });

                    ui.separator();

                    ui.columns(2, |cols| {
                        for (i, col) in cols.iter_mut().enumerate() {
                            match i {
                                0 => {
                                    egui::Grid::new("Away Batting").show(col, |ui| {
                                        display_bo(ui, &game.away, awayteam, &self.players, &stat_map);
                                    });
                                }
                                1 => {
                                    egui::Grid::new("Home Batting").show(col, |ui| {
                                        display_bo(ui, &game.home, hometeam, &self.players, &stat_map);
                                    });
                                }
                                _ => {}
                            }
                        }
                    });

                    ui.separator();

                    ui.columns(2, |cols| {
                        for (i, col) in cols.iter_mut().enumerate() {
                            match i {
                                0 => {
                                    egui::Grid::new("Away Pitching").show(col, |ui| {
                                        display_pitching(ui, &game.away, awayteam, &self.players, &stat_map);
                                    });
                                }
                                1 => {
                                    egui::Grid::new("Home Pitching").show(col, |ui| {
                                        display_pitching(ui, &game.home, hometeam, &self.players, &stat_map);
                                    });
                                }
                                _ => {}
                            }
                        }
                    });

                    ui.separator();

                    mode
                }
                Mode::GameLog(disp_league, game_idx) => {
                    let league = &self.leagues[*disp_league];
                    let mut mode = Mode::GameLog(*disp_league, *game_idx);
                    let game = &league.schedule.games[*game_idx];

                    if ui.button("Box Score").clicked() {
                        mode = Mode::BoxScore(*disp_league, *game_idx);
                    }

                    ScrollArea::auto_sized().show(ui, |ui| {
                        let mut prevhalf = false;
                        let mut previnn = 0;

                        for_each_event(game, |inning, tophalf, event, error| {
                            let player = self.players.get(&event.player).unwrap();
                            let player_str = player.fullname();

                            let pitching_change = event.event == Stat::G && player.pos.is_pitcher();

                            if !pitching_change && (!event.event.is_batting() || event.event == Stat::Brbi) {
                                return;
                            }

                            if prevhalf != tophalf || previnn != inning {
                                ui.heading(format!("{} of the {}", if tophalf { "Top" } else { "Bottom" }, Ordinal(inning)));
                                prevhalf = tophalf;
                                previnn = inning;
                            }


                            if pitching_change {
                                ui.label(format!("{} is now pitching.", player_str));
                                return;
                            }


                            let target_str = if let Some(target) = event.target {
                                format!(" to {}", target)
                            } else {
                                "".to_string()
                            };

                            let label_str = match event.event {
                                Stat::B1b => format!("{} singles{}.", player_str, target_str),
                                Stat::B2b => format!("{} doubles{}.", player_str, target_str),
                                Stat::B3b => format!("{} triples{}.", player_str, target_str),
                                Stat::Bhr => format!("{} homers{}.", player_str, target_str),
                                Stat::Bbb => format!("{} walks.", player_str),
                                Stat::Bhbp => format!("{} is hit by pitch.", player_str),
                                Stat::Bso => format!("{} strikes out.", player_str),
                                Stat::Bo => if error {
                                    format!("{} reaches on error{}.", player_str, target_str)
                                } else {
                                    format!("{} flies out{}.", player_str, target_str)
                                },
                                Stat::Br => format!("{} scores.", player_str),
                                _ => "".to_string()
                            };

                            ui.label(label_str);
                        });
                    });

                    mode
                }
                Mode::Standings(disp_league) => {
                    let league = &self.leagues[*disp_league];
                    let mut mode = Mode::Standings(*disp_league);
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
                                mode = Mode::Team(*disp_league, **team_id);
                            }
                            ui.label(format!("{}-{}", team.get_wins(), team.get_losses()));
                            ui.end_row();
                            rank += 1;
                        }
                    });
                    mode
                }
                Mode::Team(disp_league, id) => {
                    let mut mode = Mode::Team(*disp_league, *id);
                    if ui.button("Close").clicked() {
                        mode = Mode::Standings(*disp_league);
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

                                egui::Grid::new("batting").striped(true).show(ui, |mut ui| {
                                    if let Some(player_id) = display_team_stats(&mut ui, true, &BATTING_HEADERS, &team.players, &self.players) {
                                        mode = Mode::Player(*disp_league, player_id, Some(*id));
                                    }
                                });
                                ui.heading("Pitching");
                                egui::Grid::new("pitching").striped(true).show(ui, |mut ui| {
                                    if let Some(player_id) = display_team_stats(&mut ui, false, &PITCHING_HEADERS, &team.players, &self.players) {
                                        mode = Mode::Player(*disp_league, player_id, Some(*id));
                                    }
                                });
                            });
                        }
                    });


                    mode
                }
                Mode::Player(disp_league, player_id, team_id) => {
                    let mut mode = Mode::Player(*disp_league, *player_id, *team_id);

                    let player = self.players.get(player_id).unwrap();

                    if ui.button("Close").clicked() {
                        if let Some(team_id) = team_id {
                            mode = Mode::Team(*disp_league, *team_id);
                        } else if player.pos.is_pitcher() {
                            mode = Mode::PitLeaders(*disp_league, Stat::Pw, true);
                        } else {
                            mode = Mode::BatLeaders(*disp_league, Stat::Bhr, true);
                        }
                    }
                    ui.label(format!("Name: {}", player.fullname()));
                    ui.label(format!("Age: {} Born: {}", player.age(self.year), player.born));
                    ui.label(format!("Pos: {}", player.pos));
                    ui.label(format!("Bats: {}", player.bats));
                    ui.label(format!("Throws: {}", player.throws));

                    if !player.pos.is_pitcher() {
                        ui.heading("Batting History");
                        egui::Grid::new("bhistory").striped(true).show(ui, |mut ui| {
                            display_historical_stats( &mut ui, &BATTING_HEADERS, &player.historical, &self.teams );
                        });
                    } else {
                        ui.heading("Pitching History");
                        egui::Grid::new("phistory").striped(true).show(ui, |mut ui| {
                            display_historical_stats( &mut ui, &PITCHING_HEADERS, &player.historical, &self.teams );
                        });
                    }

                    mode
                }
                Mode::BatLeaders(disp_league, result, reverse) => {
                    let league = &self.leagues[*disp_league];
                    let mut mode = Mode::BatLeaders(*disp_league, *result, *reverse);

                    egui::Grid::new("bleaders").striped(true).show(ui, |ui| {
                        mode = display_leaders(ui, true, &BATTING_HEADERS, league, &self.teams, &self.players, mode);
                    });

                    mode
                }
                Mode::PitLeaders(disp_league, result, reverse) => {
                    let league = &self.leagues[*disp_league];
                    let mut mode = Mode::PitLeaders(*disp_league, *result, *reverse);

                    egui::Grid::new("pleaders").striped(true).show(ui, |ui| {
                        mode = display_leaders(ui, false, &PITCHING_HEADERS, league, &self.teams, &self.players, mode);
                    });

                    mode
                }
                Mode::LeagueRecords(disp_league) => {
                    let league = &self.leagues[*disp_league];
                    for (stat, entry) in &league.records {
                        if let Some(record) = entry {
                            ui.horizontal(|ui| {
                                let team = self.teams.get(&record.team_id).unwrap();
                                let player = self.players.get(&record.player_id).unwrap();
                                ui.label(format!("{}: {} ({}) {} ({})", stat, player.fullname(), team.abbr, stat.value(record.record), record.year));
                            });
                        }
                    }

                    Mode::LeagueRecords(*disp_league)
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

    fn max_size_points(&self) -> egui::Vec2 {
        // Some browsers get slow with huge WebGL canvases, so we limit the size:
        egui::Vec2::new(2048.0, 1024.0)
    }
}
