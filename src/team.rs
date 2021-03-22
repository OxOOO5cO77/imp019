use rand::rngs::ThreadRng;

use crate::data::Data;
use crate::player::Player;

#[derive(Default, Copy, Clone)]
pub(crate) struct Results {
    pub(crate) win: u32,
    pub(crate) lose: u32,
}

impl Results {
    pub(crate) fn reset(&mut self) {
        self.win = 0;
        self.lose = 0;
    }
}

pub(crate) struct HistoricalResults {
    pub(crate) year: u32,
    pub(crate) league: usize,
    pub(crate) rank: usize,
    pub(crate) win: u32,
    pub(crate) lose: u32,
}

#[derive(Default)]
pub(crate) struct History {
    pub(crate) founded: u32,
    pub(crate) best: Option<u32>,
    pub(crate) worst: Option<u32>,
    pub(crate) wins: u32,
    pub(crate) losses: u32,
    pub(crate) results: Vec<HistoricalResults>,
}

impl History {
    pub(crate) fn record_results(&mut self, year: u32, league: usize, rank: usize, results: Results) {
        self.results.push(HistoricalResults {
            year,
            league: league + 1,
            rank: rank + 1,
            win: results.win,
            lose: results.lose,
        });
    }
}

pub(crate) struct Team {
    pub(crate) id: u64,
    pub(crate) abbr: String,
    city: String,
    state: String,
    nickname: String,
    pub(crate) players: Vec<Player>,
    pub(crate) results: Results,
    pub(crate) history: History,
}

impl Team {
    pub(crate) fn new(data: &mut Data, year: u32, team_id: &mut u64, player_id: &mut u64, rng: &mut ThreadRng) -> Self {
        let loc = data.pull_loc();
        let mut loc = loc.split(',');
        let abbr = loc.next().or(Some("")).unwrap().to_owned();
        let city = loc.next().or(Some("")).unwrap().to_owned();
        let state = loc.next().or(Some("")).unwrap().to_owned() + "-" + loc.next().or(Some("")).unwrap();


        let mut players = Vec::new();
        for _ in 0..25 {
            players.push(Player::new(data, player_id, rng));
        }

        *team_id += 1;

        Team {
            id: *team_id,
            abbr,
            city,
            state,
            nickname: data.pull_nick(),
            players,
            results: Results::default(),
            history: History {
                founded: year,
                ..History::default()
            },
        }
    }

    pub(crate) fn name(&self) -> String {
        format!("{} {} ({})", self.city, self.nickname, self.state)
    }
}
