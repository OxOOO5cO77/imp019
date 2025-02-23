use std::collections::{HashMap, HashSet};

use eframe::{App, egui, Frame};
use eframe::egui::{Button, ScrollArea, Ui};
use ordinal::Ordinal;
use rand::rngs::ThreadRng;

use crate::data::Data;
use crate::game::{Game, GameLogEvent, Scoreboard};
use crate::league::{end_of_season, League, RECORD_STATS};
use crate::player::{collect_all_active, generate_players, PlayerId, PlayerMap};
use crate::stat::{HistoricalStats, Stat, Stats};
use crate::team::{Team, TeamId, TeamMap};

#[derive(Copy, Clone, PartialEq)]
enum Mode {
    Schedule(usize, Option<usize>),
    BoxScore(usize, usize),
    GameLog(usize, usize),
    Standings(usize),
    Team(usize, TeamId),
    Player(usize, PlayerId, Option<TeamId>),
    BatLeaders(usize, Stat, bool),
    PitLeaders(usize, Stat, bool),
    LeagueRecords(usize),
}

/// We derive Deserialize/Serialize, so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct Imp019App {
    rng: ThreadRng,
    data: Data,
    player_map: PlayerMap,
    team_map: TeamMap,
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
            player_map: HashMap::new(),
            team_map: HashMap::new(),
            leagues: Vec::new(),
            year: 2030,
            disp_mode: Mode::Schedule(0, None),
            sim_all: false,
        }
    }
}

impl Imp019App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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
            let loc = locs[team_id].clone();
            let nick = nicks[team_id].clone();
            let mut team = Team::new(loc, nick, year);

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
            player_map: players,
            team_map: teams,
            leagues,
            year,
            disp_mode: Mode::Schedule(0, None),
            sim_all: false,
        }
    }

    pub fn update(&mut self) -> bool {
        let mut result = false;
        for league in &mut self.leagues {
            result = league.sim(&mut self.team_map, &mut self.player_map, self.year, &mut self.rng) || result;
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
                    ui.monospace(away_team.abbr());
                    ui.monospace(format!("{:3}", game.away.r));
                    ui.monospace(format!("{:3}", game.away.h));
                    ui.monospace(format!("{:3}", game.away.e));
                } else {
                    ui.monospace(format!("  {}", away_team.abbr()));
                }
            });
            ui.horizontal(|ui| {
                if complete {
                    ui.monospace(home_team.abbr());
                    ui.monospace(format!("{:3}", game.home.r));
                    ui.monospace(format!("{:3}", game.home.h));
                    ui.monospace(format!("{:3}", game.home.e));
                } else {
                    ui.monospace(format!("@ {}", home_team.abbr()));
                }
            });
            clicked = complete && ui.button("Box Score").clicked();
        });
    });

    clicked
}

