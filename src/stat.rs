use std::fmt::{Display, Formatter};
use std::fmt;

use crate::team::TeamId;

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
    Bibb,
    Bhbp,
    Bso,
    Bo,
    Br,
    Brbi,
    Bgidp,
    Bsb,
    Bcs,
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
    Pibb,
    Phbp,
    Po,
    Pso,
    Pr,
    Per,
    Pw,
    Pl,
    Psv,
    Pbs,
    Phld,
    Pcg,
    Psho,
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

impl Stat {
    pub(crate) fn is_batting(&self) -> bool {
        matches!(self, Stat::B1b | Stat::B2b | Stat::B3b | Stat::Bhr | Stat::Bbb | Stat::Bibb | Stat::Bhbp | Stat::Bso | Stat::Bo | Stat::Bgidp | Stat::Bsb | Stat::Bcs | Stat::Br | Stat::Brbi | Stat::Bh | Stat::Bab | Stat::Bpa | Stat::Bavg | Stat::Bobp | Stat::Bslg)
    }

    pub(crate) fn value(&self, val: u32) -> String {
        match self {
            Stat::Bavg |
            Stat::Bobp |
            Stat::Bslg |
            Stat::Pavg |
            Stat::Pobp |
            Stat::Pslg |
            Stat::Pera |
            Stat::Pwhip => format!("{}.{:03}", val / 1000, val % 1000),
            Stat::Po => format!("{}.{}", val / 3, val % 3),
            _ => format!("{}", val),
        }
    }

    pub(crate) fn is_reverse_sort(&self) -> bool {
        matches!(self, Stat::Pavg | Stat::Pobp | Stat::Pslg | Stat::Pera | Stat::Pwhip)
    }

    pub(crate) fn is_qualified(&self, player_stats: &Stats, games: u32) -> bool {
        let qual = match self {
            Stat::Bavg |
            Stat::Bobp |
            Stat::Bslg => Some((Stat::Bpa, 31)),
            Stat::Pobp |
            Stat::Pslg |
            Stat::Pera |
            Stat::Pwhip => Some((Stat::Po, 30)),
            _ => None,
        };
        if let Some((qstat, factor)) = qual {
            let qval = player_stats.get_stat(qstat);
            let qual = games * factor / 10;
            if qval < qual {
                return false;
            }
        }
        true
    }
}

impl Display for Stat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let str = match self {
            Stat::G => "G",
            Stat::Gs => "GS",
            Stat::B1b => "1B",
            Stat::B2b => "2B",
            Stat::B3b => "3B",
            Stat::Bhr => "HR",
            Stat::Bbb => "BB",
            Stat::Bibb => "IBB",
            Stat::Bhbp => "HBP",
            Stat::Bso => "SO",
            Stat::Bo => "O",
            Stat::Bgidp => "GIDP",
            Stat::Bsb => "SB",
            Stat::Bcs => "CS",
            Stat::Br => "R",
            Stat::Brbi => "RBI",
            Stat::Bh => "H",
            Stat::Bab => "AB",
            Stat::Bpa => "PA",
            Stat::Bavg => "AVG",
            Stat::Bobp => "OBP",
            Stat::Bslg => "SLG",
            Stat::P1b => "1B",
            Stat::P2b => "2B",
            Stat::P3b => "3B",
            Stat::Phr => "HR",
            Stat::Pbb => "BB",
            Stat::Pibb => "IBB",
            Stat::Phbp => "HBP",
            Stat::Po => "IP",
            Stat::Pso => "SO",
            Stat::Pr => "R",
            Stat::Per => "ER",
            Stat::Pw => "W",
            Stat::Pl => "L",
            Stat::Psv => "SV",
            Stat::Pbs => "BS",
            Stat::Phld => "HLD",
            Stat::Pcg => "CG",
            Stat::Psho => "SHO",
            Stat::Ph => "H",
            Stat::Pbf => "BF",
            Stat::Pavg => "BAA",
            Stat::Pobp => "OBP",
            Stat::Pslg => "SLG",
            Stat::Pera => "ERA",
            Stat::Pwhip => "WHIP",
            Stat::Fpo => "PO",
            Stat::Fe => "E",
        };
        write!(f, "{}", str)
    }
}

