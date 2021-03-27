use std::collections::HashMap;

use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::player::{Player, Stat};
use crate::team::Team;

#[derive(Default)]
pub(crate) struct Scoreboard {
    pub(crate) id: u64,
    onbase: [Option<u64>; 4],
    runs_in: Vec<u64>,
    pub(crate) r: u8,
    //    pub(crate) h: u8,
//    pub(crate) e: u8,
    ab: u8,

}

impl Scoreboard {
    fn new(id: u64) -> Self {
        Scoreboard {
            id,
            ..Scoreboard::default()
        }
    }

    fn advance_onbase(&mut self, batter: u64, amt: u8) {
        self.onbase[0] = Some(batter);
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

    fn is_away(&self) -> bool {
        self.inning.half == InningHalf::Top || self.inning.half == InningHalf::Middle
    }

    // fn matchup_morey_z(batter: f64, pitcher: f64, league: f64) -> f64 {
    //     let sqrt_league = (league * (1.0 - league)).sqrt();
    //     let top_left = (batter - league) / sqrt_league;
    //     let top_right = (pitcher - league) / sqrt_league;
    //     let left = (top_left + top_right) / 2.0f64.sqrt();
    //     (left * sqrt_league) + league
    // }

    pub(crate) fn sim(&mut self, teams: &mut HashMap<u64, Team>, players: &mut HashMap<u64, Player>, rng: &mut ThreadRng) {
        self.inning.number = 1;
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

            let scoreboard = if self.is_away() { &mut self.away } else { &mut self.home };

            let team = teams.get(&scoreboard.id).unwrap();
            let player_id = team.players[scoreboard.ab as usize];
            let player = players.get_mut(&player_id).unwrap();
            let result = player.get_expected_pa(rng);
            match result {
                Stat::H1b => scoreboard.advance_onbase(player_id, 1),
                Stat::H2b => scoreboard.advance_onbase(player_id, 2),
                Stat::H3b => scoreboard.advance_onbase(player_id, 3),
                Stat::Hr => scoreboard.advance_onbase(player_id, 4),
                Stat::Bb => scoreboard.advance_onbase(player_id, 1),
                Stat::Hbp => scoreboard.advance_onbase(player_id, 1),
                Stat::O => self.outs += 1,
                _ => {}
            };
            player.record_stat(result);

            for _ in &scoreboard.runs_in {
                player.record_stat(Stat::Rbi);
            }

            for runner_id in &scoreboard.runs_in {
                let runner = players.get_mut(&runner_id).unwrap();
                runner.record_stat(Stat::R);
            }

            scoreboard.record_runs();

            scoreboard.ab = (scoreboard.ab + 1) % 9;

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
        test1.advance_onbase(1, 1);
        assert_eq!(test1.onbase, [None, Some(1), None, None]);

        test1.advance_onbase(2, 2);
        assert_eq!(test1.onbase, [None, None, Some(2), Some(1)]);

        test1.advance_onbase(3, 1);
        assert_eq!(test1.onbase, [None, Some(3), Some(2), Some(1)]);

        test1.advance_onbase(4, 4);
        assert_eq!(test1.onbase, [None, None, None, None]);
        assert_eq!(test1.runs_in.len(), 4);

        test1.runs_in.clear();
        test1.advance_onbase(3, 3);
        test1.advance_onbase(2, 2);
        test1.advance_onbase(1, 3);
        assert_eq!(test1.onbase, [None, None, None, Some(1)]);
        assert_eq!(test1.runs_in.len(), 2);

        test1.runs_in.clear();
        test1.advance_onbase(1, 4);
        assert_eq!(test1.onbase, [None, None, None, None]);
        assert_eq!(test1.runs_in.len(), 2);

        test1.runs_in.clear();
        test1.advance_onbase(1, 4);
        assert_eq!(test1.onbase, [None, None, None, None]);
        assert_eq!(test1.runs_in.len(), 1);
    }
}
