use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::player::Stat;
use crate::team::Team;

#[derive(Default)]
pub(crate) struct Scoreboard {
    pub(crate) team: usize,
    onbase: [bool; 4],
    pub(crate) r: u8,
//    pub(crate) h: u8,
//    pub(crate) e: u8,
    pub(crate) ab: u8,

}

impl Scoreboard {
    fn new(team: usize) -> Self {
        Scoreboard {
            team,
            ..Scoreboard::default()
        }
    }

    fn advance_onbase(&mut self, batter: bool, amt: u8) -> u8 {
        let mut runs = 0;

        self.onbase[0] = batter;
        for _ in 0..amt {
            if self.onbase[3] {
                runs += 1;
            }
            self.onbase[3] = self.onbase[2];
            self.onbase[2] = self.onbase[1];
            self.onbase[1] = self.onbase[0];
            self.onbase[0] = false;
        }
        runs
    }
}

#[derive(PartialEq)]
enum Inning {
    Top,
    Middle,
    Bottom,
    End,
}

impl Default for Inning {
    fn default() -> Self { Inning::Top }
}

#[derive(Default)]
pub(crate) struct Game {
    pub(crate) home: Scoreboard,
    pub(crate) away: Scoreboard,
    inning: (u8, Inning),
    outs: u8,
}

impl Game {
    fn new(home: usize, away: usize) -> Self {
        Game {
            home: Scoreboard::new(home),
            away: Scoreboard::new(away),
            ..Game::default()
        }
    }

    fn complete(&self) -> bool {
        self.inning.0 >= 9 && ((self.inning.1 != Inning::Top && self.home.r > self.away.r) || (self.inning.1 == Inning::End && self.away.r > self.home.r))
    }

    fn is_away(&self) -> bool {
        self.inning.1 == Inning::Top || self.inning.1 == Inning::Middle
    }

    pub(crate) fn sim(&mut self, teams: &mut [Team], rng: &mut ThreadRng) {
        self.inning.0 = 1;
        while !self.complete() {
            if self.inning.1 == Inning::Middle {
                self.home.onbase.fill(false);
                self.outs = 0;
                self.inning.1 = Inning::Bottom;
                continue;
            }
            if self.inning.1 == Inning::End {
                self.away.onbase.fill(false);
                self.outs = 0;
                self.inning.0 += 1;
                self.inning.1 = Inning::Top;
                continue;
            }

            let scoreboard = if self.is_away() { &mut self.away } else { &mut self.home };

            let team = &mut teams[scoreboard.team];
            let player = &mut team.players[scoreboard.ab as usize];
            let result = player.get_expected_pa(rng);
            let runs = match result {
                Stat::H1b => scoreboard.advance_onbase(true, 1),
                Stat::H2b => scoreboard.advance_onbase(true, 2),
                Stat::H3b => scoreboard.advance_onbase(true, 3),
                Stat::HR => scoreboard.advance_onbase(true, 4),
                Stat::BB => scoreboard.advance_onbase(true, 1),
                Stat::HBP => scoreboard.advance_onbase(true, 1),
                Stat::O => {
                    self.outs += 1;
                    0
                },
                _ => 0
            };
            scoreboard.r += runs;
            player.record_stat(result);
            scoreboard.ab = (scoreboard.ab + 1) % 9;

            if self.outs >= 3 {
                if self.inning.1 == Inning::Top {
                    self.inning.1 = Inning::Middle;
                } else if self.inning.1 == Inning::Bottom {
                    self.inning.1 = Inning::End;
                }
            }
        }
    }
}


pub(crate) struct Schedule {
    pub(crate) games: Vec<Game>,
}

impl Schedule {
    pub(crate) fn new(teams: usize, rng: &mut ThreadRng) -> Self {
        let mut raw_matchups = Vec::new();
        raw_matchups.reserve(teams * teams);

        for home in 0..teams {
            for away in 0..teams {
                if home != away {
                    raw_matchups.push(Game::new(home, away));
                }
            }
        }

        raw_matchups.shuffle(rng);

        let mut matchups = Vec::new();
        while !raw_matchups.is_empty() {
            let mut teams_to_pick = (0..teams).collect::<Vec<_>>();
            teams_to_pick.shuffle(rng);

            while !teams_to_pick.is_empty() {
                if let Some(team) = teams_to_pick.pop() {
                    if let Some(idx) = raw_matchups.iter().position(|x| x.home.team == team && teams_to_pick.contains(&x.away.team)) {
                        let game = raw_matchups.remove(idx);
                        let other_team = if game.home.team == team { game.away.team } else { game.home.team };
                        matchups.push(game);
                        if let Some(other_pos) = teams_to_pick.iter().position(|&o| o == other_team) {
                            teams_to_pick.remove(other_pos);
                        }
                    }
                }
            }
        }

        let mut games = Vec::new();
        for idx in (0..matchups.len()).step_by(teams / 2) {
            for _ in 0..4 {
                for offset in 0..(teams / 2) {
                    let game = &matchups[idx + offset];
                    games.push(Game::new(game.home.team, game.away.team));
                }
            }
        }

        Schedule {
            games
        }
    }
}
