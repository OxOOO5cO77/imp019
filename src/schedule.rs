use std::collections::HashMap;

use lazy_static::lazy_static;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::player::{Expect, ExpectMap, Handedness, Player, PlayerId, PlayerMap, Position, Stat};
use crate::team::{TeamId, TeamMap};

#[derive(Copy, Clone, Default)]
struct RunnerInfo {
    runner: PlayerId,
    pitcher: PlayerId,
    earned: bool,
}

#[derive(Clone, Default)]
struct DefenseInfo {
    player: PlayerId,
    pos: Position,
}

#[derive(Default)]
pub(crate) struct Scoreboard {
    pub(crate) id: TeamId,
    onbase: [Option<RunnerInfo>; 4],
    runs_in: Vec<RunnerInfo>,
    pub(crate) r: u8,
    //    pub(crate) h: u8,
//    pub(crate) e: u8,
    bo: [DefenseInfo; 9],
    ab: usize,
    pitcher_of_record: PlayerId,

}

impl Scoreboard {
    fn new(id: TeamId) -> Self {
        Self {
            id,
            ..Self::default()
        }
    }

    fn advance_onbase(&mut self, batter: PlayerId, pitcher: PlayerId, earned: bool, amt: u8) -> u8 {
        self.onbase[0] = Some(RunnerInfo { runner: batter, pitcher, earned });
        for _ in 0..amt {
            if self.onbase[1].is_some() {
                if self.onbase[2].is_some() {
                    if self.onbase[3].is_some() {
                        self.runs_in.push(self.onbase[3].unwrap());
                    }
                    self.onbase[3] = self.onbase[2];
                }
                self.onbase[2] = self.onbase[1];
            }
            self.onbase[1] = self.onbase[0];
        }
        for idx in 0..amt as usize {
            self.onbase[idx] = None;
        }
        0
    }

    fn player_at_pos(&self, pos: Position) -> PlayerId {
        if pos.is_pitcher() { self.pitcher_of_record } else { self.bo.iter().find(|o| o.pos == pos).unwrap().player }
    }

