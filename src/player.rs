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
    B1b,
    B2b,
    B3b,
    Bhr,
    Bbb,
    Bhbp,
    Bo,
    Br,
    Brbi,
    // calculated
    Bh,
    Bab,
    Bpa,
    Bavg,
    Bobp,
    Bslg,
    // recorded
    P1b,
    P2b,
    P3b,
    Phr,
    Pbb,
    Phbp,
    Po,
    Pr,
    Per,
    // calculated
    Ph,
    Pbf,
    Pavg,
    Pobp,
    Pslg,
    Pera,
    Pwhip,
}

pub(crate) fn opposing_stat(stat: Stat) -> Option<Stat> {
    Some(match stat {
        Stat::B1b => Stat::P1b,
        Stat::B2b => Stat::P2b,
        Stat::B3b => Stat::P3b,
        Stat::Bhr => Stat::Phr,
        Stat::Bbb => Stat::Pbb,
        Stat::Bhbp => Stat::Phbp,
        Stat::Bo => Stat::Po,
        _ => return None
    })
}

fn div1000_or_0(n: u32, d: u32) -> u32 {
    if d > 0 { (n * 1000) / d } else { d }
}

fn calc_avg1000(ab: u32, h: u32) -> u32 {
    div1000_or_0(h, ab)
}

fn calc_obp1000(pa: u32, h: u32, bb: u32, hbp: u32) -> u32 {
    div1000_or_0(h + bb + hbp, pa)
}

fn calc_slg1000(ab: u32, h1b: u32, h2b: u32, h3b: u32, hr: u32) -> u32 {
    div1000_or_0(h1b + (2 * h2b) + (3 * h3b) + (4 * hr), ab)
}

fn calc_era1000(er: u32, o: u32) -> u32 {
    div1000_or_0(27 * er, o)
}

fn calc_whip1000(h: u32, bb: u32, o: u32) -> u32 {
    div1000_or_0(3 * (h + bb), o)
}

pub(crate) struct Stats {
    b_1b: u32,
    pub(crate) b_2b: u32,
    pub(crate) b_3b: u32,
    pub(crate) b_hr: u32,
    pub(crate) b_bb: u32,
    pub(crate) b_hbp: u32,
    pub(crate) b_r: u32,
    pub(crate) b_rbi: u32,
    b_o: u32,
    pub(crate) b_h: u32,
    pub(crate) b_ab: u32,
    pub(crate) b_pa: u32,
    pub(crate) b_avg: u32,
    pub(crate) b_obp: u32,
    pub(crate) b_slg: u32,

    p_1b: u32,
    pub(crate) p_2b: u32,
    pub(crate) p_3b: u32,
    pub(crate) p_hr: u32,
    pub(crate) p_bb: u32,
    pub(crate) p_hbp: u32,
    pub(crate) p_r: u32,
    pub(crate) p_er: u32,
    pub(crate) p_o: u32,
    pub(crate) p_h: u32,
    pub(crate) p_bf: u32,
    pub(crate) p_avg: u32,
    pub(crate) p_obp: u32,
    pub(crate) p_slg: u32,
    pub(crate) p_era: u32,
    pub(crate) p_whip: u32,
}

impl Stats {
    pub(crate) fn get_stat(&self, stat: Stat) -> u32 {
        match stat {
            Stat::B1b => self.b_1b,
            Stat::B2b => self.b_2b,
            Stat::B3b => self.b_3b,
            Stat::Bhr => self.b_hr,
            Stat::Bbb => self.b_bb,
            Stat::Bhbp => self.b_hbp,
            Stat::Bo => self.b_o,
            Stat::Br => self.b_r,
            Stat::Brbi => self.b_rbi,
            Stat::Bh => self.b_h,
            Stat::Bab => self.b_ab,
            Stat::Bpa => self.b_pa,
            Stat::Bavg => self.b_avg,
            Stat::Bobp => self.b_obp,
            Stat::Bslg => self.b_slg,
            Stat::P1b => self.p_1b,
            Stat::P2b => self.p_2b,
            Stat::P3b => self.p_3b,
            Stat::Phr => self.p_hr,
            Stat::Pbb => self.p_bb,
            Stat::Phbp => self.p_hbp,
            Stat::Po => self.p_o,
            Stat::Pr => self.p_r,
            Stat::Per => self.p_er,
            Stat::Ph => self.p_h,
            Stat::Pbf => self.p_bf,
            Stat::Pavg => self.p_avg,
            Stat::Pobp => self.p_obp,
            Stat::Pslg => self.p_slg,
            Stat::Pera => self.p_era,
            Stat::Pwhip => self.p_whip,
        }
    }
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
        let b_1b = *self.batting_stats.get(&Stat::B1b).unwrap_or(&0);
        let b_2b = *self.batting_stats.get(&Stat::B2b).unwrap_or(&0);
        let b_3b = *self.batting_stats.get(&Stat::B3b).unwrap_or(&0);
        let b_hr = *self.batting_stats.get(&Stat::Bhr).unwrap_or(&0);
        let b_bb = *self.batting_stats.get(&Stat::Bbb).unwrap_or(&0);
        let b_hbp = *self.batting_stats.get(&Stat::Bhbp).unwrap_or(&0);
        let b_o = *self.batting_stats.get(&Stat::Bo).unwrap_or(&0);
        let b_r = *self.batting_stats.get(&Stat::Br).unwrap_or(&0);
        let b_rbi = *self.batting_stats.get(&Stat::Brbi).unwrap_or(&0);

