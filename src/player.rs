use std::collections::HashMap;

use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand_distr::{Distribution, Gamma, Normal};
use strum_macros::EnumIter;

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
    pub(crate) fn to_str(&self) -> &str {
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

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum Handedness {
    Left,
    Right,
    Switch,
}

impl Handedness {
    pub(crate) fn to_str(&self) -> &str {
        match self {
            Handedness::Left => "L",
            Handedness::Right => "R",
            Handedness::Switch => "S",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Stat {
    // recorded
    H1b,
    H2b,
    H3b,
    Hr,
    Bb,
    Hbp,
    O,
    R,
    Rbi,
    // calculated
    H,
    Ab,
    Pa,
    Avg,
    Obp,
    Slg,
}

fn div1000_or_0(n: u32, d: u32 ) -> u32 {
    if d > 0 { (n * 1000) / d } else { d }
}

fn calc_avg1000(ab: u32, h: u32) -> u32 {
    div1000_or_0(h, ab )
}

fn calc_obp1000(pa: u32, h: u32, bb: u32, hbp: u32) -> u32 {
    div1000_or_0(h + bb + hbp, pa)
}

fn calc_slg1000(ab: u32, h1b: u32, h2b: u32, h3b: u32, hr: u32) -> u32 {
    div1000_or_0(h1b + (2 * h2b) + (3 * h3b) + (4 * hr), ab)
}

pub(crate) struct Stats {
    h1b: u32,
    pub(crate) h2b: u32,
    pub(crate) h3b: u32,
    pub(crate) hr: u32,
    pub(crate) bb: u32,
    pub(crate) hbp: u32,
    pub(crate) r: u32,
    pub(crate) rbi: u32,
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
    pub(crate) batting_stats: HashMap<Stat, u32>,
}

impl HistoricalStats {
    pub(crate) fn get_stats(&self) -> Stats {
        let h1b = *self.batting_stats.get(&Stat::H1b).unwrap_or(&0);
        let h2b = *self.batting_stats.get(&Stat::H2b).unwrap_or(&0);
        let h3b = *self.batting_stats.get(&Stat::H3b).unwrap_or(&0);
        let hr = *self.batting_stats.get(&Stat::Hr).unwrap_or(&0);
        let bb = *self.batting_stats.get(&Stat::Bb).unwrap_or(&0);
        let hbp = *self.batting_stats.get(&Stat::Hbp).unwrap_or(&0);
        let o = *self.batting_stats.get(&Stat::O).unwrap_or(&0);
        let r = *self.batting_stats.get(&Stat::R).unwrap_or(&0);
        let rbi = *self.batting_stats.get(&Stat::Rbi).unwrap_or(&0);

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
            r,
            rbi,
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
    name_first: String,
    name_last: String,
    pub(crate) age: u8,
    pub(crate) pos: Position,
    pub(crate) bats: Handedness,
    pub(crate) throws: Handedness,
    expect: Vec<(Stat, u32)>,
    stats: Vec<Stat>,
    pub(crate) historical: Vec<HistoricalStats>,
}

fn gen_normal(rng: &mut ThreadRng, mean: f64, stddev: f64) -> f64 {
    Normal::new(mean, stddev).unwrap().sample(rng)
}

fn gen_gamma(rng: &mut ThreadRng, shape: f64, scale: f64) -> f64 {
    Gamma::new(shape, scale).unwrap().sample(rng)
}

impl Player {
    pub(crate) fn new(name_first: String, name_last: String, pos: &Position, rng: &mut ThreadRng) -> Self {
        let target_obp = gen_normal(rng, 0.320, 0.036) * 1000.0;

        let raw_h1b = gen_normal(rng, 96.6, 21.5);
        let raw_h2b = gen_normal(rng, 0.342, 0.137) * raw_h1b;
        let raw_h3b = gen_normal(rng, 0.0985, 0.0666) * raw_h2b;
        let raw_hr = gen_gamma(rng, 1.75, 8.0);
        let raw_bb = gen_normal(rng, 59.44, 18.71);
        let raw_hbp = gen_normal(rng, 4.0, 4.0);

        let obp_total = raw_h1b + raw_h2b + raw_h3b + raw_hr + raw_bb + raw_hbp;
        let h1b = ((raw_h1b / obp_total) * target_obp) as u32;
        let h2b = ((raw_h2b / obp_total) * target_obp) as u32;
        let h3b = ((raw_h3b / obp_total) * target_obp) as u32;
        let hr = ((raw_hr / obp_total) * target_obp) as u32;
        let bb = ((raw_bb / obp_total) * target_obp) as u32;
        let hbp = ((raw_hbp / obp_total) * target_obp) as u32;

        let o = 1000 - target_obp as u32;

        let expect = vec![
            (Stat::H1b, h1b),
            (Stat::H2b, h2b),
            (Stat::H3b, h3b),
            (Stat::Hr, hr),
            (Stat::Bb, bb),
            (Stat::Hbp, hbp),
            (Stat::O, o),
        ];

        let age = 17 + gen_gamma(rng, 2.0, 3.0) as u8;

        let batting_hand = vec![
            (Handedness::Right,54),
            (Handedness::Left,33),
            (Handedness::Switch,13),
        ];
        let bat_hand = &batting_hand.choose_weighted(rng,|o| o.1).unwrap().0;

        let pitching_hand = vec![
            (Handedness::Right,67),
            (Handedness::Left,33),
        ];

        let pitch_hand = &pitching_hand.choose_weighted(rng,|o| o.1).unwrap().0;

        Player {
            name_first,
            name_last,
            age,
            pos: *pos,
            bats: *bat_hand,
            throws: *pitch_hand,
            expect,
            stats: vec![],
            historical: vec![],
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

    pub(crate) fn record_stat_history(&mut self, year: u32, league: u32, team_id: u64) {
        let mut historical = HistoricalStats {
            year,
            league,
            team: team_id,
            ..HistoricalStats::default()
        };
        for stat in &self.stats {
            let val = historical.batting_stats.entry(*stat).or_insert(0);
            *val += 1;
        }
        self.historical.push(historical);

        self.reset_stats()
    }

    pub(crate) fn update_age(&mut self) {
        self.age += 1;
    }

    pub(crate) fn get_expected_pa(&self, rng: &mut ThreadRng) -> Stat {
        self.expect.choose_weighted(rng, |o| o.1).unwrap().0
    }

    pub(crate) fn get_stat(&self, stat: Stat) -> u32 {
        match stat {
            Stat::H1b => self.get_stats().h1b,
            Stat::H2b => self.get_stats().h2b,
            Stat::H3b => self.get_stats().h3b,
            Stat::Hr => self.get_stats().hr,
            Stat::Bb => self.get_stats().bb,
            Stat::Hbp => self.get_stats().hbp,
            Stat::O => self.get_stats().o,
            Stat::R => self.get_stats().r,
            Stat::Rbi => self.get_stats().rbi,
            Stat::H => self.get_stats().h,
            Stat::Ab => self.get_stats().ab,
            Stat::Pa => self.get_stats().pa,
            Stat::Avg => self.get_stats().avg,
            Stat::Obp => self.get_stats().obp,
            Stat::Slg => self.get_stats().slg,
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
        let mut r = 0;
        let mut rbi = 0;

        for stat in &self.stats {
            match stat {
                Stat::H1b => h1b += 1,
                Stat::H2b => h2b += 1,
                Stat::H3b => h3b += 1,
                Stat::Hr => hr += 1,
                Stat::Bb => bb += 1,
                Stat::Hbp => hbp += 1,
                Stat::O => o += 1,
                Stat::R => r += 1,
                Stat::Rbi => rbi += 1,
                _ => {}
            }
        }

        let h = h1b + h2b + h3b + hr;
        let ab = h + o;
        let pa = ab + bb + hbp;
        let avg = calc_avg1000(ab, h);
        let obp = calc_obp1000(pa, h, bb, hbp);
        let slg = calc_slg1000(ab, h1b, h2b, h3b, hr);

        Stats {
            h1b,
            h2b,
            h3b,
            hr,
            bb,
            hbp,
            o,
            r,
            rbi,
            h,
            ab,
            pa,
            avg,
            obp,
            slg,
        }
    }

    pub(crate) fn display_position(&self) -> String {
        self.pos.to_str().to_string()
    }
}
