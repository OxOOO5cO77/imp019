use std::collections::HashMap;

use enum_iterator::IntoEnumIterator;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand_distr::{Distribution, Gamma, Normal};

use crate::data::Data;

#[derive(Copy, Clone, PartialEq, IntoEnumIterator)]
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

impl Default for Position {
    fn default() -> Self {
        Position::Pitcher
    }
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
    G,
    Gs,
    // recorded
    B1b,
    B2b,
    B3b,
    Bhr,
    Bbb,
    Bhbp,
    Bso,
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
    Pso,
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
    pub(crate) g: u32,
    pub(crate) gs: u32,
    b_1b: u32,
    pub(crate) b_2b: u32,
    pub(crate) b_3b: u32,
    pub(crate) b_hr: u32,
    pub(crate) b_bb: u32,
    pub(crate) b_hbp: u32,
    pub(crate) b_r: u32,
    pub(crate) b_rbi: u32,
    pub(crate) b_so: u32,
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
    pub(crate) p_so: u32,
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
            Stat::G => self.g,
            Stat::Gs => self.gs,
            Stat::B1b => self.b_1b,
            Stat::B2b => self.b_2b,
            Stat::B3b => self.b_3b,
            Stat::Bhr => self.b_hr,
            Stat::Bbb => self.b_bb,
            Stat::Bhbp => self.b_hbp,
            Stat::Bso => self.b_so,
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
            Stat::Pso => self.p_so,
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
        let g = *self.batting_stats.get(&Stat::G).unwrap_or(&0);
        let gs = *self.batting_stats.get(&Stat::Gs).unwrap_or(&0);

        let b_1b = *self.batting_stats.get(&Stat::B1b).unwrap_or(&0);
        let b_2b = *self.batting_stats.get(&Stat::B2b).unwrap_or(&0);
        let b_3b = *self.batting_stats.get(&Stat::B3b).unwrap_or(&0);
        let b_hr = *self.batting_stats.get(&Stat::Bhr).unwrap_or(&0);
        let b_bb = *self.batting_stats.get(&Stat::Bbb).unwrap_or(&0);
        let b_hbp = *self.batting_stats.get(&Stat::Bhbp).unwrap_or(&0);
        let b_so = *self.batting_stats.get(&Stat::Bso).unwrap_or(&0);
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
        let p_so = *self.batting_stats.get(&Stat::Pso).unwrap_or(&0);
        let p_o = *self.batting_stats.get(&Stat::Po).unwrap_or(&0);
        let p_r = *self.batting_stats.get(&Stat::Pr).unwrap_or(&0);
        let p_er = *self.batting_stats.get(&Stat::Per).unwrap_or(&0);

        let p_h = p_1b + p_2b + p_3b + p_hr;
        let p_ab = p_h + p_o;
        let p_bf = p_ab + p_bb + p_hbp;

        Stats {
            g,
            gs,
            b_1b,
            b_2b,
            b_3b,
            b_hr,
            b_bb,
            b_hbp,
            b_r,
            b_rbi,
            b_so,
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
            p_so,
            p_o,
            p_h,
            p_bf,
            p_avg: calc_avg1000(p_ab, p_h),
            p_obp: calc_obp1000(p_bf, p_h, p_bb, p_hbp),
            p_slg: calc_slg1000(p_ab, p_1b, p_2b, p_3b, p_hr),
            p_era: calc_era1000(p_er, p_o),
            p_whip: calc_whip1000(p_h, p_bb, p_o),
        }
    }
}

pub(crate) struct Player {
    pub(crate) active: bool,
    name_first: String,
    name_last: String,
    pub(crate) born: u32,
    pub(crate) pos: Position,
    pub(crate) bats: Handedness,
    pub(crate) throws: Handedness,
    pub(crate) bat_expect: HashMap<Expect, f64>,
    pub(crate) pit_expect: HashMap<Expect, f64>,
    stat_stream: Vec<Stat>,
    pub(crate) historical: Vec<HistoricalStats>,
    pub(crate) fatigue: u16,
}

fn gen_normal(rng: &mut ThreadRng, mean: f64, stddev: f64) -> f64 {
    Normal::new(mean, stddev).unwrap().sample(rng)
}

