use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fmt;

use enum_iterator::IntoEnumIterator;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::data::{Data, AgeData};
use crate::stat::{HistoricalStats, Stat, Stats};
use crate::team::TeamId;
use crate::util::{gen_gamma, gen_normal};

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

    pub(crate) fn is_outfield(&self) -> bool {
        matches!(self,
            Position::LeftField |
            Position::CenterField |
            Position::RightField
        )
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let str = match self {
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
        };
        write!(f, "{}", str)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum Handedness {
    Left,
    Right,
    Switch,
}

impl Display for Handedness {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let str = match self {
            Handedness::Left => "L",
            Handedness::Right => "R",
            Handedness::Switch => "S",
        };
        write!(f, "{}", str)
    }
}

pub(crate) type ExpectMap = HashMap<Expect, f64>;
type SprayChart = HashMap<Expect, HashMap<Position, u32>>;

pub(crate) struct Player {
    pub(crate) active: bool,
    name_first: &'static str,
    name_last: &'static str,
    pub(crate) birthplace: String,
    pub(crate) born: u32,
    pub(crate) pos: Position,
    pub(crate) bats: Handedness,
    pub(crate) throws: Handedness,
    pub(crate) bat_expect: (ExpectMap, ExpectMap),
    pub(crate) bat_spray: SprayChart,
    pub(crate) pit_expect: (ExpectMap, ExpectMap),
    pub(crate) pit_spray: SprayChart,
    pub(crate) error_rate: f64,
    pub(crate) patience: f64,
    pub(crate) control: f64,
    stat_stream: Vec<Stat>,
    pub(crate) historical: Vec<HistoricalStats>,
    pub(crate) fatigue: u16,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, IntoEnumIterator)]
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

struct ExpectRaw {
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

    fn generate_bat_expect(rng: &mut ThreadRng) -> ExpectMap {
        let target_obp = gen_normal(rng, 0.320, 0.036);

        let h1b = gen_gamma(rng, 4.4746090247171, 22.0123537722845);
        let h2b = gen_gamma(rng, 3.28935903780274, 10.0760991667206);
        let h3b = gen_gamma(rng, 0.596598224150856, 0.155987658023824) * h2b;
        let hr = gen_gamma(rng, 12.2812930750413, 2.09953872829662);
        let bb = gen_gamma(rng, 8.34381266257955, 7.16855765752819);
        let hbp = gen_gamma(rng, 18.8629868507638, 0.404463971747468);
        let so = gen_normal(rng, 0.1914556061, 0.02597102753);

        let expect = ExpectRaw {
            target_obp,
            h1b,
            h2b,
            h3b,
            hr,
            bb,
            hbp,
            so,
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
        rng.gen_bool(self.error_rate)
    }

    pub(crate) fn check_for_sb(&self, rng: &mut ThreadRng) -> bool {
        let triple = (*self.bat_expect.0.get(&Expect::Triple).unwrap() * 10.0) - 0.25;
        let sb_pct = (0.7 + (triple * 0.20) + (triple * 0.20) + (triple * 0.20)).clamp(0.0, 1.0);
        rng.gen_bool(sb_pct)
    }

    pub(crate) fn new(data: &Data, pos: &Position, year: u32, rng: &mut ThreadRng) -> Self {
        let loc_data = data.choose_location(rng);
        let name_first = data.choose_name_first(loc_data.country, rng);
        let name_last = data.choose_name_last(loc_data.country, rng);

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

        let error_rate = 1.0 - gen_normal(rng, 0.9765828221, 0.03).clamp(0.0, 1.0);
        let patience = gen_gamma(rng, 4.5, 1.0).round().max(1.0);
        let control = gen_gamma(rng, 18.0, 0.2195).round().max(1.0);

        Self {
            active: true,
            name_first,
            name_last,
            birthplace: format!("{}, {}, {}", loc_data.city, loc_data.state, loc_data.country),
            born: year - age,
            pos: *pos,
            bats: *bat_hand,
            throws: *pitch_hand,
            bat_expect,
            bat_spray,
            pit_expect,
            pit_spray,
            error_rate,
            patience,
            control,
            stat_stream: vec![],
            historical: vec![],
            fatigue: 0,
        }
    }

    pub(crate) fn fullname(&self) -> String {
        format!("{} {}", self.name_first, self.name_last)
    }

    pub(crate) fn fname(&self) -> String {
        format!("{}. {}", self.name_first.chars().next().unwrap(), self.name_last)
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

        historical.stats = Stats::compile_stats(&self.stat_stream);

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
        Stats::compile_stats(&self.stat_stream)
    }

    pub(crate) fn age(&self, year: u32) -> u32 {
        year - self.born
    }

    pub(crate) fn fatigue_threshold(&self, year: u32) -> f64 {
        let mut age_factor = (50u64 - self.age(year).min(49) as u64) * 2;
        age_factor = age_factor * age_factor;
        age_factor as f64
    }

    fn apply_age_to_value(cur: f64, other: f64, age_data: &AgeData, rng: &mut ThreadRng) -> f64 {
        match age_data.skew.iter().zip(0..2).collect::<Vec<(_,_)>>().choose_weighted(rng, |o| o.1).unwrap().1 {
            0 => f64::min(cur,other),
            1 => cur,
            2 => f64::max(cur,other),
            _ => cur
        }
    }

    fn apply_age_to_expect(expect_self: &mut ExpectMap, expect_other: &ExpectMap, age_data: &AgeData, rng: &mut ThreadRng) {
        for expect in Expect::into_enum_iter() {
            expect_self.insert(expect, Self::apply_age_to_value(expect_self[&expect], expect_other[&expect], age_data, rng ));
        }
    }

    pub(crate) fn apply_age(&mut self, year: u32, data: &Data, rng: &mut ThreadRng ) {
        let age_data = data.age.iter().find(|o| o.age == self.age(year) ).expect(&*format!("age was {}", self.age(year)));
        let target = Player::new(data, &self.pos, year, rng);

        Self::apply_age_to_expect( &mut self.bat_expect.0, &target.bat_expect.0, age_data, rng );
        Self::apply_age_to_expect( &mut self.bat_expect.1, &target.bat_expect.1, age_data, rng );
        Self::apply_age_to_expect( &mut self.pit_expect.0, &target.pit_expect.0, age_data, rng );
        Self::apply_age_to_expect( &mut self.pit_expect.1, &target.pit_expect.1, age_data, rng );

    }

    pub(crate) fn should_retire(&self, year: u32, rng: &mut ThreadRng) -> bool {
        const MIN_AGE: u32 = 30;
        const MAX_AGE: u32 = 45;
        let age = self.age(year);
        let age_factor = age.clamp(MIN_AGE, MAX_AGE) - MIN_AGE;
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
        players.insert(player_id, Player::new(data, pos_gen.choose(rng).unwrap(), year, rng));
        player_id += 1;
    }
}

pub(crate) fn collect_all_active(players: &PlayerMap) -> PlayerRefMap<'_> {
    players.iter()
        .filter(|(_, v)| v.active)
        .map(|(k, v)| (*k, v))
        .collect()
}
