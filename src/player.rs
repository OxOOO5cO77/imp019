use std::collections::HashMap;

use enum_iterator::IntoEnumIterator;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand_distr::{Distribution, Gamma, Normal};

use crate::data::Data;
use crate::team::TeamId;

pub(crate) type PlayerId = u64;
pub(crate) type PlayerMap = HashMap<PlayerId, Player>;
pub(crate) type PlayerRefMap<'a> = HashMap<PlayerId, &'a Player>;

#[derive(Copy, Clone, PartialEq, Eq, Hash, IntoEnumIterator)]
pub(crate) enum Position {
    StartingPitcher,
    Catcher,
    FirstBase,
    SecondBase,
    ThirdBase,
    ShortStop,
    LeftField,
    CenterField,
    RightField,
    DesignatedHitter,
    LongRelief,
    ShortRelief,
    Setup,
    Closer,
}

impl Default for Position {
    fn default() -> Self {
        Self::StartingPitcher
    }
}

impl Position {
    pub(crate) fn to_str(&self) -> &str {
        match self {
            Position::StartingPitcher => "SP",
            Position::Catcher => "C",
            Position::FirstBase => "1B",
            Position::SecondBase => "2B",
            Position::ThirdBase => "3B",
            Position::ShortStop => "SS",
            Position::LeftField => "LF",
            Position::CenterField => "CF",
            Position::RightField => "RF",
            Position::DesignatedHitter => "DH",
            Position::LongRelief => "LR",
            Position::ShortRelief => "SR",
            Position::Setup => "SU",
            Position::Closer => "CL",
        }
    }

    pub(crate) fn is_pitcher(&self) -> bool {
        matches!(self,
            Position::StartingPitcher |
            Position::LongRelief |
            Position::ShortRelief |
            Position::Setup |
            Position::Closer
        )
    }

    pub(crate) fn is_infield(&self) -> bool {
        matches!(self,
            Position::FirstBase |
            Position::SecondBase |
            Position::ThirdBase |
            Position::ShortStop
        )
    }

    pub(crate) fn is_oufield(&self) -> bool {
        matches!(self,
            Position::LeftField |
            Position::CenterField |
            Position::RightField
        )
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
    // recorded
    Fpo,
    Fe,
}

#[derive(Default)]
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

    pub(crate) f_po: u32,
    pub(crate) f_e: u32,
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
            Stat::Fpo => self.f_po,
            Stat::Fe => self.f_e,
        }
    }

    fn div1000_or_0(n: u32, d: u32) -> u32 {
        if d > 0 { (n * 1000) / d } else { d }
    }

    fn calc_avg1000(ab: u32, h: u32) -> u32 {
        Self::div1000_or_0(h, ab)
    }

    fn calc_obp1000(pa: u32, h: u32, bb: u32, hbp: u32) -> u32 {
        Self::div1000_or_0(h + bb + hbp, pa)
    }

    fn calc_slg1000(ab: u32, h1b: u32, h2b: u32, h3b: u32, hr: u32) -> u32 {
        Self::div1000_or_0(h1b + (2 * h2b) + (3 * h3b) + (4 * hr), ab)
    }

    fn calc_era1000(er: u32, o: u32) -> u32 {
        Self::div1000_or_0(27 * er, o)
    }

    fn calc_whip1000(h: u32, bb: u32, o: u32) -> u32 {
        Self::div1000_or_0(3 * (h + bb), o)
    }

    fn calculate(&mut self) {
        self.b_h = self.b_1b + self.b_2b + self.b_3b + self.b_hr;
        self.b_ab = self.b_h + self.b_o;
        self.b_pa = self.b_ab + self.b_bb + self.b_hbp;

        self.b_avg = Self::calc_avg1000(self.b_ab, self.b_h);
        self.b_obp = Self::calc_obp1000(self.b_pa, self.b_h, self.b_bb, self.b_hbp);
        self.b_slg = Self::calc_slg1000(self.b_ab, self.b_1b, self.b_2b, self.b_3b, self.b_hr);


        self.p_h = self.p_1b + self.p_2b + self.p_3b + self.p_hr;
        let p_ab = self.p_h + self.p_o;
        self.p_bf = p_ab + self.p_bb + self.p_hbp;

        self.p_avg = Self::calc_avg1000(p_ab, self.p_h);
        self.p_obp = Self::calc_obp1000(self.p_bf, self.p_h, self.p_bb, self.p_hbp);
        self.p_slg = Self::calc_slg1000(p_ab, self.p_1b, self.p_2b, self.p_3b, self.p_hr);
        self.p_era = Self::calc_era1000(self.p_er, self.p_o);
        self.p_whip = Self::calc_whip1000(self.p_h, self.p_bb, self.p_o);
    }
}

