use std::collections::HashMap;

use lazy_static::lazy_static;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::player::{Expect, Handedness, Player, Position, Stat};
use crate::team::Team;
use rand::Rng;

#[derive(Copy, Clone, Default)]
struct RunnerInfo {
    runner: u64,
    pitcher: u64,
    earned: bool,
}

#[derive(Clone, Default)]
struct DefenseInfo {
    player: u64,
    pos: Position,
}

#[derive(Default)]
pub(crate) struct Scoreboard {
    pub(crate) id: u64,
    onbase: [Option<RunnerInfo>; 4],
    runs_in: Vec<RunnerInfo>,
    pub(crate) r: u8,
    //    pub(crate) h: u8,
//    pub(crate) e: u8,
    bo: [DefenseInfo; 9],
    ab: usize,
    pitcher_of_record: u64,

}

impl Scoreboard {
    fn new(id: u64) -> Self {
        Scoreboard {
            id,
            ..Scoreboard::default()
        }
    }

    fn advance_onbase(&mut self, batter: u64, pitcher: u64, earned: bool, amt: u8) {
        self.onbase[0] = Some(RunnerInfo { runner: batter, pitcher, earned });
        for _ in 0..amt {
            if self.onbase[1].is_some() {
                if self.onbase[2].is_some() {
                    if self.onbase[3].is_some() {
                        self.runs_in.push(self.onbase[3].unwrap());
                    }
                    self.onbase[3] = self.onbase[2];
                }
                self.onbase[2] = self.onbase[1];
            }
            self.onbase[1] = self.onbase[0];
        }
        for idx in 0..amt as usize {
            self.onbase[idx] = None;
        }
    }

    pub(crate) fn record_runs(&mut self) {
        self.r += self.runs_in.len() as u8;
        self.runs_in.clear();
    }
}

#[derive(PartialEq)]
enum InningHalf {
    Top,
    Middle,
    Bottom,
    End,
}

impl Default for InningHalf {
    fn default() -> Self { InningHalf::Top }
}

#[derive(Default)]
struct Inning {
    number: u8,
    half: InningHalf,
}

#[derive(Default)]
pub(crate) struct Game {
    pub(crate) home: Scoreboard,
    pub(crate) away: Scoreboard,
    inning: Inning,
    outs: u8,
}

lazy_static! {
    static ref LEAGUE_AVG: HashMap<Expect, f64> = {
        let mut expect = HashMap::new();
        expect.insert(Expect::Single, 0.1379988963);
        expect.insert(Expect::Double, 0.045119492);
        expect.insert(Expect::Triple, 0.004006693438);
        expect.insert(Expect::HomeRun, 0.03522694576);
        expect.insert(Expect::Walk, 0.08492014357);
        expect.insert(Expect::HitByPitch, 0.01096355115);
        expect.insert(Expect::Strikeout, 0.19);
        expect.insert(Expect::Out, 0.4909664694);
        expect
    };
}


impl Game {
    fn new(home: u64, away: u64) -> Self {
        Game {
            home: Scoreboard::new(home),
            away: Scoreboard::new(away),
            inning: Inning {
                number: 1,
                half: InningHalf::Top,
            },
            outs: 0,
        }
    }

    fn complete(&self) -> bool {
        self.inning.number >= 9 && ((self.inning.half != InningHalf::Top && self.home.r > self.away.r) || (self.inning.half == InningHalf::End && self.away.r > self.home.r))
    }

    fn is_away_ab(&self) -> bool {
        self.inning.half == InningHalf::Top || self.inning.half == InningHalf::Middle
    }

    fn matchup_morey_z(batter: f64, pitcher: f64, league: f64) -> f64 {
        let sqrt_league = (league * (1.0 - league)).sqrt();
        let top_left = (batter - league) / sqrt_league;
        let top_right = (pitcher - league) / sqrt_league;
        let left = (top_left + top_right) / 2.0f64.sqrt();
        (left * sqrt_league) + league
    }

    fn setup_pitcher(players: &mut HashMap<u64, Player>, teams: &mut HashMap<u64, Team>, scoreboard: &mut Scoreboard) -> Handedness {
        let team = teams.get_mut(&scoreboard.id).unwrap();
        scoreboard.pitcher_of_record = team.rotation[0];
        let pitcher = Game::record_stat(players, team.rotation[0], Stat::Gs);
        team.rotation.rotate_left(1);
        pitcher.throws
    }

