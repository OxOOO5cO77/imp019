use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::player::{Expect, ExpectMap, PlayerId, Position, PlayerMap, Handedness, Player};
use crate::team::{TeamId, TeamMap};
use crate::stat::Stat;
use rand::rngs::ThreadRng;
use rand::Rng;
use rand::seq::{SliceRandom, IteratorRandom};
use crate::util::gen_gamma;

lazy_static! {
    static ref LEAGUE_AVG: ExpectMap = {
        let mut expect = HashMap::new();
        expect.insert(Expect::Single, 0.1379988963);
        expect.insert(Expect::Double, 0.045119492);
        expect.insert(Expect::Triple, 0.004006693438);
        expect.insert(Expect::HomeRun, 0.03522694576);
        expect.insert(Expect::Walk, 0.08492014357);
        expect.insert(Expect::HitByPitch, 0.01096355115);
        expect.insert(Expect::Strikeout, 0.19);
        expect.insert(Expect::Out, 0.4909664694);
        expect
    };
}

#[derive(Copy, Clone, Default)]
struct RunnerInfo {
    runner: PlayerId,
    pitcher: PlayerId,
    earned: bool,
}

#[derive(Clone, Default)]
pub(crate) struct DefenseInfo {
    pub(crate) player: PlayerId,
    pub(crate) pos: Position,
}

#[derive(Clone, Default)]
pub(crate) struct PitcherRecord {
    pub(crate) pitcher: PlayerId,
    outs: u8,
    run_diff_in: i8,
    run_diff_out: i8,
}

#[derive(Default)]
pub(crate) struct Scoreboard {
    pub(crate) id: TeamId,
    pub(crate) r: u8,
    pub(crate) h: u8,
    pub(crate) e: u8,
    onbase: [Option<RunnerInfo>; 4],
    runs_in: Vec<RunnerInfo>,
    pub(crate) bo: [DefenseInfo; 9],
    ab: usize,
    pitcher: PlayerId,
    pitches: u32,
    pitcher_outs: u8,
    pitcher_run_diff_in: i8,
    pub(crate) pitcher_record: Vec<PitcherRecord>,
}

impl Scoreboard {
    fn new(id: TeamId) -> Self {
        Self {
            id,
            ..Self::default()
        }
    }

    fn advance_onbase(&mut self, start: usize) {
        if start > 3 || self.onbase[start].is_none() {
            return;
        }
        match start {
            3 => self.runs_in.push(self.onbase[3].unwrap()),
            _ => {
                self.advance_onbase(start + 1);
                self.onbase[start + 1] = self.onbase[start];
            }
        }

        self.onbase[start] = None;
    }


    fn advance_batter(&mut self, batter: PlayerId, pitcher: PlayerId, earned: bool, amt: usize) {
        self.onbase[0] = Some(RunnerInfo { runner: batter, pitcher, earned });
        for idx in 0..amt {
            self.advance_onbase(idx);
        }
    }

    fn player_at_pos(&self, pos: Position) -> PlayerId {
        if pos.is_pitcher() { self.pitcher } else { self.bo.iter().find(|o| o.pos == pos).unwrap().player }
    }

    fn record_runs(&mut self) {
        self.r += self.runs_in.len() as u8;
        self.runs_in.clear();
    }

    fn record_pitcher(&mut self, other_r: i8) {
        self.pitcher_record.push(PitcherRecord {
            pitcher: self.pitcher,
            outs: self.pitcher_outs,
            run_diff_in: self.pitcher_run_diff_in,
            run_diff_out: self.r as i8 - other_r,
        });
    }
}

#[derive(PartialEq)]
enum InningHalf {
    Top,
    Middle,
    Bottom,
    End,
}

impl Default for InningHalf {
    fn default() -> Self { Self::Top }
}

#[derive(Default)]
struct Inning {
    number: u8,
    half: InningHalf,
}

pub(crate) struct GameLogEvent {
    pub(crate) player: PlayerId,
    pub(crate) event: Stat,
    pub(crate) target: Option<Position>,
}

pub(crate) type GameLog = Vec<GameLogEvent>;

#[derive(Default)]
pub(crate) struct Game {
    pub(crate) home: Scoreboard,
    pub(crate) away: Scoreboard,
    pub(crate) playbyplay: GameLog,
}


