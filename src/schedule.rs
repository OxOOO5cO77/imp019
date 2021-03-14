use rand::{Rng, rngs::ThreadRng};
use rand::seq::SliceRandom;

pub(crate) struct Scoreboard {
    pub(crate) team: usize,
    pub(crate) r: u8,
    h: u8,
    e: u8,
}

impl Scoreboard {
    fn new(team: usize) -> Self {
        Scoreboard {
            team,
            r: 0,
            h: 0,
            e: 0,
        }
    }
}

pub(crate) struct Game {
    pub(crate) home: Scoreboard,
    pub(crate) away: Scoreboard,
}

impl Game {
    fn new(home: usize, away: usize) -> Self {
        Game {
            home: Scoreboard::new(home),
            away: Scoreboard::new(away),
        }
    }

    pub(crate) fn sim(&mut self, rng: &mut ThreadRng) {
        self.home.r = rng.gen_range(0..12);
        self.away.r = rng.gen_range(0..12);
        if self.home.r == self.away.r {
            if rng.gen_bool(0.5) {
                self.home.r += 1
            } else {
                self.away.r += 1
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
        raw_matchups.reserve(teams*teams);

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
        for idx in (0..matchups.len()).step_by(teams/2)  {
            for _ in 0..4 {
                for offset in 0..(teams/2) {
                    let game = &matchups[idx+offset];
                    games.push( Game::new( game.home.team, game.away.team));
                }
            }
        }

        Schedule {
            games
        }
    }
}