    fn setup_bo(players: &mut HashMap<u64, Player>, teams: &mut HashMap<u64, Team>, scoreboard: &mut Scoreboard, year: u32, rng: &mut ThreadRng) {
        let team = teams.get_mut(&scoreboard.id).unwrap();
        let mut team_players = team.players.iter().map(|o| (*o, players.get(o).unwrap())).filter(|o| o.1.pos != Position::Pitcher).collect::<Vec<_>>();
        team_players.sort_by_cached_key(|o| o.1.get_stats().b_obp);
        team_players.reverse();

        let mut index = 0;
        for (id, player) in &team_players {
            if scoreboard.bo.iter().find(|o| o.pos == player.pos).is_none() {
                scoreboard.bo[index] = DefenseInfo {
                    player: *id,
                    pos: player.pos,
                };
                index += 1;
            }
        }

        for starter in scoreboard.bo.iter_mut() {

            if let Some(replacement) = team_players.iter().find(|o| o.0 != starter.player && o.1.pos == starter.pos ) {
                let starter_player = players.get(&starter.player).unwrap();
                let fat_pct = starter_player.fatigue as f64 / starter_player.fatigue_threshold(year);
                if rng.gen_bool(fat_pct.min(1.0)) {
                    starter.player = replacement.0;
                }
            }
        }

        for starter in scoreboard.bo.iter() {
            let player = Game::record_stat(players,starter.player,Stat::Gs);
            player.fatigue += 1;
        }
    }

    fn setup_game(&mut self, players: &mut HashMap<u64, Player>, teams: &mut HashMap<u64, Team>, year: u32, rng: &mut ThreadRng) {
        let _home_hand = Self::setup_pitcher(players, teams, &mut self.home);
        let _away_hand = Self::setup_pitcher(players, teams, &mut self.away);

        Self::setup_bo(players, teams, &mut self.home, year, rng);
        Self::setup_bo(players, teams, &mut self.away, year, rng);

        self.inning.number = 1;
    }

    fn get_expected_pa(batter: &Player, pitcher: &Player, rng: &mut ThreadRng) -> Expect {
        *batter.bat_expect.iter().map(|kv| {
            let bval = kv.1;
            let pval = pitcher.pit_expect.get(&kv.0).unwrap_or(&0.0);
            let lval = LEAGUE_AVG.get(&kv.0).unwrap_or(&0.0);
            let res = (Game::matchup_morey_z(*bval, *pval, *lval) * 1000.0) as u32;
            (kv.0, res)
        }).collect::<Vec<_>>().choose_weighted(rng, |o| o.1).unwrap().0
    }

    fn record_stat(players: &mut HashMap<u64, Player>, player_id: u64, stat: Stat) -> &mut Player {
        let player = players.get_mut(&player_id).unwrap();
        player.record_stat(stat);
        match stat {
            Stat::Bso => player.record_stat(Stat::Bo),
            Stat::Pso => player.record_stat(Stat::Po),
            Stat::Gs => player.record_stat(Stat::G),
            _ => {}
        }
        player
    }

    pub(crate) fn sim(&mut self, teams: &mut HashMap<u64, Team>, players: &mut HashMap<u64, Player>, year: u32, rng: &mut ThreadRng) {
        self.setup_game(players, teams, year, rng);

        while !self.complete() {
            if self.inning.half == InningHalf::Middle {
                self.home.onbase.fill(None);
                self.outs = 0;
                self.inning.half = InningHalf::Bottom;
                continue;
            }
            if self.inning.half == InningHalf::End {
                self.away.onbase.fill(None);
                self.outs = 0;
                self.inning.number += 1;
                self.inning.half = InningHalf::Top;
                continue;
            }

            let pitcher_id = if self.is_away_ab() { self.home.pitcher_of_record } else { self.away.pitcher_of_record };
            let pitcher = players.get(&pitcher_id).unwrap();
            let bat_scoreboard = if self.is_away_ab() { &mut self.away } else { &mut self.home };

            let batter_id = bat_scoreboard.bo[bat_scoreboard.ab].player;
            let batter = players.get(&batter_id).unwrap();
            let result = Game::get_expected_pa(batter, pitcher, rng);
            match result {
                Expect::Single => bat_scoreboard.advance_onbase(batter_id, pitcher_id, true, 1),
                Expect::Double => bat_scoreboard.advance_onbase(batter_id, pitcher_id, true, 2),
                Expect::Triple => bat_scoreboard.advance_onbase(batter_id, pitcher_id, true, 3),
                Expect::HomeRun => bat_scoreboard.advance_onbase(batter_id, pitcher_id, true, 4),
                Expect::Walk => bat_scoreboard.advance_onbase(batter_id, pitcher_id, true, 1),
                Expect::HitByPitch => bat_scoreboard.advance_onbase(batter_id, pitcher_id, true, 1),
                Expect::Strikeout => self.outs += 1,
                Expect::Out => self.outs += 1,
            };
            let batting_stat = result.to_batting_stat();
            let pitching_stat = result.to_pitching_stat();

            let batter = Game::record_stat(players, batter_id, batting_stat);

            for _ in &bat_scoreboard.runs_in {
                batter.record_stat(Stat::Brbi);
            }

            Game::record_stat(players, pitcher_id, pitching_stat);

            for runner in &bat_scoreboard.runs_in {
                Game::record_stat(players, runner.runner, Stat::Br);
                let pitcher = Game::record_stat(players, runner.pitcher, Stat::Pr);
                if runner.earned {
                    pitcher.record_stat(Stat::Per);
                }
            }

            bat_scoreboard.record_runs();

            bat_scoreboard.ab = (bat_scoreboard.ab + 1) % 9;

            if self.outs >= 3 {
                if self.inning.half == InningHalf::Top {
                    self.inning.half = InningHalf::Middle;
                } else if self.inning.half == InningHalf::Bottom {
                    self.inning.half = InningHalf::End;
                }
            }
        }

        teams.get_mut(&self.home.id).unwrap().results(self.home.r, self.away.r);
        teams.get_mut(&self.away.id).unwrap().results(self.away.r, self.home.r);
    }
}