impl Game {
    pub(crate) fn new(home: TeamId, away: TeamId) -> Self {
        Self {
            home: Scoreboard::new(home),
            away: Scoreboard::new(away),
            playbyplay: Vec::new(),
        }
    }

    fn is_complete(&self, inning: &Inning) -> bool {
        inning.number >= 9 && ((inning.half != InningHalf::Top && self.home.r > self.away.r) || (inning.half == InningHalf::End && self.away.r > self.home.r))
    }

    fn is_away_ab(&self, inning: &Inning) -> bool {
        inning.half == InningHalf::Top || inning.half == InningHalf::Middle
    }

    fn matchup_morey_z(batter: f64, pitcher: f64, league: f64) -> f64 {
        let sqrt_league = (league * (1.0 - league)).sqrt();
        let top_left = (batter - league) / sqrt_league;
        let top_right = (pitcher - league) / sqrt_league;
        let left = (top_left + top_right) / 2.0f64.sqrt();
        (left * sqrt_league) + league
    }

    fn setup_pitcher(players: &mut PlayerMap, teams: &mut TeamMap, scoreboard: &mut Scoreboard, boxscore: &mut GameLog) -> Handedness {
        let team = teams.get_mut(&scoreboard.id).unwrap();
        scoreboard.pitcher = team.rotation[0];
        Self::record_stat(boxscore, team.rotation[0], Stat::Gs, None);
        team.rotation.rotate_left(1);

        let pitcher = players.get_mut(&scoreboard.pitcher).unwrap();
        pitcher.throws
    }

    fn setup_bo(players: &mut PlayerMap, teams: &mut TeamMap, scoreboard: &mut Scoreboard, boxscore: &mut GameLog, year: u32, rng: &mut ThreadRng) {
        let team = teams.get_mut(&scoreboard.id).unwrap();
        let mut team_players = team.players.iter().map(|o| (*o, players.get(o).unwrap())).filter(|o| !o.1.pos.is_pitcher()).collect::<Vec<_>>();
        team_players.sort_by_cached_key(|o| o.1.get_stats().b_obp);
        team_players.reverse();

        let mut index = 0;
        for (id, player) in &team_players {
            if scoreboard.bo.iter().find(|o| o.pos == player.pos).is_none() {
                scoreboard.bo[index] = DefenseInfo {
                    player: *id,
                    pos: player.pos,
                };
                index += 1;
            }
        }

        for starter in scoreboard.bo.iter_mut() {
            if let Some(replacement) = team_players.iter().find(|o| o.0 != starter.player && o.1.pos == starter.pos) {
                let starter_player = players.get(&starter.player).unwrap();
                let fat_pct = starter_player.fatigue as f64 / starter_player.fatigue_threshold(year);
                if rng.gen_bool(fat_pct.min(1.0)) {
                    starter.player = replacement.0;
                }
            }
        }

        for starter in scoreboard.bo.iter() {
            Self::record_stat(boxscore, starter.player, Stat::Gs, None);

            let player = players.get_mut(&starter.player).unwrap();
            player.fatigue += 1;
        }
    }

    fn setup_game(&mut self, players: &mut PlayerMap, teams: &mut TeamMap, boxscore: &mut GameLog, year: u32, rng: &mut ThreadRng) {
        let _home_hand = Self::setup_pitcher(players, teams, &mut self.home, boxscore);
        let _away_hand = Self::setup_pitcher(players, teams, &mut self.away, boxscore);

        Self::setup_bo(players, teams, &mut self.home, boxscore, year, rng);
        Self::setup_bo(players, teams, &mut self.away, boxscore, year, rng);
    }

    fn get_expected_pa(batter: &HashMap<Expect, f64>, pitcher: &HashMap<Expect, f64>, rng: &mut ThreadRng) -> Expect {
        *batter.iter().map(|kv| {
            let bval = kv.1;
            let pval = pitcher.get(&kv.0).unwrap_or(&0.0);
            let lval = LEAGUE_AVG.get(&kv.0).unwrap_or(&0.0);
            let res = (Self::matchup_morey_z(*bval, *pval, *lval) * 1000.0) as u32;
            (kv.0, res)
        }).collect::<Vec<_>>().choose_weighted(rng, |o| o.1).unwrap().0
    }

    fn record_stat(boxscore: &mut GameLog, player: PlayerId, event: Stat, target: Option<Position>) {
        boxscore.push(GameLogEvent {
            player,
            event,
            target,
        });
    }

