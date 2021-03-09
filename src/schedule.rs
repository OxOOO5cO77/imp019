use rand::{Rng, rngs::ThreadRng};

pub struct Scoreboard {
    pub team: usize,
    pub r: u8,
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

pub struct Game {
    pub home: Scoreboard,
    pub away: Scoreboard,
}

impl Game {
    fn new(home: usize, away: usize) -> Self {
        Game {
            home: Scoreboard::new(home),
            away: Scoreboard::new(away),
        }
    }

    pub fn sim(&mut self, rng: &mut ThreadRng) {
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


pub struct Schedule {
    pub games: Vec<Game>,
}

impl Schedule {
    pub fn new(teams: usize) -> Self {
        let mut games = Vec::new();
        for home in 0..teams {
            for away in 0..teams {
                if home != away {
                    games.push(Game::new(home, away));
                    games.push(Game::new(home, away));
                    games.push(Game::new(home, away));
                }
            }
        }
        Schedule {
            games
        }
    }
}