pub(crate) struct Schedule {
    pub(crate) games: Vec<Game>,
}

impl Schedule {
    pub(crate) fn new(teams: &[u64], rng: &mut ThreadRng) -> Self {
        let mut raw_matchups = Vec::new();
        let team_count = teams.len();
        raw_matchups.reserve(team_count * (team_count - 1));

        for home in teams {
            for away in teams {
                if home != away {
                    raw_matchups.push(Game::new(*home, *away));
                }
            }
        }

        raw_matchups.shuffle(rng);

        let mut matchups = Vec::new();
        while !raw_matchups.is_empty() {
            let mut teams_to_pick = (0..team_count).map(|o| teams[o]).collect::<Vec<_>>();
            teams_to_pick.shuffle(rng);

            while !teams_to_pick.is_empty() {
                if let Some(team) = teams_to_pick.pop() {
                    if let Some(idx) = raw_matchups.iter().position(|x| x.home.id == team && teams_to_pick.contains(&x.away.id)) {
                        let game = raw_matchups.remove(idx);
                        let other_team = if game.home.id == team { game.away.id } else { game.home.id };
                        matchups.push(game);
                        if let Some(other_pos) = teams_to_pick.iter().position(|&o| o == other_team) {
                            teams_to_pick.remove(other_pos);
                        }
                    }
                }
            }
        }

        let mut games = Vec::new();
        for idx in (0..matchups.len()).step_by(team_count / 2) {
            for _ in 0..4 {
                for offset in 0..(team_count / 2) {
                    let game = &matchups[idx + offset];
                    games.push(Game::new(game.home.id, game.away.id));
                }
            }
        }

        Schedule {
            games
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::schedule::Scoreboard;

    #[test]
    fn test_advance_onbase() {
        let mut test1 = Scoreboard::new(0);
        test1.advance_onbase(1, 0, true, 1);
        assert!(test1.onbase[0].is_none());
        assert!(test1.onbase[1].is_some());
        assert!(test1.onbase[2].is_none());
        assert!(test1.onbase[3].is_none());

        test1.advance_onbase(2, 0, true, 2);
        assert!(test1.onbase[0].is_none());
        assert!(test1.onbase[1].is_none());
        assert!(test1.onbase[2].is_some());
        assert!(test1.onbase[3].is_some());

        test1.advance_onbase(3, 0, true, 1);
        assert!(test1.onbase[0].is_none());
        assert!(test1.onbase[1].is_some());
        assert!(test1.onbase[2].is_some());
        assert!(test1.onbase[3].is_some());

        test1.advance_onbase(4, 0, true, 4);
        assert!(test1.onbase[0].is_none());
        assert!(test1.onbase[1].is_none());
        assert!(test1.onbase[2].is_none());
        assert!(test1.onbase[3].is_none());
        assert_eq!(test1.runs_in.len(), 4);

        test1.runs_in.clear();
        test1.advance_onbase(3, 0, true, 3);
        test1.advance_onbase(2, 0, true, 2);
        test1.advance_onbase(1, 0, true, 3);
        assert!(test1.onbase[0].is_none());
        assert!(test1.onbase[1].is_none());
        assert!(test1.onbase[2].is_none());
        assert!(test1.onbase[3].is_some());
        assert_eq!(test1.runs_in.len(), 2);

        test1.runs_in.clear();
        test1.advance_onbase(1, 0, true, 4);
        assert!(test1.onbase[0].is_none());
        assert!(test1.onbase[1].is_none());
        assert!(test1.onbase[2].is_none());
        assert!(test1.onbase[3].is_none());
        assert_eq!(test1.runs_in.len(), 2);

        test1.runs_in.clear();
        test1.advance_onbase(1, 0, true, 4);
        assert!(test1.onbase[0].is_none());
        assert!(test1.onbase[1].is_none());
        assert!(test1.onbase[2].is_none());
        assert!(test1.onbase[3].is_none());
        assert_eq!(test1.runs_in.len(), 1);
    }
}