#[derive(Default)]
pub(crate) struct HistoricalStats {
    pub(crate) year: u32,
    pub(crate) league: u32,
    pub(crate) team: TeamId,
    pub(crate) stats: HashMap<Stat, u32>,
}

impl HistoricalStats {
    pub(crate) fn get_stats(&self) -> Stats {
        let mut stats = Stats {
            g: *self.stats.get(&Stat::G).unwrap_or(&0),
            gs: *self.stats.get(&Stat::Gs).unwrap_or(&0),
            b_1b: *self.stats.get(&Stat::B1b).unwrap_or(&0),
            b_2b: *self.stats.get(&Stat::B2b).unwrap_or(&0),
            b_3b: *self.stats.get(&Stat::B3b).unwrap_or(&0),
            b_hr: *self.stats.get(&Stat::Bhr).unwrap_or(&0),
            b_bb: *self.stats.get(&Stat::Bbb).unwrap_or(&0),
            b_hbp: *self.stats.get(&Stat::Bhbp).unwrap_or(&0),
            b_so: *self.stats.get(&Stat::Bso).unwrap_or(&0),
            b_o: *self.stats.get(&Stat::Bo).unwrap_or(&0),
            b_r: *self.stats.get(&Stat::Br).unwrap_or(&0),
            b_rbi: *self.stats.get(&Stat::Brbi).unwrap_or(&0),
            p_1b: *self.stats.get(&Stat::P1b).unwrap_or(&0),
            p_2b: *self.stats.get(&Stat::P2b).unwrap_or(&0),
            p_3b: *self.stats.get(&Stat::P3b).unwrap_or(&0),
            p_hr: *self.stats.get(&Stat::Phr).unwrap_or(&0),
            p_bb: *self.stats.get(&Stat::Pbb).unwrap_or(&0),
            p_hbp: *self.stats.get(&Stat::Phbp).unwrap_or(&0),
            p_so: *self.stats.get(&Stat::Pso).unwrap_or(&0),
            p_o: *self.stats.get(&Stat::Po).unwrap_or(&0),
            p_r: *self.stats.get(&Stat::Pr).unwrap_or(&0),
            p_er: *self.stats.get(&Stat::Per).unwrap_or(&0),
            f_po: *self.stats.get(&Stat::Fpo).unwrap_or(&0),
            f_e: *self.stats.get(&Stat::Fe).unwrap_or(&0),
            ..Stats::default()
        };

        stats.calculate();

        stats
    }
}

pub(crate) type ExpectMap = HashMap<Expect, f64>;
type SprayChart = HashMap<Expect, HashMap<Position, u32>>;

pub(crate) struct Player {
    pub(crate) active: bool,
    name_first: String,
    name_last: String,
    pub(crate) born: u32,
    pub(crate) pos: Position,
    pub(crate) bats: Handedness,
    pub(crate) throws: Handedness,
    pub(crate) bat_expect: (ExpectMap, ExpectMap),
    pub(crate) bat_spray: SprayChart,
    pub(crate) pit_expect: (ExpectMap, ExpectMap),
    pub(crate) pit_spray: SprayChart,
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
    Error,
}

impl Expect {
    pub(crate) fn to_batting_stat(&self) -> Stat {
        match self {
            Self::Single => Stat::B1b,
            Self::Double => Stat::B2b,
            Self::Triple => Stat::B3b,
            Self::HomeRun => Stat::Bhr,
            Self::Walk => Stat::Bbb,
            Self::HitByPitch => Stat::Bhbp,
            Self::Strikeout => Stat::Bso,
            Self::Out => Stat::Bo,
            Self::Error => Stat::Bo,
        }
    }
    pub(crate) fn to_pitching_stat(&self) -> Stat {
        match self {
            Self::Single => Stat::P1b,
            Self::Double => Stat::P2b,
            Self::Triple => Stat::P3b,
            Self::HomeRun => Stat::Phr,
            Self::Walk => Stat::Pbb,
            Self::HitByPitch => Stat::Phbp,
            Self::Strikeout => Stat::Pso,
            Self::Out => Stat::Po,
            Self::Error => Stat::Po,
        }
    }
}