#[derive(Default)]
pub(crate) struct Stats {
    pub(crate) g: u32,
    pub(crate) gs: u32,
    pub(crate) b_1b: u32,
    pub(crate) b_2b: u32,
    pub(crate) b_3b: u32,
    pub(crate) b_hr: u32,
    pub(crate) b_bb: u32,
    pub(crate) b_ibb: u32,
    pub(crate) b_hbp: u32,
    pub(crate) b_r: u32,
    pub(crate) b_rbi: u32,
    pub(crate) b_so: u32,
    pub(crate) b_o: u32,
    pub(crate) b_gidp: u32,
    pub(crate) b_sb: u32,
    pub(crate) b_cs: u32,
    pub(crate) b_h: u32,
    pub(crate) b_ab: u32,
    pub(crate) b_pa: u32,
    pub(crate) b_avg: u32,
    pub(crate) b_obp: u32,
    pub(crate) b_slg: u32,

    pub(crate) p_1b: u32,
    pub(crate) p_2b: u32,
    pub(crate) p_3b: u32,
    pub(crate) p_hr: u32,
    pub(crate) p_bb: u32,
    pub(crate) p_ibb: u32,
    pub(crate) p_hbp: u32,
    pub(crate) p_r: u32,
    pub(crate) p_er: u32,
    pub(crate) p_w: u32,
    pub(crate) p_l: u32,
    pub(crate) p_sv: u32,
    pub(crate) p_bs: u32,
    pub(crate) p_hld: u32,
    pub(crate) p_cg: u32,
    pub(crate) p_sho: u32,
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
            Stat::Bibb => self.b_ibb,
            Stat::Bhbp => self.b_hbp,
            Stat::Bso => self.b_so,
            Stat::Bo => self.b_o,
            Stat::Bgidp => self.b_gidp,
            Stat::Bsb => self.b_sb,
            Stat::Bcs => self.b_cs,
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
            Stat::Pibb => self.p_ibb,
            Stat::Phbp => self.p_hbp,
            Stat::Pso => self.p_so,
            Stat::Po => self.p_o,
            Stat::Pr => self.p_r,
            Stat::Per => self.p_er,
            Stat::Pw => self.p_w,
            Stat::Pl => self.p_l,
            Stat::Psv => self.p_sv,
            Stat::Pbs => self.p_bs,
            Stat::Phld => self.p_hld,
            Stat::Pcg => self.p_cg,
            Stat::Psho => self.p_sho,
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
    pub(crate) fn compile(&mut self, rhs: &Self) {
        self.g += rhs.g;
        self.gs += rhs.gs;
        self.b_1b += rhs.b_1b;
        self.b_2b += rhs.b_2b;
        self.b_3b += rhs.b_3b;
        self.b_hr += rhs.b_hr;
        self.b_bb += rhs.b_bb;
        self.b_ibb += rhs.b_ibb;
        self.b_hbp += rhs.b_hbp;
        self.b_r += rhs.b_r;
        self.b_rbi += rhs.b_rbi;
        self.b_so += rhs.b_so;
        self.b_o += rhs.b_o;
        self.b_gidp += rhs.b_gidp;
        self.b_sb += rhs.b_sb;
        self.b_cs += rhs.b_cs;
        self.b_h += rhs.b_h;
        self.b_ab += rhs.b_ab;
        self.b_pa += rhs.b_pa;
        self.b_avg += rhs.b_avg;
        self.b_obp += rhs.b_obp;
        self.b_slg += rhs.b_slg;
        self.p_1b += rhs.p_1b;
        self.p_2b += rhs.p_2b;
        self.p_3b += rhs.p_3b;
        self.p_hr += rhs.p_hr;
        self.p_bb += rhs.p_bb;
        self.p_ibb += rhs.p_ibb;
        self.p_hbp += rhs.p_hbp;
        self.p_r += rhs.p_r;
        self.p_er += rhs.p_er;
        self.p_w += rhs.p_w;
        self.p_l += rhs.p_l;
        self.p_sv += rhs.p_sv;
        self.p_bs += rhs.p_bs;
        self.p_hld += rhs.p_hld;
        self.p_cg += rhs.p_cg;
        self.p_sho += rhs.p_sho;
        self.p_so += rhs.p_so;
        self.p_o += rhs.p_o;
        self.p_h += rhs.p_h;
        self.p_bf += rhs.p_bf;
        self.p_avg += rhs.p_avg;
        self.p_obp += rhs.p_obp;
        self.p_slg += rhs.p_slg;
        self.p_era += rhs.p_era;
        self.p_whip += rhs.p_whip;
        self.f_po += rhs.f_po;
        self.f_e += rhs.f_e;
        self.calculate();
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

    pub(crate) fn compile_stats(stream: &[Stat]) -> Stats {
        let mut stats = Stats {
            ..Stats::default()
        };
        for stat in stream.iter() {
            match stat {
                Stat::G => stats.g += 1,
                Stat::Gs => {
                    stats.gs += 1;
                    stats.g += 1
                }
                Stat::B1b => stats.b_1b += 1,
                Stat::B2b => stats.b_2b += 1,
                Stat::B3b => stats.b_3b += 1,
                Stat::Bhr => stats.b_hr += 1,
                Stat::Bbb => stats.b_bb += 1,
                Stat::Bibb => {
                    stats.b_ibb += 1;
                    stats.b_bb += 1
                },
                Stat::Bhbp => stats.b_hbp += 1,
                Stat::Bso => {
                    stats.b_so += 1;
                    stats.b_o += 1
                }
                Stat::Bo => stats.b_o += 1,
                Stat::Bgidp => {
                    stats.b_gidp += 1;
                    stats.b_o += 1
                },
                Stat::Bsb => stats.b_sb += 1,
                Stat::Bcs => stats.b_cs += 1,
                Stat::Br => stats.b_r += 1,
                Stat::Brbi => stats.b_rbi += 1,
                Stat::P1b => stats.p_1b += 1,
                Stat::P2b => stats.p_2b += 1,
                Stat::P3b => stats.p_3b += 1,
                Stat::Phr => stats.p_hr += 1,
                Stat::Pbb => stats.p_bb += 1,
                Stat::Pibb => {
                    stats.p_ibb += 1;
                    stats.p_bb += 1
                },
                Stat::Phbp => stats.p_hbp += 1,
                Stat::Pso => {
                    stats.p_so += 1;
                    stats.p_o += 1
                }
                Stat::Po => stats.p_o += 1,
                Stat::Pr => stats.p_r += 1,
                Stat::Per => {
                    stats.p_er += 1;
                    stats.p_r += 1
                }
                Stat::Pw => stats.p_w += 1,
                Stat::Pl => stats.p_l += 1,
                Stat::Psv => stats.p_sv += 1,
                Stat::Pbs => stats.p_bs += 1,
                Stat::Phld => stats.p_hld += 1,
                Stat::Pcg => stats.p_cg += 1,
                Stat::Psho => stats.p_sho += 1,
                Stat::Fpo => stats.f_po += 1,
                Stat::Fe => stats.f_e += 1,
                _ => {}
            }
        }

        stats.calculate();
        stats
    }
}

#[derive(Default)]
pub(crate) struct HistoricalStats {
    pub(crate) year: u32,
    pub(crate) league: u32,
    pub(crate) team: TeamId,
    pub(crate) stats: Stats,
}