        let b_h = b_1b + b_2b + b_3b + b_hr;
        let b_ab = b_h + b_o;
        let b_pa = b_ab + b_bb + b_hbp;

        let p_1b = *self.batting_stats.get(&Stat::P1b).unwrap_or(&0);
        let p_2b = *self.batting_stats.get(&Stat::P2b).unwrap_or(&0);
        let p_3b = *self.batting_stats.get(&Stat::P3b).unwrap_or(&0);
        let p_hr = *self.batting_stats.get(&Stat::Phr).unwrap_or(&0);
        let p_bb = *self.batting_stats.get(&Stat::Pbb).unwrap_or(&0);
        let p_hbp = *self.batting_stats.get(&Stat::Phbp).unwrap_or(&0);
        let p_o = *self.batting_stats.get(&Stat::Po).unwrap_or(&0);
        let p_r = *self.batting_stats.get(&Stat::Pr).unwrap_or(&0);
        let p_er = *self.batting_stats.get(&Stat::Per).unwrap_or(&0);

        let p_h = p_1b + p_2b + p_3b + p_hr;
        let p_ab = p_h + p_o;
        let p_bf = p_ab + p_bb + p_hbp;

        Stats {
            b_1b,
            b_2b,
            b_3b,
            b_hr,
            b_bb,
            b_hbp,
            b_r,
            b_rbi,
            b_o,
            b_h,
            b_ab,
            b_pa,
            b_avg: calc_avg1000(b_ab, b_h),
            b_obp: calc_obp1000(b_pa, b_h, b_bb, b_hbp),
            b_slg: calc_slg1000(b_ab, b_1b, b_2b, b_3b, b_hr),
            p_1b,
            p_2b,
            p_3b,
            p_hr,
            p_bb,
            p_hbp,
            p_r,
            p_er,
            p_o,
            p_h,
            p_bf,
            p_avg: calc_avg1000(p_ab, p_h),
            p_obp: calc_obp1000(p_bf, p_h, p_bb, p_hbp),
            p_slg: calc_slg1000(p_ab, p_1b, p_2b, p_3b, p_hr),
            p_era: calc_era1000(p_er, p_o),
            p_whip: calc_whip1000(p_h,p_bb,p_o),
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
    pub(crate) bat_expect: HashMap<Stat, f64>,
    pub(crate) pit_expect: HashMap<Stat, f64>,
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
    fn generate_expect(target_obp: f64, raw_h1b: f64, raw_h2b: f64, raw_h3b: f64, raw_hr: f64, raw_bb: f64, raw_hbp: f64) -> HashMap<Stat, f64> {
        let obp_total = raw_h1b + raw_h2b + raw_h3b + raw_hr + raw_bb + raw_hbp;
        let h1b = (raw_h1b / obp_total) * target_obp;
        let h2b = (raw_h2b / obp_total) * target_obp;
        let h3b = (raw_h3b / obp_total) * target_obp;
        let hr = (raw_hr / obp_total) * target_obp;
        let bb = (raw_bb / obp_total) * target_obp;
        let hbp = (raw_hbp / obp_total) * target_obp;

        let o = 1.0 - target_obp;

        let mut expect = HashMap::new();
        expect.insert(Stat::B1b, h1b);
        expect.insert(Stat::B2b, h2b);
        expect.insert(Stat::B3b, h3b);
        expect.insert(Stat::Bhr, hr);
        expect.insert(Stat::Bbb, bb);
        expect.insert(Stat::Bhbp, hbp);
        expect.insert(Stat::Bo, o);
        expect
    }

    fn generate_bat_expect(rng: &mut ThreadRng) -> HashMap<Stat, f64> {
        let target_obp = gen_normal(rng, 0.320, 0.036);

        let raw_h1b = gen_normal(rng, 96.6, 21.5);
        let raw_h2b = gen_normal(rng, 0.342, 0.137) * raw_h1b;
        let raw_h3b = gen_normal(rng, 0.0985, 0.0666) * raw_h2b;
        let raw_hr = gen_gamma(rng, 1.75, 9.0);
        let raw_bb = gen_normal(rng, 59.44, 18.71);
        let raw_hbp = gen_normal(rng, 4.0, 4.0);

        Player::generate_expect(target_obp, raw_h1b, raw_h2b, raw_h3b, raw_hr, raw_bb, raw_hbp)
    }

    fn generate_pit_expect(rng: &mut ThreadRng) -> HashMap<Stat, f64> {
        let target_obp = gen_normal(rng, 0.321, 0.039);

        let raw_h1b = gen_normal(rng, 96.6, 21.5);
        let raw_h2b = gen_normal(rng, 0.342, 0.137) * raw_h1b;
        let raw_h3b = gen_normal(rng, 0.0985, 0.0666) * raw_h2b;
        let raw_hr = gen_normal(rng, 12.812, 8.058196141);
        let raw_bb = gen_normal(rng, 29.45, 15.42658287);
        let raw_hbp = gen_normal(rng, 3.624, 2.946181252);

        Player::generate_expect(target_obp, raw_h1b, raw_h2b, raw_h3b, raw_hr, raw_bb, raw_hbp)
    }

    pub(crate) fn new(name_first: String, name_last: String, pos: &Position, rng: &mut ThreadRng) -> Self {
        let age = 17 + gen_gamma(rng, 2.0, 3.0) as u8;

        let batting_hand = vec![
            (Handedness::Right, 54),
            (Handedness::Left, 33),
            (Handedness::Switch, 13),
        ];
        let bat_hand = &batting_hand.choose_weighted(rng, |o| o.1).unwrap().0;

        let pitching_hand = vec![
            (Handedness::Right, 67),
            (Handedness::Left, 33),
        ];

        let pitch_hand = &pitching_hand.choose_weighted(rng, |o| o.1).unwrap().0;

        let bat_expect = Player::generate_bat_expect(rng);
        let pit_expect = Player::generate_pit_expect(rng);

        Player {
            name_first,
            name_last,
            age,
            pos: *pos,
            bats: *bat_hand,
            throws: *pitch_hand,
            bat_expect,
            pit_expect,
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

    pub(crate) fn get_stats(&self) -> Stats {
        let mut b_1b = 0;
        let mut b_2b = 0;
        let mut b_3b = 0;
        let mut b_hr = 0;
        let mut b_bb = 0;
        let mut b_hbp = 0;
        let mut b_o = 0;
        let mut b_r = 0;
        let mut b_rbi = 0;
        let mut p_1b = 0;
        let mut p_2b = 0;
        let mut p_3b = 0;
        let mut p_hr = 0;
        let mut p_bb = 0;
        let mut p_hbp = 0;
        let mut p_o = 0;
        let mut p_r = 0;
        let mut p_er = 0;

        for stat in &self.stats {
            match stat {
                Stat::B1b => b_1b += 1,
                Stat::B2b => b_2b += 1,
                Stat::B3b => b_3b += 1,
                Stat::Bhr => b_hr += 1,
                Stat::Bbb => b_bb += 1,
                Stat::Bhbp => b_hbp += 1,
                Stat::Bo => b_o += 1,
                Stat::Br => b_r += 1,
                Stat::Brbi => b_rbi += 1,
                Stat::P1b => p_1b += 1,
                Stat::P2b => p_2b += 1,
                Stat::P3b => p_3b += 1,
                Stat::Phr => p_hr += 1,
                Stat::Pbb => p_bb += 1,
                Stat::Phbp => p_hbp += 1,
                Stat::Po => p_o += 1,
                Stat::Pr => p_r += 1,
                Stat::Per => p_er += 1,
                _ => {}
            }
        }

        let b_h = b_1b + b_2b + b_3b + b_hr;
        let b_ab = b_h + b_o;
        let b_pa = b_ab + b_bb + b_hbp;
        let b_avg = calc_avg1000(b_ab, b_h);
        let b_obp = calc_obp1000(b_pa, b_h, b_bb, b_hbp);
        let b_slg = calc_slg1000(b_ab, b_1b, b_2b, b_3b, b_hr);


        let p_h = p_1b + p_2b + p_3b + p_hr;
        let p_ab = p_h + p_o;
        let p_bf = p_ab + p_bb + p_hbp;
        let p_avg = calc_avg1000(p_ab, p_h);
        let p_obp = calc_obp1000(p_bf, p_h, p_bb, p_hbp);
        let p_slg = calc_slg1000(p_ab, p_1b, p_2b, p_3b, p_hr);
        let p_era = calc_era1000(p_er, p_o);
        let p_whip = calc_whip1000(p_h, p_bb, p_o);

        Stats {
            b_1b,
            b_2b,
            b_3b,
            b_hr,
            b_bb,
            b_hbp,
            b_o,
            b_r,
            b_rbi,
            b_h,
            b_ab,
            b_pa,
            b_avg,
            b_obp,
            b_slg,
            p_1b,
            p_2b,
            p_3b,
            p_hr,
            p_bb,
            p_hbp,
            p_r,
            p_er,
            p_o,
            p_h,
            p_bf,
            p_avg,
            p_obp,
            p_slg,
            p_era,
            p_whip,
        }
    }
}