struct ExpectRaw {
    target_obp: f64,
    h1b: f64,
    h2b: f64,
    h3b: f64,
    hr: f64,
    bb: f64,
    hbp: f64,
    so: f64,
    e: f64,
}

impl Player {
    fn generate_expect(expect_pct: ExpectRaw) -> ExpectMap {
        let obp_total = expect_pct.h1b + expect_pct.h2b + expect_pct.h3b + expect_pct.hr + expect_pct.bb + expect_pct.hbp;
        let h1b = (expect_pct.h1b / obp_total) * expect_pct.target_obp;
        let h2b = (expect_pct.h2b / obp_total) * expect_pct.target_obp;
        let h3b = (expect_pct.h3b / obp_total) * expect_pct.target_obp;
        let hr = (expect_pct.hr / obp_total) * expect_pct.target_obp;
        let bb = (expect_pct.bb / obp_total) * expect_pct.target_obp;
        let hbp = (expect_pct.hbp / obp_total) * expect_pct.target_obp;

        let so = expect_pct.so;
        let o = 1.0 - expect_pct.target_obp - so;
        let e = expect_pct.e;

        let mut expect = HashMap::new();
        expect.insert(Expect::Single, h1b);
        expect.insert(Expect::Double, h2b);
        expect.insert(Expect::Triple, h3b);
        expect.insert(Expect::HomeRun, hr);
        expect.insert(Expect::Walk, bb);
        expect.insert(Expect::HitByPitch, hbp);
        expect.insert(Expect::Strikeout, so);
        expect.insert(Expect::Out, o);
        expect.insert(Expect::Error, e);

        expect
    }

    fn generate_bat_expect(rng: &mut ThreadRng) -> ExpectMap {
        let target_obp = gen_normal(rng, 0.320, 0.036);

        let h1b = gen_gamma(rng, 4.89051721563733, 19.7826596218742);
        let h2b = gen_normal(rng, 0.342, 0.137) * h1b;
        let h3b = gen_normal(rng, 0.0985, 0.0666) * h2b;
        let hr = gen_gamma(rng, 12.2812930750413, 2.09953872829662);
        let bb = gen_gamma(rng, 8.34381266257955, 7.16855765752819);
        let hbp = gen_gamma(rng, 18.8629868507638, 0.404463971747468);
        let so = gen_normal(rng, 0.1914556061, 0.02597102753);
        let e = (1.0 - gen_normal(rng, 0.9765828221, 0.9765828221).clamp(0.0, 1.0)) / 3.0;

        let expect = ExpectRaw {
            target_obp,
            h1b,
            h2b,
            h3b,
            hr,
            bb,
            hbp,
            so,
            e,
        };

        Self::generate_expect(expect)
    }

    fn generate_pit_expect(rng: &mut ThreadRng) -> ExpectMap {
        let target_obp = gen_normal(rng, 0.321, 0.039);
        let h = gen_gamma(rng, 3.58229424925063, 43.691697161455);
        let h2b = gen_normal(rng, 0.342, 0.137) * h;
        let h3b = gen_normal(rng, 0.0985, 0.0666) * h2b;
        let h1b = h - h2b - h3b;
        let hr = gen_gamma(rng, 3.30666140034948, 7.53788040691485);
        let bb = gen_gamma(rng, 6.64203372642545, 9.13486625765644);
        let hbp = gen_gamma(rng, 19.9583780886045, 0.390444942208961);
        let so = gen_normal(rng, 0.1928022279, 0.02819196439);

        let expect = ExpectRaw {
            target_obp,
            h1b,
            h2b,
            h3b,
            hr,
            bb,
            hbp,
            so,
            e: 0.0,
        };

        Self::generate_expect(expect)
    }


    fn normalize(hashmap: &mut HashMap<Position, u32>) {
        let sum = hashmap.iter().map(|(_, v)| v).sum::<u32>();
        for (_, val) in hashmap.iter_mut() {
            *val = (*val * 1000) / sum;
        }
    }