    fn batting_pitching(&mut self, inning: &Inning) -> (&mut Scoreboard, &Scoreboard) {
        if self.is_away_ab(inning) { (&mut self.away, &self.home) } else { (&mut self.home, &self.away) }
    }

    fn batting(&mut self, inning: &Inning) -> &mut Scoreboard {
        if self.is_away_ab(inning) { &mut self.away } else { &mut self.home }
    }

    fn pitching(&mut self, inning: &Inning) -> &mut Scoreboard {
        if self.is_away_ab(inning) { &mut self.home } else { &mut self.away }
    }

    fn check_for_error(players: &PlayerMap, fielder_id: PlayerId, result: Expect, rng: &mut ThreadRng) -> Expect {
        let fielder = players.get(&fielder_id).unwrap();
        if result == Expect::Out && fielder.check_for_e(rng) {
            Expect::Error
        } else {
            result
        }
    }

    fn max_pitches_for_pos(pos: Position) -> u32 {
        match pos {
            Position::StartingPitcher => 110,
            Position::LongRelief => 50,
            Position::ShortRelief => 25,
            Position::Setup => 25,
            Position::Closer => 25,
            _ => 0,
        }
    }

    fn sub_pitcher(&mut self, inning: &Inning, teams: &mut TeamMap, players: &mut PlayerMap, boxscore: &mut GameLog, rng: &mut ThreadRng) {
        let bat_scoreboard = self.batting(inning);
        let bat_r = bat_scoreboard.r as i8;
        //let batter_id = bat_scoreboard.bo[bat_scoreboard.ab].player;
        //let batter_hand = players.get(&batter_id).unwrap().bats;

        let pit_scoreboard = self.pitching(inning);
        let pit_r = pit_scoreboard.r as i8;
        let pit_team = teams.get(&pit_scoreboard.id).unwrap();
        let cur_pitching = players.get(&pit_scoreboard.pitcher).unwrap().pos;
        let pitch_max = Self::max_pitches_for_pos(cur_pitching);

        let run_diff = pit_r - bat_r;

        let mut used_pitchers = pit_scoreboard.pitcher_record.iter().map(|o| o.pitcher).collect::<Vec<_>>();
        used_pitchers.push(pit_scoreboard.pitcher);
        let available = pit_team.players.iter().filter(|o| !used_pitchers.contains(*o)).collect::<Vec<_>>();

        let sub = if run_diff > 0 && run_diff <= 3 {
            if inning.number == 8 && cur_pitching != Position::Setup {
                available.iter().filter(|o| players.get(o).unwrap().pos == Position::Setup).choose(rng)
            } else if inning.number >= 9 && cur_pitching != Position::Closer {
                available.iter().filter(|o| players.get(o).unwrap().pos == Position::Closer).choose(rng)
            } else {
                None
            }
        } else {
            None
        };

        let sub = if sub.is_none() && pit_scoreboard.pitches > pitch_max {
            if inning.number < 7 {
                available.iter().filter(|o| players.get(o).unwrap().pos == Position::LongRelief).choose(rng)
            } else {
                available.iter().filter(|o| players.get(o).unwrap().pos == Position::ShortRelief).choose(rng)
            }
        } else {
            sub
        };

        if let Some(&&new_pitcher) = sub {
            pit_scoreboard.record_pitcher(bat_r);

            pit_scoreboard.pitcher = new_pitcher;
            pit_scoreboard.pitches = 0;
            pit_scoreboard.pitcher_outs = 0;
            pit_scoreboard.pitcher_run_diff_in = run_diff;
            Self::record_stat(boxscore, new_pitcher, Stat::G, None);
        }
    }

