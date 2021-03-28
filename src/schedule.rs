use std::collections::HashMap;

use lazy_static::lazy_static;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::player::{Player, Stat};
use crate::team::Team;
use crate::player;

#[derive(Copy, Clone, Default)]
struct RunnerInfo {
    runner: u64,
    pitcher: u64,
    earned: bool,
}

#[derive(Default)]
pub(crate) struct Scoreboard {
    pub(crate) id: u64,
    onbase: [Option<RunnerInfo>; 4],
    runs_in: Vec<RunnerInfo>,
    pub(crate) r: u8,
    //    pub(crate) h: u8,
//    pub(crate) e: u8,
    ab: u8,
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
    static ref LEAGUE_AVG: HashMap<Stat, f64> = {
        let mut expect = HashMap::new();
        expect.insert(Stat::B1b, 0.1379988963);
        expect.insert(Stat::B2b, 0.045119492);
        expect.insert(Stat::B3b, 0.004006693438);
        expect.insert(Stat::Bhr, 0.03522694576);
        expect.insert(Stat::Bbb, 0.08492014357);
        expect.insert(Stat::Bhbp, 0.01096355115);
        expect.insert(Stat::Bo, 0.6809664694);
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

    fn setup_game(&mut self, teams: &mut HashMap<u64, Team>) {
        let home = teams.get_mut(&self.home.id).unwrap();
        self.home.pitcher_of_record = home.rotation[0];
        home.rotation.rotate_right(1);

        let away = teams.get_mut(&self.away.id).unwrap();
        self.away.pitcher_of_record = away.rotation[0];
        away.rotation.rotate_right(1);

        self.inning.number = 1;
    }

    fn get_expected_pa(batter: &Player, pitcher: &Player, rng: &mut ThreadRng) -> Stat {
        *batter.bat_expect.iter().map(|kv| {
            let bval = kv.1;
            let pval = pitcher.pit_expect.get(&kv.0).unwrap();
            let lval = LEAGUE_AVG.get(&kv.0).unwrap();
            let res = (Game::matchup_morey_z(*bval, *pval, *lval) * 1000.0) as u32;
            (kv.0, res)
        }).collect::<Vec<_>>().choose_weighted(rng, |o| o.1).unwrap().0
    }

    pub(crate) fn sim(&mut self, teams: &mut HashMap<u64, Team>, players: &mut HashMap<u64, Player>, rng: &mut ThreadRng) {
        self.setup_game(teams);

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

            let team = teams.get(&bat_scoreboard.id).unwrap();
            let batter_id = team.players[bat_scoreboard.ab as usize];
            let batter = players.get(&batter_id).unwrap();
            let result = Game::get_expected_pa(batter, pitcher, rng);
            match result {
                Stat::B1b => bat_scoreboard.advance_onbase(batter_id, pitcher_id, true, 1),
                Stat::B2b => bat_scoreboard.advance_onbase(batter_id, pitcher_id, true, 2),
                Stat::B3b => bat_scoreboard.advance_onbase(batter_id, pitcher_id, true, 3),
                Stat::Bhr => bat_scoreboard.advance_onbase(batter_id, pitcher_id, true, 4),
                Stat::Bbb => bat_scoreboard.advance_onbase(batter_id, pitcher_id, true, 1),
                Stat::Bhbp => bat_scoreboard.advance_onbase(batter_id, pitcher_id, true, 1),
                Stat::Bo => self.outs += 1,
                _ => {}
            };
            {
                let batter = players.get_mut(&batter_id).unwrap();
                batter.record_stat(result);

                for _ in &bat_scoreboard.runs_in {
                    batter.record_stat(Stat::Brbi);
                }
            }
            {
                let pitcher = players.get_mut(&pitcher_id).unwrap();
                pitcher.record_stat(player::opposing_stat(result).unwrap());
            }


            for runner in &bat_scoreboard.runs_in {
                {
                    let runner = players.get_mut(&runner.runner).unwrap();
                    runner.record_stat(Stat::Br);
                }
                {
                    let pitcher = players.get_mut(&runner.pitcher).unwrap();
                    if runner.earned  {
                        pitcher.record_stat( Stat::Per);
                    }
                    pitcher.record_stat( Stat::Pr);
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
