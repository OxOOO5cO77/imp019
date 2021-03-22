use std::collections::HashMap;

use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::data::Data;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum PAResult {
    H1b,
    H2b,
    H3b,
    HR,
    BB,
    HBP,
    O,
}

#[derive(Default)]
pub(crate) struct HistoricalStats {
    pub(crate) year: u32,
    pub(crate) league: u32,
    pub(crate) team: u32,
    pub(crate) stats: HashMap<PAResult, u32>,
}

#[derive(Default)]
pub(crate) struct Player {
    name_first: String,
    name_last: String,
    pub(crate) expect: Vec<(PAResult, u32)>,
    pub(crate) stats: Vec<PAResult>,
    pub(crate) historical: Vec<HistoricalStats>,
}

impl Player {
    pub(crate) fn new(data: &Data, rng: &mut ThreadRng) -> Self {
        let name_first = data.names_first.choose_weighted(rng, |o| o.1).unwrap().0.clone();
        let name_last = data.names_last.choose_weighted(rng, |o| o.1).unwrap().0.clone();

        let obp = 235 + rng.gen_range(0..165);
        let bb = rng.gen_range(100..360);
        let hbp = rng.gen_range(0..40).max(20) - 10;
        let hr = rng.gen_range(0..220);
        let h3b = rng.gen_range(0..40).max(20) - 10;
        let h2b = rng.gen_range(0..260);
        let h1b = 1000 - bb - hbp - hr - h3b - h2b;
        let o = ((1000 * 1000) / obp) - 1000;

        let mut expect = Vec::new();
        expect.push((PAResult::H1b, h1b));
        expect.push((PAResult::H2b, h2b));
        expect.push((PAResult::H3b, h3b));
        expect.push((PAResult::HR, hr));
        expect.push((PAResult::BB, bb));
        expect.push((PAResult::HBP, hbp));
        expect.push((PAResult::O, o));

        Player {
            name_first,
            name_last,
            expect,
            ..Player::default()
        }
    }

    pub(crate) fn fullname(&self) -> String {
        format!("{} {}", self.name_first, self.name_last)
    }

    pub(crate) fn reset_stats(&mut self) {
        self.stats.clear();
    }

    pub(crate) fn record_stats(&mut self, year: u32, league: u32, team: u32) {
        let mut historical = HistoricalStats {
            year,
            league,
            team,
            ..HistoricalStats::default()
        };
        for stat in &self.stats {
            let val = historical.stats.entry(*stat).or_insert(0);
            *val += 1;
        }
        self.historical.push(historical);
    }
}
