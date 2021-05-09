use std::cmp::{max, min};
use std::collections::HashMap;

use enum_iterator::IntoEnumIterator;

use crate::player::{Player, PlayerId, PlayerMap, PlayerRefMap, Position};
use crate::data::LocData;

pub(crate) type TeamId = u64;
pub(crate) type TeamMap = HashMap<TeamId, Team>;

#[derive(Default, Copy, Clone)]
pub(crate) struct Results {
    win: u32,
    lose: u32,
}

impl Results {
    pub(crate) fn games(&self) -> u32 {
        self.win + self.lose
    }
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
    pub(crate) loc: LocData,
    pub(crate) nickname: String,
    pub(crate) players: Vec<PlayerId>,
    pub(crate) rotation: [PlayerId; 5],
    pub(crate) results: Results,
    pub(crate) history: History,
}

impl Team {
    pub(crate) fn new(loc: LocData, nickname: String, year: u32) -> Self {
        Self {
            loc,
            nickname,
            players: Vec::new(),
            rotation: [0, 0, 0, 0, 0],
            results: Results::default(),
            history: History {
                founded: year,
                ..History::default()
            },
        }
    }
    pub(crate) fn abbr(&self) -> &str {
        self.loc.abbr
    }

    pub(crate) fn name(&self) -> String {
        format!("{} {} ({}-{})", self.loc.city, self.nickname, self.loc.state, self.loc.country)
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

    fn players_per_position(pos: Position) -> usize {
        match pos {
            Position::StartingPitcher => 5,
            Position::LongRelief => 3,
            Position::ShortRelief => 3,
            Position::Catcher => 2,
            _ => 1,
        }
    }

    fn count_at(&self, players: &PlayerMap, pred: &dyn Fn(&&Player) -> bool) -> usize {
        self.players.iter().filter_map(|o| players.get(o)).filter(pred).count()
    }

    fn pick(available: &mut PlayerRefMap<'_>, pred: &dyn Fn(&&Player) -> bool) -> Option<PlayerId> {
        if let Some(avail) = available.iter().find(|(_, v)| pred(v)) {
            let id = *avail.0;
            available.remove(&id);
            Some(id)
        } else {
            None
        }
    }

    fn fill_in(&mut self, available: &mut PlayerRefMap<'_>, players: &PlayerMap, max: usize, pred: &dyn Fn(&&Player) -> bool) {
        let cur = self.count_at(players, pred);
        for _ in cur..max {
            if let Some(id) = Self::pick(available, pred) {
                self.players.push(id);
            }
        }
    }

    pub(crate) fn populate(&mut self, available: &mut PlayerRefMap<'_>, players: &PlayerMap) {
        for pos in Position::into_enum_iter() {
            let max = Self::players_per_position(pos);
            let exact_position = |o: &&Player| o.pos == pos;
            self.fill_in(available, players, max, &exact_position);
        }

        let is_infield = |o: &&Player| o.pos.is_infield();
        self.fill_in(available, players, 6, &is_infield);

        let is_outfield = |o: &&Player| o.pos.is_outfield();
        self.fill_in(available, players, 4, &is_outfield);

        let pitchers = self.players.iter().filter(|o| players.get(o).unwrap().pos == Position::StartingPitcher).collect::<Vec<_>>();
        for (idx, p) in pitchers[0..5].iter().enumerate() {
            self.rotation[idx] = **p;
        }
    }
}