    pub(crate) fn record_runs(&mut self) {
        self.r += self.runs_in.len() as u8;
        self.runs_in.clear();
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

#[derive(Default)]
pub(crate) struct Game {
    pub(crate) home: Scoreboard,
    pub(crate) away: Scoreboard,
    inning: Inning,
    outs: u8,
    virtual_outs: u8,
}

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


impl Game {
    fn new(home: TeamId, away: TeamId) -> Self {
        Self {
            home: Scoreboard::new(home),
            away: Scoreboard::new(away),
            inning: Inning {
                number: 1,
                half: InningHalf::Top,
            },
            outs: 0,
            virtual_outs: 0,
        }
    }

    fn complete(&self) -> bool {
        self.inning.number >= 9 && ((self.inning.half != InningHalf::Top && self.home.r > self.away.r) || (self.inning.half == InningHalf::End && self.away.r > self.home.r))
    }

    fn is_away_ab(&self) -> bool {
        self.inning.half == InningHalf::Top || self.inning.half == InningHalf::Middle
    }

    fn matchup_morey_z(batter: f64, pitcher: f64, league: f64) -> f64 {
        let sqrt_league = (league * (1.0 - league)).sqrt();
        let top_left = (batter - league) / sqrt_league;
        let top_right = (pitcher - league) / sqrt_league;
        let left = (top_left + top_right) / 2.0f64.sqrt();
        (left * sqrt_league) + league
    }

    fn setup_pitcher(players: &mut PlayerMap, teams: &mut TeamMap, scoreboard: &mut Scoreboard) -> Handedness {
        let team = teams.get_mut(&scoreboard.id).unwrap();
        scoreboard.pitcher_of_record = team.rotation[0];
        let pitcher = Self::record_stat(players, team.rotation[0], Stat::Gs);
        team.rotation.rotate_left(1);
        pitcher.throws
    }

    fn setup_bo(players: &mut PlayerMap, teams: &mut TeamMap, scoreboard: &mut Scoreboard, year: u32, rng: &mut ThreadRng) {
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
            let player = Self::record_stat(players, starter.player, Stat::Gs);
            player.fatigue += 1;
        }
    }

    fn setup_game(&mut self, players: &mut PlayerMap, teams: &mut TeamMap, year: u32, rng: &mut ThreadRng) {
        let _home_hand = Self::setup_pitcher(players, teams, &mut self.home);
        let _away_hand = Self::setup_pitcher(players, teams, &mut self.away);

        Self::setup_bo(players, teams, &mut self.home, year, rng);
        Self::setup_bo(players, teams, &mut self.away, year, rng);

        self.inning.number = 1;
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

    fn record_stat(players: &mut PlayerMap, player_id: PlayerId, stat: Stat) -> &mut Player {
        let player = players.get_mut(&player_id).unwrap();
        player.record_stat(stat);
        match stat {
            Stat::Bso => player.record_stat(Stat::Bo),
            Stat::Pso => player.record_stat(Stat::Po),
            Stat::Gs => player.record_stat(Stat::G),
            _ => {}
        }
        player
    }

    fn home_away(&mut self) -> (&mut Scoreboard, &Scoreboard) {
        if self.is_away_ab() { (&mut self.away, &self.home) } else { (&mut self.home, &self.away) }
    }

    fn check_for_error(players: &mut PlayerMap, fielder_id: PlayerId, result: Expect, rng: &mut ThreadRng) -> Expect {
        let fielder = players.get(&fielder_id).unwrap();
        if result == Expect::Out && fielder.check_for_e(rng) {
            Expect::Error
        } else {
            result
        }
    }

    pub(crate) fn sim(&mut self, teams: &mut TeamMap, players: &mut PlayerMap, year: u32, rng: &mut ThreadRng) {
        self.setup_game(players, teams, year, rng);

        while !self.complete() {
            if self.inning.half == InningHalf::Middle {
                self.home.onbase.fill(None);
                self.outs = 0;
                self.virtual_outs = 0;
                self.inning.half = InningHalf::Bottom;
                continue;
            }
            if self.inning.half == InningHalf::End {
                self.away.onbase.fill(None);
                self.outs = 0;
                self.virtual_outs = 0;
                self.inning.number += 1;
                self.inning.half = InningHalf::Top;
                continue;
            }
            let earned = self.virtual_outs < 3;

            let (bat_scoreboard, pit_scoreboard) = self.home_away();

            let pitcher_id = pit_scoreboard.pitcher_of_record;
            let pitcher = players.get(&pitcher_id).unwrap();

            let batter_id = bat_scoreboard.bo[bat_scoreboard.ab].player;
            let batter = players.get(&batter_id).unwrap();

            let batter_expect = batter.bat_expect_vs(pitcher.throws);
            let pitcher_expect = pitcher.pit_expect_vs(batter.bats);

            let result = Self::get_expected_pa(batter_expect, pitcher_expect, rng);
            let target = Player::determine_spray(&batter.bat_spray, &pitcher.pit_spray, &result, rng);

            let fielder_id = pit_scoreboard.player_at_pos(target);
            let result = Self::check_for_error(players, fielder_id, result, rng);


            let outs = match result {
                Expect::Single => bat_scoreboard.advance_onbase(batter_id, pitcher_id, earned, 1),
                Expect::Double => bat_scoreboard.advance_onbase(batter_id, pitcher_id, earned, 2),
                Expect::Triple => bat_scoreboard.advance_onbase(batter_id, pitcher_id, earned, 3),
                Expect::HomeRun => bat_scoreboard.advance_onbase(batter_id, pitcher_id, earned, 4),
                Expect::Walk => bat_scoreboard.advance_onbase(batter_id, pitcher_id, earned, 1),
                Expect::HitByPitch => bat_scoreboard.advance_onbase(batter_id, pitcher_id, earned, 1),
                Expect::Error => {
                    Self::record_stat(players, fielder_id, Stat::Fe);
                    bat_scoreboard.advance_onbase(batter_id, pitcher_id, false, 1)
                }
                Expect::Strikeout => 1,
                Expect::Out => {
                    Self::record_stat(players, fielder_id, Stat::Fpo);
                    1
                }
            };
            let batting_stat = result.to_batting_stat();
            let pitching_stat = result.to_pitching_stat();

            let batter = Self::record_stat(players, batter_id, batting_stat);

            if result != Expect::Error {
                for _ in &bat_scoreboard.runs_in {
                    batter.record_stat(Stat::Brbi);
                }
            }

            Self::record_stat(players, pitcher_id, pitching_stat);

            for runner in &bat_scoreboard.runs_in {
                Self::record_stat(players, runner.runner, Stat::Br);
                let pitcher = Self::record_stat(players, runner.pitcher, Stat::Pr);
                if runner.earned {
                    pitcher.record_stat(Stat::Per);
                }
            }

            bat_scoreboard.record_runs();

            bat_scoreboard.ab = (bat_scoreboard.ab + 1) % 9;

            self.outs += outs;
            self.virtual_outs += outs;
            if result == Expect::Error {
                self.virtual_outs += 1;
            }
            if self.outs >= 3 {
                if self.inning.half == InningHalf::Top {
                    self.inning.half = InningHalf::Middle;
                } else if self.inning.half == InningHalf::Bottom {
                    self.inning.half = InningHalf::End;
                }
            }
        }

        teams.get_mut(&self.home.id).unwrap().results(self.home.r, self.away.r);
        teams.get_mut(&self.away.id).unwrap().results(self.away.r, self.home.r);
    }
}


pub(crate) struct Schedule {
    pub(crate) games: Vec<Game>,
}

impl Schedule {
    pub(crate) fn new(teams: &[TeamId], rng: &mut ThreadRng) -> Self {
        let mut raw_matchups = Vec::new();
        let team_count = teams.len();
        raw_matchups.reserve(team_count * (team_count - 1));

        for home in teams {
            for away in teams {
                if home != away {
                    raw_matchups.push(Game::new(*home, *away));
                }
            }
        }

        raw_matchups.shuffle(rng);

        let mut matchups = Vec::new();
        while !raw_matchups.is_empty() {
            let mut teams_to_pick = (0..team_count).map(|o| teams[o]).collect::<Vec<_>>();
            teams_to_pick.shuffle(rng);

            while !teams_to_pick.is_empty() {
                if let Some(team) = teams_to_pick.pop() {
                    if let Some(idx) = raw_matchups.iter().position(|x| x.home.id == team && teams_to_pick.contains(&x.away.id)) {
                        let game = raw_matchups.remove(idx);
                        let other_team = if game.home.id == team { game.away.id } else { game.home.id };
                        matchups.push(game);
                        teams_to_pick.retain(|&o| o != other_team);
                    }
                }
            }
        }

        let mut games = Vec::new();
        for idx in (0..matchups.len()).step_by(team_count / 2) {
            for _ in 0..4 {
                for offset in 0..(team_count / 2) {
                    let game = &matchups[idx + offset];
                    games.push(Game::new(game.home.id, game.away.id));
                }
            }
        }

        Self {
            games
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::schedule::Scoreboard;

    #[test]
    fn test_advance_onbase() {
        let mut test1 = Scoreboard::new(0);
        test1.advance_onbase(1, 0, true, 1);
        assert!(test1.onbase[0].is_none());
        assert!(test1.onbase[1].is_some());
        assert!(test1.onbase[2].is_none());
        assert!(test1.onbase[3].is_none());

        test1.advance_onbase(2, 0, true, 2);
        assert!(test1.onbase[0].is_none());
        assert!(test1.onbase[1].is_none());
        assert!(test1.onbase[2].is_some());
        assert!(test1.onbase[3].is_some());

        test1.advance_onbase(3, 0, true, 1);
        assert!(test1.onbase[0].is_none());
        assert!(test1.onbase[1].is_some());
        assert!(test1.onbase[2].is_some());
        assert!(test1.onbase[3].is_some());

        test1.advance_onbase(4, 0, true, 4);
        assert!(test1.onbase[0].is_none());
        assert!(test1.onbase[1].is_none());
        assert!(test1.onbase[2].is_none());
        assert!(test1.onbase[3].is_none());
        assert_eq!(test1.runs_in.len(), 4);

        test1.runs_in.clear();
        test1.advance_onbase(3, 0, true, 3);
        test1.advance_onbase(2, 0, true, 2);
        test1.advance_onbase(1, 0, true, 3);
        assert!(test1.onbase[0].is_none());
        assert!(test1.onbase[1].is_none());
        assert!(test1.onbase[2].is_none());
        assert!(test1.onbase[3].is_some());
        assert_eq!(test1.runs_in.len(), 2);

        test1.runs_in.clear();
        test1.advance_onbase(1, 0, true, 4);
        assert!(test1.onbase[0].is_none());
        assert!(test1.onbase[1].is_none());
        assert!(test1.onbase[2].is_none());
        assert!(test1.onbase[3].is_none());
        assert_eq!(test1.runs_in.len(), 2);

        test1.runs_in.clear();
        test1.advance_onbase(1, 0, true, 4);
        assert!(test1.onbase[0].is_none());
        assert!(test1.onbase[1].is_none());
        assert!(test1.onbase[2].is_none());
        assert!(test1.onbase[3].is_none());
        assert_eq!(test1.runs_in.len(), 1);
    }
}