    fn record_wls(boxscore: &mut GameLog, sb: &Scoreboard, oppo_r: i8) {
        let last_pitcher = sb.pitcher_record.len() - 1;
        let mut idx = last_pitcher;
        let mut winner = None;
        loop {
            if sb.pitcher_record[idx].run_diff_out > 0 {
                winner = Some(sb.pitcher_record[idx].pitcher);
                if idx > 0 {
                    idx -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        let mut idx = last_pitcher;
        let mut loser = None;
        loop {
            if sb.pitcher_record[idx].run_diff_out < 0 {
                loser = Some(sb.pitcher_record[idx].pitcher);
                if idx > 0 {
                    idx -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if let Some(w) = winner {
            Self::record_stat(boxscore, w, Stat::Pw, None);
            if last_pitcher == 0 {
                Self::record_stat(boxscore, w, Stat::Pcg, None);
                if oppo_r == 0 {
                    Self::record_stat(boxscore, w, Stat::Psho, None);
                }
            }

            let last_pr = &sb.pitcher_record[last_pitcher];
            if (last_pr.run_diff_in <= 3 || last_pr.outs >= 9) && last_pr.pitcher != w {
                Self::record_stat(boxscore, last_pr.pitcher, Stat::Psv, None);
            }

            if last_pitcher > 0 {
                let mut hold_idx = last_pitcher - 1;
                while hold_idx > 0 && sb.pitcher_record[hold_idx].pitcher != w && sb.pitcher_record[hold_idx].run_diff_in <= 3 {
                    Self::record_stat(boxscore, sb.pitcher_record[hold_idx].pitcher, Stat::Phld, None);
                    hold_idx -= 1;
                }
            }
        }
        if let Some(l) = loser {
            Self::record_stat(boxscore, l, Stat::Pl, None);
            if last_pitcher == 0 {
                Self::record_stat(boxscore, l, Stat::Pcg, None);
            }
        }
    }

    fn end_of_game(&mut self, players: &mut PlayerMap, boxscore: GameLog) {
        for event in &boxscore {
            let player = players.get_mut(&event.player).unwrap();
            player.record_stat(event.event);
        }

        self.playbyplay = boxscore;
    }

    pub(crate) fn sim(&mut self, teams: &mut TeamMap, players: &mut PlayerMap, year: u32, rng: &mut ThreadRng) {
        let mut boxscore = GameLog::new();
        let mut inning = Inning {
            number: 1,
            half: InningHalf::Top,
        };
        let mut outs = 0;
        let mut virtual_outs = 0;

        self.setup_game(players, teams, &mut boxscore, year, rng);

        while !self.is_complete(&inning) {
            if inning.half == InningHalf::Middle {
                self.home.onbase.fill(None);
                outs = 0;
                virtual_outs = 0;
                inning.half = InningHalf::Bottom;
                continue;
            }
            if inning.half == InningHalf::End {
                self.away.onbase.fill(None);
                outs = 0;
                virtual_outs = 0;
                inning.number += 1;
                inning.half = InningHalf::Top;
                continue;
            }
            let earned = virtual_outs < 3;

            self.sub_pitcher(&inning, teams, players, &mut boxscore, rng);

            let (bat_scoreboard, pit_scoreboard) = self.batting_pitching(&inning);

            let pitcher_id = pit_scoreboard.pitcher;
            let pitcher = players.get(&pitcher_id).unwrap();

            let batter_id = bat_scoreboard.bo[bat_scoreboard.ab].player;
            let batter = players.get(&batter_id).unwrap();

            let batter_expect = batter.bat_expect_vs(pitcher.throws);
            let pitcher_expect = pitcher.pit_expect_vs(batter.bats);

            let result = Self::get_expected_pa(batter_expect, pitcher_expect, rng);
            let target = Player::determine_spray(&batter.bat_spray, &pitcher.pit_spray, &result, rng);

            let fielder_id = pit_scoreboard.player_at_pos(target);
            let result = Self::check_for_error(players, fielder_id, result, rng);

            let pitch_avg = (batter.patience + pitcher.control) / 2.0;
            let mut pitches = gen_gamma(rng, pitch_avg, 1.0).round().max(1.0) as u32;

            let mut box_target = None;

            let result_outs = match result {
                Expect::Single => {
                    box_target = Some(target);

                    if target == Position::RightField {
                        bat_scoreboard.advance_onbase(3);
                        bat_scoreboard.advance_onbase(2);
                    }

                    bat_scoreboard.h += 1;
                    bat_scoreboard.advance_batter(batter_id, pitcher_id, earned, 1);
                    0
                }
                Expect::Double => {
                    box_target = Some(target);
                    bat_scoreboard.h += 1;
                    bat_scoreboard.advance_batter(batter_id, pitcher_id, earned, 2);
                    0
                }
                Expect::Triple => {
                    box_target = Some(target);
                    bat_scoreboard.h += 1;
                    bat_scoreboard.advance_batter(batter_id, pitcher_id, earned, 3);
                    0
                }
                Expect::HomeRun => {
                    box_target = Some(target);
                    bat_scoreboard.h += 1;
                    bat_scoreboard.advance_batter(batter_id, pitcher_id, earned, 4);
                    0
                }
                Expect::Walk => {
                    pitches = pitches.max(4);
                    bat_scoreboard.advance_batter(batter_id, pitcher_id, earned, 1);
                    0
                }
                Expect::HitByPitch => {
                    bat_scoreboard.advance_batter(batter_id, pitcher_id, earned, 1);
                    0
                },
                Expect::Error => {
                    box_target = Some(target);
                    Self::record_stat(&mut boxscore, fielder_id, Stat::Fe, None);
                    bat_scoreboard.e += 1;
                    bat_scoreboard.advance_batter(batter_id, pitcher_id, false, 1);
                    0
                }
                Expect::Strikeout => {
                    pitches = pitches.max(3);
                    1
                }
                Expect::Out => {
                    box_target = Some(target);

                    let mut add_outs = 1;
                    if outs < 2 {
                        match target {
                            Position::LeftField |
                            Position::CenterField |
                            Position::RightField => {
                                bat_scoreboard.advance_onbase(3);
                            }
                            Position::Catcher |
                            Position::StartingPitcher => {
                                if bat_scoreboard.onbase[3].is_none() {
                                    bat_scoreboard.advance_onbase(1);
                                }
                            }
                            _ => {
                                if bat_scoreboard.onbase[1].is_some() {
                                    bat_scoreboard.onbase[1] = None;
                                    add_outs += 1;
                                }
                            }
                        }
                    }

                    Self::record_stat(&mut boxscore, fielder_id, Stat::Fpo, None);
                    add_outs
                }
            };

            Self::record_stat(&mut boxscore, batter_id, result.to_batting_stat(), box_target);

            if result != Expect::Error {
                for _ in &bat_scoreboard.runs_in {
                    Self::record_stat(&mut boxscore, batter_id, Stat::Brbi, None);
                }
            }

            if let Some(pitching_stat) = result.to_pitching_stat() {
                Self::record_stat(&mut boxscore, pitcher_id, pitching_stat, None);
            }

            for runner in &bat_scoreboard.runs_in {
                Self::record_stat(&mut boxscore, runner.runner, Stat::Br, None);
                if runner.earned {
                    Self::record_stat(&mut boxscore, runner.pitcher, Stat::Per, None);
                } else {
                    Self::record_stat(&mut boxscore, runner.pitcher, Stat::Pr, None);
                }
            }

            bat_scoreboard.record_runs();

            bat_scoreboard.ab = (bat_scoreboard.ab + 1) % 9;

            let mut pit_scoreboard = self.pitching(&inning);
            pit_scoreboard.pitches += pitches;
            pit_scoreboard.pitcher_outs += result_outs;

            outs += result_outs;
            virtual_outs += result_outs;
            if result == Expect::Error {
                virtual_outs += 1;
            }
            if outs >= 3 {
                if inning.half == InningHalf::Top {
                    inning.half = InningHalf::Middle;
                } else if inning.half == InningHalf::Bottom {
                    inning.half = InningHalf::End;
                }
            }
        }

        let bat_r = self.batting(&inning).r as i8;

        let pitching = self.pitching(&inning);
        pitching.record_pitcher(bat_r);
        Self::record_wls(&mut boxscore, pitching, bat_r);

        let pit_r = pitching.r as i8;

        let batting = self.batting(&inning);
        batting.record_pitcher(pit_r);
        Self::record_wls(&mut boxscore, batting, pit_r);

        teams.get_mut(&self.home.id).unwrap().results(self.home.r, self.away.r);
        teams.get_mut(&self.away.id).unwrap().results(self.away.r, self.home.r);

        self.end_of_game(players, boxscore);
    }
}

#[cfg(test)]
mod tests {
    use crate::game::{RunnerInfo, Scoreboard};

    #[test]
    fn test_advance_onbase() {
        let mut test = Scoreboard::new(0);
        assert!(test.onbase[0].is_none());
        assert!(test.onbase[1].is_none());
        assert!(test.onbase[2].is_none());
        assert!(test.onbase[3].is_none());

        test.advance_onbase(0);
        assert!(test.onbase[0].is_none());
        assert!(test.onbase[1].is_none());
        assert!(test.onbase[2].is_none());
        assert!(test.onbase[3].is_none());

        test.onbase[0] = Some(RunnerInfo { runner: 23, pitcher: 0, earned: true });
        test.advance_onbase(0);
        assert!(test.onbase[0].is_none());
        assert!(test.onbase[1].is_some());
        assert!(test.onbase[2].is_none());
        assert!(test.onbase[3].is_none());

        test.advance_onbase(0);
        assert!(test.onbase[0].is_none());
        assert!(test.onbase[1].is_some());
        assert!(test.onbase[2].is_none());
        assert!(test.onbase[3].is_none());

        test.advance_onbase(1);
        assert!(test.onbase[0].is_none());
        assert!(test.onbase[1].is_none());
        assert!(test.onbase[2].is_some());
        assert!(test.onbase[3].is_none());

        test.advance_onbase(2);
        test.advance_onbase(3);
        assert!(test.onbase[0].is_none());
        assert!(test.onbase[1].is_none());
        assert!(test.onbase[2].is_none());
        assert!(test.onbase[3].is_none());
        assert_eq!(test.runs_in.len(), 1);

        test.onbase[0] = Some(RunnerInfo { runner: 23, pitcher: 0, earned: true });
        test.onbase[1] = Some(RunnerInfo { runner: 23, pitcher: 0, earned: true });
        test.onbase[2] = Some(RunnerInfo { runner: 23, pitcher: 0, earned: true });
        test.onbase[3] = Some(RunnerInfo { runner: 23, pitcher: 0, earned: true });
        test.advance_onbase(0);
        assert!(test.onbase[0].is_none());
        assert!(test.onbase[1].is_some());
        assert!(test.onbase[2].is_some());
        assert!(test.onbase[3].is_some());
        assert_eq!(test.runs_in.len(), 2);

        test.advance_onbase(1);
        assert!(test.onbase[0].is_none());
        assert!(test.onbase[1].is_none());
        assert!(test.onbase[2].is_some());
        assert!(test.onbase[3].is_some());
        assert_eq!(test.runs_in.len(), 3);

        test.onbase[0] = Some(RunnerInfo { runner: 23, pitcher: 0, earned: true });
        test.advance_onbase(0);
        assert!(test.onbase[0].is_none());
        assert!(test.onbase[1].is_some());
        assert!(test.onbase[2].is_some());
        assert!(test.onbase[3].is_some());
        assert_eq!(test.runs_in.len(), 3);
    }

    #[test]
    fn test_advance_onbase_n() {
        let mut test = Scoreboard::new(0);

        test.advance_batter(1, 0, true, 1);
        assert!(test.onbase[0].is_none());
        assert!(test.onbase[1].is_some());
        assert!(test.onbase[2].is_none());
        assert!(test.onbase[3].is_none());

        test.advance_batter(2, 0, true, 2);
        assert!(test.onbase[0].is_none());
        assert!(test.onbase[1].is_none());
        assert!(test.onbase[2].is_some());
        assert!(test.onbase[3].is_some());

        test.advance_batter(3, 0, true, 1);
        assert!(test.onbase[0].is_none());
        assert!(test.onbase[1].is_some());
        assert!(test.onbase[2].is_some());
        assert!(test.onbase[3].is_some());

        test.advance_batter(4, 0, true, 4);
        assert!(test.onbase[0].is_none());
        assert!(test.onbase[1].is_none());
        assert!(test.onbase[2].is_none());
        assert!(test.onbase[3].is_none());
        assert_eq!(test.runs_in.len(), 4);

        test.runs_in.clear();
        test.advance_batter(3, 0, true, 3);
        test.advance_batter(2, 0, true, 2);
        test.advance_batter(1, 0, true, 3);
        assert!(test.onbase[0].is_none());
        assert!(test.onbase[1].is_none());
        assert!(test.onbase[2].is_none());
        assert!(test.onbase[3].is_some());
        assert_eq!(test.runs_in.len(), 2);

        test.runs_in.clear();
        test.advance_batter(1, 0, true, 4);
        assert!(test.onbase[0].is_none());
        assert!(test.onbase[1].is_none());
        assert!(test.onbase[2].is_none());
        assert!(test.onbase[3].is_none());
        assert_eq!(test.runs_in.len(), 2);

        test.runs_in.clear();
        test.advance_batter(1, 0, true, 4);
        assert!(test.onbase[0].is_none());
        assert!(test.onbase[1].is_none());
        assert!(test.onbase[2].is_none());
        assert!(test.onbase[3].is_none());
        assert_eq!(test.runs_in.len(), 1);
    }
}