fn display_bo(ui: &mut Ui, scoreboard: &Scoreboard, team: &Team, players: &PlayerMap, stat_map: &HashMap<PlayerId, Vec<Stat>>) {
    ui.label(format!("{} {} Batters", team.abbr(), team.nickname()));

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
    ui.label(format!("{} {} Pitchers", team.abbr(), team.nickname()));

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

        if event.event == Stat::Bgidp {
            outs += 1;  // add the second out below
        }

        if matches!( event.event, Stat::Bo | Stat::Bso | Stat::Bgidp | Stat::Bcs) {
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

fn display_historical_stat_row(ui: &mut Ui, headers: &[Stat], stats: &Stats, year: Option<u32>, league: Option<u32>, abbr: &str) {
    ui.label(year.map_or("CAREER".to_string(), |o| o.to_string()));
    ui.label(league.map_or(" ".to_string(), |o| o.to_string()));
    ui.label(abbr);
    for header in headers {
        ui.label(header.value(stats.get_stat(*header)));
    }
    ui.end_row();
}

fn display_historical_stats(ui: &mut Ui, headers: &[Stat], historical: &[HistoricalStats], teams: &TeamMap) -> Stats {
    ui.label("Year");
    ui.label("League");
    ui.label("Team");
    for header in headers {
        ui.label(header.to_string());
    }
    ui.end_row();

    let mut total = Stats::default();

    for history in historical {
        let stats = &history.stats;
        let team = teams.get(&history.team).unwrap();
        display_historical_stat_row(ui, headers, stats, Some(history.year), Some(history.league), team.abbr());
        total.compile(stats);
    }

    total
}

fn display_leaders(ui: &mut Ui, is_batter: bool, headers: &[Stat], league: &League, teams: &TeamMap, players: &PlayerMap, mut mode: Mode) -> Mode {
    let (disp_league, result, reverse) = match mode {
        Mode::BatLeaders(disp_league, result, reverse) => (disp_league, result, reverse),
        Mode::PitLeaders(disp_league, result, reverse) => (disp_league, result, reverse),
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
                Mode::BatLeaders(disp_league, _, _) => Mode::BatLeaders(disp_league, *header, flip),
                Mode::PitLeaders(disp_league, _, _) => Mode::PitLeaders(disp_league, *header, flip),
                _ => panic!(),
            }
        }
    }

    ui.end_row();

    let mut all_players = Vec::new();

    for team_id in &league.teams {
        let team = &teams.get(team_id).unwrap();
        let games = team.results.games();

        for player_id in &team.players {
            let player = players.get(player_id).unwrap();
            if player.pos.is_pitcher() != is_batter {
                let stats = player.get_stats();
                if result.is_qualified(&stats, games) {
                    all_players.push((team.abbr(), player, stats, player_id));
                }
            }
        }
    }

    all_players.sort_by_key(|o| o.2.get_stat(result));
    if reverse {
        all_players.reverse()
    };

    for (rank, ap) in all_players.iter().enumerate() {
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

const BATTING_HEADERS: [Stat; 20] = [
    Stat::G,
    Stat::Gs,
    Stat::Bpa,
    Stat::Bab,
    Stat::Bh,
    Stat::B2b,
    Stat::B3b,
    Stat::Bhr,
    Stat::Bbb,
    Stat::Bibb,
    Stat::Bhbp,
    Stat::Bso,
    Stat::Bgidp,
    Stat::Bsb,
    Stat::Bcs,
    Stat::Br,
    Stat::Brbi,
    Stat::Bavg,
    Stat::Bobp,
    Stat::Bslg,
];

const PITCHING_HEADERS: [Stat; 25] = [
    Stat::G,
    Stat::Pw,
    Stat::Pl,
    Stat::Psv,
    Stat::Pbs,
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
    Stat::Pibb,
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


impl App for Imp019App {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu_button(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.separator();
                if ui.button("Sim").clicked() {
                    let result = self.update();
                    if !result {
                        end_of_season(&mut self.leagues, &mut self.team_map, &mut self.player_map, 4, self.year, &self.data, &mut self.rng);
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

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Leagues");
            for league_idx in 0..self.leagues.len() {
                ui.horizontal(|ui| {
                    ui.label(format!("League {}", league_idx + 1));
                    if ui.button("Sche").clicked() {
                        self.disp_mode = Mode::Schedule(league_idx, None);
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
                Mode::Schedule(disp_league, cur_day) => {
                    let league = &self.leagues[*disp_league];
                    let total_games = league.schedule.games.len();

                    let teams = league.teams.len();
                    let half_teams = teams / 2;

                    let mut mode = Mode::Schedule(*disp_league, *cur_day);
                    let cur = cur_day.unwrap_or(league.cur_idx / half_teams);
                    let start = cur * half_teams;
                    let end = start + half_teams;

                    ui.horizontal_wrapped(|ui| {
                        if ui.add_enabled(cur > 0, Button::new("< Prev")).clicked() {
                            mode = Mode::Schedule(*disp_league, Some(cur - 1));
                        }
                        if ui.button("Today").clicked() {
                            mode = Mode::Schedule(*disp_league, None);
                        }
                        if ui.add_enabled(end <= total_games, Button::new("Next >")).clicked() {
                            mode = Mode::Schedule(*disp_league, Some(cur + 1));
                        }
                    });
                    ui.end_row();

                    ui.heading(format!("Day {}", cur + 1));

                    ui.group(|ui| {
                        ui.horizontal_wrapped(|ui| {
                            if end <= total_games {
                                for idx in start..end {
                                    let game = &league.schedule.games[idx];
                                    if display_game(ui, game, &self.team_map) {
                                        mode = Mode::BoxScore(*disp_league, idx)
                                    }
                                    if ((idx - start + 1) % 5) == 0 {
                                        ui.end_row();
                                    }
                                }
                            } else {
                                ui.label("End of season.");
                            }
                        });
                    });

                    mode
                }
                Mode::BoxScore(disp_league, game_idx) => {
                    let league = &self.leagues[*disp_league];
                    let mut mode = Mode::BoxScore(*disp_league, *game_idx);
                    let game = &league.schedule.games[*game_idx];

                    let awayteam = self.team_map.get(&game.away.id).unwrap();
                    let hometeam = self.team_map.get(&game.home.id).unwrap();

                    let mut awayruns = Vec::new();
                    let mut homeruns = Vec::new();

                    ui.horizontal(|ui| {
                        if ui.button("Back").clicked() {
                            let half_teams = league.teams.len() / 2;
                            let cur_day = game_idx / half_teams;
                            mode = Mode::Schedule(*disp_league, Some(cur_day));
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

                        let skip = matches!(event.event, Stat::Pw|Stat::Pcg|Stat::Psho|Stat::Pl|Stat::Psv|Stat::Pbs|Stat::Phld|Stat::Po|Stat::Pso);

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
                        ui.monospace(awayteam.abbr());
                        for awayrun in awayruns.iter() {
                            ui.monospace(format!("{}", awayrun));
                        }
                        ui.monospace(format!("{:3}", game.away.r));
                        ui.monospace(format!("{:3}", game.away.h));
                        ui.monospace(format!("{:3}", game.away.e));
                        ui.end_row();
                        ui.monospace(hometeam.abbr());
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
                            let pitcher = self.player_map.get(&w).unwrap();
                            ui.label(format!("W: {}", pitcher.fname()));
                        }
                        if let Some(l) = loser {
                            let pitcher = self.player_map.get(&l).unwrap();
                            ui.label(format!("L: {}", pitcher.fname()));
                        }
                        if let Some(sv) = save {
                            let pitcher = self.player_map.get(&sv).unwrap();
                            ui.label(format!("SV: {}", pitcher.fname()));
                        }
                    });

                    ui.separator();

                    ui.columns(2, |cols| {
                        for (i, col) in cols.iter_mut().enumerate() {
                            match i {
                                0 => {
                                    egui::Grid::new("Away Batting").show(col, |ui| {
                                        display_bo(ui, &game.away, awayteam, &self.player_map, &stat_map);
                                    });
                                }
                                1 => {
                                    egui::Grid::new("Home Batting").show(col, |ui| {
                                        display_bo(ui, &game.home, hometeam, &self.player_map, &stat_map);
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
                                        display_pitching(ui, &game.away, awayteam, &self.player_map, &stat_map);
                                    });
                                }
                                1 => {
                                    egui::Grid::new("Home Pitching").show(col, |ui| {
                                        display_pitching(ui, &game.home, hometeam, &self.player_map, &stat_map);
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

                    ScrollArea::both().show(ui, |ui| {
                        let mut prevhalf = false;
                        let mut previnn = 0;

                        for_each_event(game, |inning, tophalf, event, error| {
                            let player = self.player_map.get(&event.player).unwrap();
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

                            let result_str = match event.event {
                                Stat::B1b => " singles",
                                Stat::B2b => " doubles",
                                Stat::B3b => " triples",
                                Stat::Bhr => " homers",
                                Stat::Bbb => " walks",
                                Stat::Bibb => " intentionally walked",
                                Stat::Bhbp => " is hit by pitch",
                                Stat::Bso => " strikes out",
                                Stat::Bgidp => " grounds into double play",
                                Stat::Bsb => " steals second",
                                Stat::Bcs => " is thrown out stealing",
                                Stat::Bo => if error {
                                    " reaches on error"
                                } else {
                                    " flies out"
                                },
                                Stat::Br => " scores",
                                _ => ""
                            };

                            ui.label(format!("{}{}{}.", player_str, result_str, target_str));
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
                            let team = self.team_map.get(*o).unwrap();
                            team.win_pct()
                        });
                        teams.reverse();


                        let mut rank = 1;
                        for team_id in teams.iter() {
                            let team = self.team_map.get(*team_id).unwrap();
                            ui.label(format!("{}", rank));
                            ui.label(team.abbr());
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

                    let team = self.team_map.get(id).unwrap();
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
                                    if let Some(player_id) = display_team_stats(ui, true, &BATTING_HEADERS, &team.players, &self.player_map) {
                                        mode = Mode::Player(*disp_league, player_id, Some(*id));
                                    }
                                });
                                ui.heading("Pitching");
                                egui::Grid::new("pitching").striped(true).show(ui, |ui| {
                                    if let Some(player_id) = display_team_stats(ui, false, &PITCHING_HEADERS, &team.players, &self.player_map) {
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

                    let player = self.player_map.get(player_id).unwrap();

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
                    ui.label(format!("Age: {} Born: {} {}", player.age(self.year), player.born, player.birthplace));
                    ui.label(format!("Pos: {}", player.pos));
                    ui.label(format!("Bats: {}", player.bats));
                    ui.label(format!("Throws: {}", player.throws));

                    ui.heading(if player.pos.is_pitcher() { "Pitching History" } else { "Batting History" });
                    let headers = if player.pos.is_pitcher() { &PITCHING_HEADERS[..] } else { &BATTING_HEADERS[..] };
                    egui::Grid::new("history").striped(true).show(ui, |ui| {
                        let mut total = display_historical_stats(ui, headers, &player.historical, &self.team_map);
                        let stats = player.get_stats();
                        let mut teams = HashSet::new();

                        for historical in &player.historical {
                            teams.insert(historical.team);
                        }

                        if stats.g > 0 {
                            let team = self.team_map.iter().find(|kv| kv.1.players.contains(player_id)).unwrap();
                            teams.insert(*team.0);
                            let league = self.leagues.iter().position(|o| o.teams.contains(team.0)).unwrap();
                            display_historical_stat_row(ui, headers, &stats, Some(self.year), Some((league + 1) as u32), team.1.abbr());
                            total.compile(&stats);
                        }
                        let team_count = if teams.len() == 1 { "1 team".to_owned() } else { format!("{} team(s)", teams.len()) };
                        display_historical_stat_row(ui, headers, &total, None, None, team_count.as_str());
                    });

                    mode
                }
                Mode::BatLeaders(disp_league, result, reverse) => {
                    let league = &self.leagues[*disp_league];
                    let mut mode = Mode::BatLeaders(*disp_league, *result, *reverse);

                    ScrollArea::both().show(ui, |ui| {
                        egui::Grid::new("bleaders").striped(true).show(ui, |ui| {
                            mode = display_leaders(ui, true, &BATTING_HEADERS, league, &self.team_map, &self.player_map, mode);
                        });
                    });

                    mode
                }
                Mode::PitLeaders(disp_league, result, reverse) => {
                    let league = &self.leagues[*disp_league];
                    let mut mode = Mode::PitLeaders(*disp_league, *result, *reverse);

                    ScrollArea::both().show(ui, |ui| {
                        egui::Grid::new("pleaders").striped(true).show(ui, |ui| {
                            mode = display_leaders(ui, false, &PITCHING_HEADERS, league, &self.team_map, &self.player_map, mode);
                        });
                    });

                    mode
                }
                Mode::LeagueRecords(disp_league) => {
                    let league = &self.leagues[*disp_league];

                    let mut mode = Mode::LeagueRecords(*disp_league);

                    let mut cnt = 0;
                    let mut batting = true;

                    ui.horizontal_wrapped(|ui| {
                        ui.heading("Batting Records");
                        ui.end_row();
                        ui.end_row();

                        for stat in &RECORD_STATS {
                            if let Some(Some(record)) = league.records.get(stat) {
                                let team = self.team_map.get(&record.team_id).unwrap();
                                let player = self.player_map.get(&record.player_id).unwrap();

                                if batting != stat.is_batting() {
                                    cnt = 0;
                                    batting = stat.is_batting();
                                    ui.end_row();
                                    ui.heading("Pitching Records");
                                    ui.end_row();
                                }

                                ui.group(|ui| {
                                    ui.vertical(|ui| {
                                        ui.heading(format!("{}: {}", stat, stat.value(record.record)));

                                        if ui.add(Button::new(player.fullname()).frame(false)).clicked() {
                                            mode = Mode::Player(*disp_league, record.player_id, None);
                                        }

                                        ui.small(format!("{} - {}", &team.abbr(), record.year));
                                    });
                                });

                                cnt += 1;
                                if cnt % 4 == 0 {
                                    ui.end_row();
                                }
                            }
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

    /// Called by the framework to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }
    //fn name(&self) -> &str { "imp019" }
    //fn max_size_points(&self) -> egui::Vec2 { egui::Vec2::new(2048.0, 1024.0) }
}