    fn generate_bat_spray(rng: &mut ThreadRng, pos: &Position) -> SprayChart {
        let mut spray = SprayChart::new();

        if !pos.is_pitcher() {
            let mut single = HashMap::new();
            single.insert(Position::StartingPitcher, rng.gen_range(0..3));
            single.insert(Position::Catcher, rng.gen_range(0..3));
            single.insert(Position::FirstBase, rng.gen_range(0..3));
            single.insert(Position::SecondBase, rng.gen_range(10..20));
            single.insert(Position::ThirdBase, rng.gen_range(10..20));
            single.insert(Position::ShortStop, rng.gen_range(10..20));
            single.insert(Position::LeftField, rng.gen_range(100..200));
            single.insert(Position::CenterField, rng.gen_range(100..200));
            single.insert(Position::RightField, rng.gen_range(100..200));
            Self::normalize(&mut single);

            let mut double = HashMap::new();
            double.insert(Position::LeftField, rng.gen_range(100..200));
            double.insert(Position::CenterField, rng.gen_range(100..200));
            double.insert(Position::RightField, rng.gen_range(100..200));
            Self::normalize(&mut double);

            let mut triple = HashMap::new();
            triple.insert(Position::LeftField, rng.gen_range(100..200));
            triple.insert(Position::CenterField, rng.gen_range(100..200));
            triple.insert(Position::RightField, rng.gen_range(100..200));
            Self::normalize(&mut triple);

            let mut homerun = HashMap::new();
            homerun.insert(Position::LeftField, rng.gen_range(100..200));
            homerun.insert(Position::CenterField, rng.gen_range(100..200));
            homerun.insert(Position::RightField, rng.gen_range(100..200));
            Self::normalize(&mut homerun);

            let mut out = HashMap::new();
            out.insert(Position::StartingPitcher, 5);
            out.insert(Position::Catcher, 5);
            out.insert(Position::FirstBase, 10);
            out.insert(Position::SecondBase, 10);
            out.insert(Position::ThirdBase, 10);
            out.insert(Position::ShortStop, 10);
            out.insert(Position::LeftField, 10);
            out.insert(Position::CenterField, 10);
            out.insert(Position::RightField, 10);
            Self::normalize(&mut out);

            spray.insert(Expect::Single, single);
            spray.insert(Expect::Double, double);
            spray.insert(Expect::Triple, triple);
            spray.insert(Expect::HomeRun, homerun);
            spray.insert(Expect::Out, out);
        } else {}

        spray
    }

    fn generate_pit_spray(rng: &mut ThreadRng, pos: &Position) -> SprayChart {
        let mut spray = SprayChart::new();

        if pos.is_pitcher() {
            let mut single = HashMap::new();
            single.insert(Position::StartingPitcher, rng.gen_range(0..3));
            single.insert(Position::Catcher, rng.gen_range(0..3));
            single.insert(Position::FirstBase, rng.gen_range(0..3));
            single.insert(Position::SecondBase, rng.gen_range(10..20));
            single.insert(Position::ThirdBase, rng.gen_range(10..20));
            single.insert(Position::ShortStop, rng.gen_range(10..20));
            single.insert(Position::LeftField, rng.gen_range(100..200));
            single.insert(Position::CenterField, rng.gen_range(100..200));
            single.insert(Position::RightField, rng.gen_range(100..200));
            Self::normalize(&mut single);

            let mut double = HashMap::new();
            double.insert(Position::LeftField, rng.gen_range(100..200));
            double.insert(Position::CenterField, rng.gen_range(100..200));
            double.insert(Position::RightField, rng.gen_range(100..200));
            Self::normalize(&mut double);

            let mut triple = HashMap::new();
            triple.insert(Position::LeftField, rng.gen_range(100..200));
            triple.insert(Position::CenterField, rng.gen_range(100..200));
            triple.insert(Position::RightField, rng.gen_range(100..200));
            Self::normalize(&mut triple);

            let mut homerun = HashMap::new();
            homerun.insert(Position::LeftField, rng.gen_range(100..200));
            homerun.insert(Position::CenterField, rng.gen_range(100..200));
            homerun.insert(Position::RightField, rng.gen_range(100..200));
            Self::normalize(&mut homerun);

            let mut out = HashMap::new();
            out.insert(Position::StartingPitcher, 5);
            out.insert(Position::Catcher, 5);
            out.insert(Position::FirstBase, 10);
            out.insert(Position::SecondBase, 10);
            out.insert(Position::ThirdBase, 10);
            out.insert(Position::ShortStop, 10);
            out.insert(Position::LeftField, 10);
            out.insert(Position::CenterField, 10);
            out.insert(Position::RightField, 10);
            Self::normalize(&mut out);

            spray.insert(Expect::Single, single);
            spray.insert(Expect::Double, double);
            spray.insert(Expect::Triple, triple);
            spray.insert(Expect::HomeRun, homerun);
            spray.insert(Expect::Out, out);
        }

        spray
    }

