use std::collections::HashMap;

use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use strum_macros::EnumIter;

use crate::data::Data;

#[derive(Copy, Clone, Debug, PartialEq, EnumIter)]
pub(crate) enum Position {
    Pitcher,
    Catcher,
    FirstBase,
    SecondBase,
    ThirdBase,
    ShortStop,
    LeftField,
    CenterField,
    RightField,
    DesignatedHitter,
}

impl Position {
    fn to_str(&self) -> &str {
        match self {
            Position::Pitcher => "P",
            Position::Catcher => "C",
            Position::FirstBase => "1B",
            Position::SecondBase => "2B",
            Position::ThirdBase => "3B",
            Position::ShortStop => "SS",
            Position::LeftField => "LF",
            Position::CenterField => "CF",
            Position::RightField => "RF",
            Position::DesignatedHitter => "DH",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Stat {
    // recorded
    H1b,
    H2b,
    H3b,
    HR,
    BB,
    HBP,
    O,
    // calculated
    H,
    AB,
    PA,
    AVG,
    OBP,
    SLG,
}

fn calc_avg1000(ab: u32, h: u32) -> u32 {
    if ab > 0 {
        h * 1000 / ab
    } else {
        0
    }
}

fn calc_obp1000(pa: u32, h: u32, bb: u32, hbp: u32) -> u32 {
    if pa > 0 {
        ((h + bb + hbp) * 1000) / pa
    } else {
        0
    }
}

fn calc_slg1000(ab: u32, h1b: u32, h2b: u32, h3b: u32, hr: u32) -> u32 {
    if ab > 0 {
        ((h1b + (2 * h2b) + (3 * h3b) + (4 * hr)) * 1000) / ab
    } else {
        0
    }
}

pub(crate) struct Stats {
    h1b: u32,
    pub(crate) h2b: u32,
    pub(crate) h3b: u32,
    pub(crate) hr: u32,
    pub(crate) bb: u32,
    pub(crate) hbp: u32,
    o: u32,
    pub(crate) h: u32,
    pub(crate) ab: u32,
    pub(crate) pa: u32,
    pub(crate) avg: u32,
    pub(crate) obp: u32,
    pub(crate) slg: u32,
}

#[derive(Default)]
pub(crate) struct HistoricalStats {
    pub(crate) year: u32,
    pub(crate) league: u32,
    pub(crate) team: u64,
    pub(crate) stats: HashMap<Stat, u32>,
}

impl HistoricalStats {
    pub(crate) fn get_stats(&self) -> Stats {
        let h1b = *self.stats.get(&Stat::H1b).unwrap_or(&0);
        let h2b = *self.stats.get(&Stat::H2b).unwrap_or(&0);
        let h3b = *self.stats.get(&Stat::H3b).unwrap_or(&0);
        let hr = *self.stats.get(&Stat::HR).unwrap_or(&0);
        let bb = *self.stats.get(&Stat::BB).unwrap_or(&0);
        let hbp = *self.stats.get(&Stat::HBP).unwrap_or(&0);
        let o = *self.stats.get(&Stat::O).unwrap_or(&0);

        let h = h1b + h2b + h3b + hr;
        let ab = h + o;
        let pa = ab + bb + hbp;


        Stats {
            h1b,
            h2b,
            h3b,
            hr,
            bb,
            hbp,
            o,
            h,
            ab,
            pa,
            avg: calc_avg1000(ab, h),
            obp: calc_obp1000(pa, h, bb, hbp),
            slg: calc_slg1000(ab, h1b, h2b, h3b, hr),
        }
    }
}

pub(crate) struct Player {
    pub(crate) id: u64,
    name_first: String,
    name_last: String,
    pub(crate) age: u8,
    pub(crate) pos: Position,
    expect: Vec<(Stat, u32)>,
    stats: Vec<Stat>,
    pub(crate) historical: Vec<HistoricalStats>,
}

impl Player {
    pub(crate) fn new(data: &Data, id: u64, pos: &Position, rng: &mut ThreadRng) -> Self {
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
        expect.push((Stat::H1b, h1b));
        expect.push((Stat::H2b, h2b));
        expect.push((Stat::H3b, h3b));
        expect.push((Stat::HR, hr));
        expect.push((Stat::BB, bb));
        expect.push((Stat::HBP, hbp));
        expect.push((Stat::O, o));

        Player {
            id,
            pos: *pos,
            name_first,
            name_last,
            expect,
            stats: vec![],
            age: 0,
            historical: vec![]
        }
    }

    pub(crate) fn fullname(&self) -> String {
        format!("{} {}", self.name_first, self.name_last)
    }

    fn reset_stats(&mut self) {
        self.stats.clear();
    }

    pub(crate) fn record_stat(&mut self, stat: Stat) {
        self.stats.push(stat);
    }

    pub(crate) fn end_of_year(&mut self, year: u32, league: u32, team_id: u64) {
        let mut historical = HistoricalStats {
            year,
            league,
            team: team_id,
            ..HistoricalStats::default()
        };
        for stat in &self.stats {
            let val = historical.stats.entry(*stat).or_insert(0);
            *val += 1;
        }
        self.historical.push(historical);

        self.age += 1;

        self.reset_stats()
    }

    pub(crate) fn get_expected_pa(&self, rng: &mut ThreadRng) -> Stat {
        self.expect.choose_weighted(rng, |o| o.1).unwrap().0
    }

    pub(crate) fn get_stat(&self, stat: Stat) -> u32 {
        match stat {
            Stat::H1b => self.get_stats().h1b,
            Stat::H2b => self.get_stats().h2b,
            Stat::H3b => self.get_stats().h3b,
            Stat::HR => self.get_stats().hr,
            Stat::BB => self.get_stats().bb,
            Stat::HBP => self.get_stats().hbp,
            Stat::O => self.get_stats().o,
            Stat::H => self.get_stats().h,
            Stat::AB => self.get_stats().ab,
            Stat::PA => self.get_stats().pa,
            Stat::AVG => self.get_stats().avg,
            Stat::OBP => self.get_stats().obp,
            Stat::SLG => self.get_stats().slg,
        }

    }


    pub(crate) fn get_stats(&self) -> Stats {
        let mut h1b = 0;
        let mut h2b = 0;
        let mut h3b = 0;
        let mut hr = 0;
        let mut bb = 0;
        let mut hbp = 0;
        let mut o = 0;

        for stat in &self.stats {
            match stat {
                Stat::H1b => h1b += 1,
                Stat::H2b => h2b += 1,
                Stat::H3b => h3b += 1,
                Stat::HR => hr += 1,
                Stat::BB => bb += 1,
                Stat::HBP => hbp += 1,
                Stat::O => o += 1,
                _ => {}
            }
        }

        let h = h1b + h2b + h3b + hr;
        let ab = h + o;
        let pa = ab + bb + hbp;

        Stats {
            h1b,
            h2b,
            h3b,
            hr,
            bb,
            hbp,
            o,
            h,
            ab,
            pa,
            avg: calc_avg1000(ab, h),
            obp: calc_obp1000(pa, h, bb, hbp),
            slg: calc_slg1000(ab, h1b, h2b, h3b, hr),
        }
    }

    pub(crate) fn display_position(&self) -> String {
        self.pos.to_str().to_string()
    }
}
