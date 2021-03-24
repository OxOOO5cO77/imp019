use std::cmp::{max, min};

use crate::data::Data;

#[derive(Default, Copy, Clone)]
pub(crate) struct Results {
    win: u32,
    lose: u32,
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

pub(crate) struct Team {
    pub(crate) abbr: String,
    city: String,
    state: String,
    nickname: String,
    pub(crate) players: Vec<u64>,
    pub(crate) results: Results,
    pub(crate) history: History,
}

impl Team {
    pub(crate) fn new(data: &mut Data, year: u32) -> Self {
        let loc = data.pull_loc();
        let mut loc = loc.split(',');
        let abbr = loc.next().unwrap_or("").to_owned();
        let city = loc.next().unwrap_or("").to_owned();
        let state = loc.next().unwrap_or("").to_owned() + "-" + loc.next().unwrap_or("");

        Team {
            abbr,
            city,
            state,
            nickname: data.pull_nick(),
            players: Vec::new(),
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

    pub(crate) fn results(&mut self, us: u8, them: u8) {
        if us > them {
            self.results.win += 1;
        } else {
            self.results.lose += 1;
        }
    }

    pub(crate) fn get_wins(&self) -> u32 {
        self.results.win
    }

    pub(crate) fn get_losses(&self) -> u32 {
        self.results.lose
    }

    pub(crate) fn win_pct(&self) -> u32 {
        let denom = self.results.win + self.results.lose;
        if denom > 0 {
            (self.results.win * 1000 / denom) + 1
        } else {
            0
        }
    }

    pub(crate) fn record_results(&mut self, year: u32, league_idx: usize, rank_idx: usize, results: Results) {
        self.history.wins += self.results.win;
        self.history.losses += self.results.lose;

        let league = league_idx + 1;
        let rank = rank_idx + 1;
        let pos = Some((league * 100 + rank) as u32);
        self.history.best = if self.history.best.is_none() { pos } else { min(pos, self.history.best) };
        self.history.worst = if self.history.worst.is_none() { pos } else { max(pos, self.history.worst) };

        self.history.results.push(HistoricalResults {
            year,
            league,
            rank,
            win: results.win,
            lose: results.lose,
        });
    }
}