    pub(crate) fn determine_spray(bat: &SprayChart, pit: &SprayChart, expect: &Expect, rng: &mut ThreadRng) -> Position {
        let merged = bat.iter().chain(pit).collect::<HashMap<_, _>>();
        if let Some(expect_spray) = merged.get(expect) {
            *expect_spray.iter()
                .collect::<Vec<(_, _)>>()
                .choose_weighted(rng, |o| o.1)
                .unwrap().0
        } else {
            Position::CenterField
        }
    }

    pub(crate) fn check_for_e(&self, rng: &mut ThreadRng) -> bool {
        rng.gen_bool(*self.bat_expect.0.get(&Expect::Error).unwrap())
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


        let bat_expect = (Self::generate_bat_expect(rng), Self::generate_bat_expect(rng));
        let pit_expect = (Self::generate_pit_expect(rng), Self::generate_pit_expect(rng));

        let bat_spray = Self::generate_bat_spray(rng, pos);
        let pit_spray = Self::generate_pit_spray(rng, pos);

        Self {
            active: true,
            name_first,
            name_last,
            born: year - age,
            pos: *pos,
            bats: *bat_hand,
            throws: *pitch_hand,
            bat_expect,
            bat_spray,
            pit_expect,
            pit_spray,
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

    pub(crate) fn record_stat_history(&mut self, year: u32, league: u32, team_id: TeamId) {
        let mut historical = HistoricalStats {
            year,
            league,
            team: team_id,
            ..HistoricalStats::default()
        };
        for stat in &self.stat_stream {
            let val = historical.stats.entry(*stat).or_insert(0);
            *val += 1;
        }
        self.historical.push(historical);

        self.reset_stats()
    }

    pub(crate) fn bat_expect_vs(&self, throws: Handedness) -> &ExpectMap {
        if throws == Handedness::Left { &self.bat_expect.0 } else { &self.bat_expect.1 }
    }
    pub(crate) fn pit_expect_vs(&self, bats: Handedness) -> &ExpectMap {
        if bats == Handedness::Left { &self.pit_expect.0 } else { &self.pit_expect.1 }
    }

    pub(crate) fn get_stats(&self) -> Stats {
        let mut stats = Stats {
            ..Stats::default()
        };

        for stat in &self.stat_stream {
            match stat {
                Stat::G => stats.g += 1,
                Stat::Gs => stats.gs += 1,
                Stat::B1b => stats.b_1b += 1,
                Stat::B2b => stats.b_2b += 1,
                Stat::B3b => stats.b_3b += 1,
                Stat::Bhr => stats.b_hr += 1,
                Stat::Bbb => stats.b_bb += 1,
                Stat::Bhbp => stats.b_hbp += 1,
                Stat::Bso => stats.b_so += 1,
                Stat::Bo => stats.b_o += 1,
                Stat::Br => stats.b_r += 1,
                Stat::Brbi => stats.b_rbi += 1,
                Stat::P1b => stats.p_1b += 1,
                Stat::P2b => stats.p_2b += 1,
                Stat::P3b => stats.p_3b += 1,
                Stat::Phr => stats.p_hr += 1,
                Stat::Pbb => stats.p_bb += 1,
                Stat::Phbp => stats.p_hbp += 1,
                Stat::Pso => stats.p_so += 1,
                Stat::Po => stats.p_o += 1,
                Stat::Pr => stats.p_r += 1,
                Stat::Per => stats.p_er += 1,
                Stat::Fpo => stats.f_po += 1,
                Stat::Fe => stats.f_e += 1,
                _ => {}
            }
        }

        stats.calculate();

        stats
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

pub(crate) fn generate_players(players: &mut PlayerMap, count: usize, year: u32, data: &Data, rng: &mut ThreadRng) {
    let pos_gen = vec![
        Position::StartingPitcher,
        Position::StartingPitcher,
        Position::StartingPitcher,
        Position::LongRelief,
        Position::LongRelief,
        Position::ShortRelief,
        Position::ShortRelief,
        Position::Setup,
        Position::Closer,
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

pub(crate) fn collect_all_active(players: &PlayerMap) -> PlayerRefMap<'_> {
    players.iter()
        .filter(|(_, v)| v.active)
        .map(|(k, v)| (*k, v))
        .collect()
}