fn gen_gamma(rng: &mut ThreadRng, shape: f64, scale: f64) -> f64 {
    Gamma::new(shape, scale).unwrap().sample(rng)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Expect {
    Single,
    Double,
    Triple,
    HomeRun,
    Walk,
    HitByPitch,
    Strikeout,
    Out,
}

impl Expect {
    pub(crate) fn to_batting_stat(&self) -> Stat {
        match self {
            Expect::Single => Stat::B1b,
            Expect::Double => Stat::B2b,
            Expect::Triple => Stat::B3b,
            Expect::HomeRun => Stat::Bhr,
            Expect::Walk => Stat::Bbb,
            Expect::HitByPitch => Stat::Bhbp,
            Expect::Strikeout => Stat::Bso,
            Expect::Out => Stat::Bo,
        }
    }
    pub(crate) fn to_pitching_stat(&self) -> Stat {
        match self {
            Expect::Single => Stat::P1b,
            Expect::Double => Stat::P2b,
            Expect::Triple => Stat::P3b,
            Expect::HomeRun => Stat::Phr,
            Expect::Walk => Stat::Pbb,
            Expect::HitByPitch => Stat::Phbp,
            Expect::Strikeout => Stat::Pso,
            Expect::Out => Stat::Po,
        }
    }
}

struct ExpectPct {
    target_obp: f64,
    h1b: f64,
    h2b: f64,
    h3b: f64,
    hr: f64,
    bb: f64,
    hbp: f64,
    so: f64,
}

impl Player {
    fn generate_expect(expect_pct: ExpectPct) -> HashMap<Expect, f64> {
        let obp_total = expect_pct.h1b + expect_pct.h2b + expect_pct.h3b + expect_pct.hr + expect_pct.bb + expect_pct.hbp;
        let h1b = (expect_pct.h1b / obp_total) * expect_pct.target_obp;
        let h2b = (expect_pct.h2b / obp_total) * expect_pct.target_obp;
        let h3b = (expect_pct.h3b / obp_total) * expect_pct.target_obp;
        let hr = (expect_pct.hr / obp_total) * expect_pct.target_obp;
        let bb = (expect_pct.bb / obp_total) * expect_pct.target_obp;
        let hbp = (expect_pct.hbp / obp_total) * expect_pct.target_obp;

        let so = expect_pct.so;
        let o = 1.0 - expect_pct.target_obp - so;

        let mut expect = HashMap::new();
        expect.insert(Expect::Single, h1b);
        expect.insert(Expect::Double, h2b);
        expect.insert(Expect::Triple, h3b);
        expect.insert(Expect::HomeRun, hr);
        expect.insert(Expect::Walk, bb);
        expect.insert(Expect::HitByPitch, hbp);
        expect.insert(Expect::Strikeout, so);
        expect.insert(Expect::Out, o);

        expect
    }

    fn generate_bat_expect(rng: &mut ThreadRng) -> HashMap<Expect, f64> {
        let target_obp = gen_normal(rng, 0.320, 0.036);
        let h1b = gen_normal(rng, 96.6, 21.5);
        let h2b = gen_normal(rng, 0.342, 0.137) * h1b;
        let h3b = gen_normal(rng, 0.0985, 0.0666) * h2b;
        let hr = gen_gamma(rng, 1.75, 9.0);
        let bb = gen_normal(rng, 59.44, 18.71);
        let hbp = gen_normal(rng, 4.0, 4.0);
        let so = gen_normal(rng, 0.1914556061, 0.02597102753);

        let expect = ExpectPct {
            target_obp,
            h1b,
            h2b,
            h3b,
            hr,
            bb,
            hbp,
            so,
        };

        Player::generate_expect(expect)
    }

    fn generate_pit_expect(rng: &mut ThreadRng) -> HashMap<Expect, f64> {
        let target_obp = gen_normal(rng, 0.321, 0.039);
        let h1b = gen_normal(rng, 96.6, 21.5);
        let h2b = gen_normal(rng, 0.342, 0.137) * h1b;
        let h3b = gen_normal(rng, 0.0985, 0.0666) * h2b;
        let hr = gen_normal(rng, 12.812, 8.058196141);
        let bb = gen_normal(rng, 29.45, 15.42658287);
        let hbp = gen_normal(rng, 3.624, 2.946181252);
        let so = gen_normal(rng, 0.1928022279, 0.02819196439);

        let expect = ExpectPct {
            target_obp,
            h1b,
            h2b,
            h3b,
            hr,
            bb,
            hbp,
            so,
        };

        Player::generate_expect(expect)
    }

    pub(crate) fn new(name_first: String, name_last: String, pos: &Position, year: u32, rng: &mut ThreadRng) -> Self {
        let age = 18 + gen_gamma(rng, 2.0, 3.0).round() as u32;

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
            active: true,
            name_first,
            name_last,
            born: year - age,
            pos: *pos,
            bats: *bat_hand,
            throws: *pitch_hand,
            bat_expect,
            pit_expect,
            stat_stream: vec![],
            historical: vec![],
            fatigue: 0,
        }
    }

    pub(crate) fn fullname(&self) -> String {
        format!("{} {}", self.name_first, self.name_last)
    }

    fn reset_stats(&mut self) {
        self.stat_stream.clear();
    }

    pub(crate) fn record_stat(&mut self, stat: Stat) {
        self.stat_stream.push(stat);
    }

    pub(crate) fn record_stat_history(&mut self, year: u32, league: u32, team_id: u64) {
        let mut historical = HistoricalStats {
            year,
            league,
            team: team_id,
            ..HistoricalStats::default()
        };
        for stat in &self.stat_stream {
            let val = historical.batting_stats.entry(*stat).or_insert(0);
            *val += 1;
        }
        self.historical.push(historical);

        self.reset_stats()
    }


    pub(crate) fn get_stats(&self) -> Stats {
        let mut g = 0;
        let mut gs = 0;
        let mut b_1b = 0;
        let mut b_2b = 0;
        let mut b_3b = 0;
        let mut b_hr = 0;
        let mut b_bb = 0;
        let mut b_hbp = 0;
        let mut b_so = 0;
        let mut b_o = 0;
        let mut b_r = 0;
        let mut b_rbi = 0;
        let mut p_1b = 0;
        let mut p_2b = 0;
        let mut p_3b = 0;
        let mut p_hr = 0;
        let mut p_bb = 0;
        let mut p_hbp = 0;
        let mut p_so = 0;
        let mut p_o = 0;
        let mut p_r = 0;
        let mut p_er = 0;

        for stat in &self.stat_stream {
            match stat {
                Stat::G => g += 1,
                Stat::Gs => gs += 1,
                Stat::B1b => b_1b += 1,
                Stat::B2b => b_2b += 1,
                Stat::B3b => b_3b += 1,
                Stat::Bhr => b_hr += 1,
                Stat::Bbb => b_bb += 1,
                Stat::Bhbp => b_hbp += 1,
                Stat::Bso => b_so += 1,
                Stat::Bo => b_o += 1,
                Stat::Br => b_r += 1,
                Stat::Brbi => b_rbi += 1,
                Stat::P1b => p_1b += 1,
                Stat::P2b => p_2b += 1,
                Stat::P3b => p_3b += 1,
                Stat::Phr => p_hr += 1,
                Stat::Pbb => p_bb += 1,
                Stat::Phbp => p_hbp += 1,
                Stat::Pso => p_so += 1,
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
            g,
            gs,
            b_1b,
            b_2b,
            b_3b,
            b_hr,
            b_bb,
            b_hbp,
            b_so,
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
            p_so,
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

    pub(crate) fn age(&self, year: u32) -> u32 {
        year - self.born
    }

    pub(crate) fn fatigue_threshold(&self, year: u32) -> f64 {
        let mut age_factor = (50u64 - self.age(year).min(49) as u64) * 2;
        age_factor = age_factor * age_factor;
        age_factor as f64
    }

    pub(crate) fn should_retire(&self, year: u32, rng: &mut ThreadRng) -> bool {
        const MIN_AGE: u32 = 30;
        const MAX_AGE: u32 = 45;
        let age_factor = self.age(year).clamp(MIN_AGE, MAX_AGE) - MIN_AGE;
        let n = (age_factor * age_factor) as f64;
        let d = ((MAX_AGE - MIN_AGE) * (MAX_AGE - MIN_AGE)) as f64;
        rng.gen_bool(n / d)
    }
}

pub(crate) fn generate_players(players: &mut HashMap<u64, Player>, count: usize, year: u32, data: &Data, rng: &mut ThreadRng) {
    let pos_gen = vec![
        Position::Pitcher,
        Position::Pitcher,
        Position::Pitcher,
        Position::Pitcher,
        Position::Pitcher,
        Position::Pitcher,
        Position::Pitcher,
        Position::Pitcher,
        Position::Pitcher,
        Position::Catcher,
        Position::FirstBase,
        Position::SecondBase,
        Position::ThirdBase,
        Position::ShortStop,
        Position::LeftField,
        Position::CenterField,
        Position::RightField,
        Position::DesignatedHitter,
    ];

    let mut player_id = players.keys().max().unwrap_or(&0) + 1;
    if players.len() < count {
        players.reserve(count - players.len());
    }

    for _ in 0..count {
        let name_first = data.choose_name_first(rng);
        let name_last = data.choose_name_last(rng);
        players.insert(player_id, Player::new(name_first, name_last, pos_gen.choose(rng).unwrap(), year, rng));
        player_id += 1;
    }
}
