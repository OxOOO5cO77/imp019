use std::collections::HashMap;

use rand::rngs::ThreadRng;

use crate::data::Data;
use crate::player::{collect_all_active, generate_players, PlayerId, PlayerMap};
use crate::schedule::Schedule;
use crate::stat::{Stat, Stats};
use crate::team::{TeamId, TeamMap};

#[derive(Default)]
pub(crate) struct LeagueRecord {
    pub(crate) player_id: PlayerId,
    pub(crate) team_id: TeamId,
    pub(crate) record: u32,
    pub(crate) year: u32,
}

#[derive(Default)]
pub(crate) struct League {
    id: u32,
    pub(crate) teams: Vec<TeamId>,
    pub(crate) schedule: Schedule,
    pub(crate) cur_idx: usize,
    pub(crate) records: HashMap<Stat, Option<LeagueRecord>>,

}

impl League {
    pub(crate) fn new(id: u32, team_count: usize, remaining_teams: &mut Vec<TeamId>, rng: &mut ThreadRng) -> League {
        let mut teams = Vec::new();
        for _ in 0..team_count {
            if let Some(team) = remaining_teams.pop() {
                teams.push(team);
            }
        }

        let schedule = Schedule::new(&teams, rng);

        Self {
            id,
            teams,
            schedule,
            ..Self::default()
        }
    }

    pub(crate) fn reset_schedule(&mut self, teams: &mut TeamMap, rng: &mut ThreadRng) {
        for team_id in &self.teams {
            let team = teams.get_mut(team_id).unwrap();
            team.results.reset();
        }
        self.schedule = Schedule::new(&self.teams, rng);
        self.cur_idx = 0;
    }

    pub(crate) fn sim(&mut self, team_data: &mut TeamMap, players: &mut PlayerMap, year: u32, mut rng: &mut ThreadRng) -> bool {
        if self.cur_idx < self.schedule.games.len() {
            let teams = self.teams.len();
            for idx in self.cur_idx..(self.cur_idx + (teams / 2)) {
                if let Some(game) = self.schedule.games.get_mut(idx) {
                    game.sim(team_data, players, year, &mut rng);
                }
            }
            self.cur_idx += teams / 2;
            return true;
        }

        self.teams.sort_by_key(|o| team_data.get(o).unwrap().get_losses());

        false
    }
}

pub(crate) const RECORD_STATS: [Stat; 17] = [
    Stat::Bhr,
    Stat::Br,
    Stat::Brbi,
    Stat::Bso,
    Stat::Bh,
    Stat::B2b,
    Stat::B3b,
    Stat::Bbb,
    Stat::Bsb,
    Stat::Bavg,
    Stat::Bobp,
    Stat::Bslg,
    Stat::Pw,
    Stat::Psv,
    Stat::Pso,
    Stat::Pwhip,
    Stat::Pera,
];

fn check_record(records: &mut HashMap<Stat, Option<LeagueRecord>>, player_stats: &Stats, player_id: PlayerId, team_id: TeamId, year: u32, games: u32) {
    for stat in &RECORD_STATS {
        let record = records.entry(*stat).or_insert(None);
        let pval = player_stats.get_stat(*stat);

        if let Some(rec) = record {
            let reverse = stat.is_reverse_sort();

            if (reverse && rec.record <= pval) || (!reverse && rec.record >= pval) {
                continue;
            }

            if !stat.is_qualified(player_stats, games) {
                continue;
            }
        }
        *record = Some(LeagueRecord {
            record: pval,
            player_id,
            team_id,
            year,
        });
    }
}

pub(crate) fn end_of_season(leagues: &mut Vec<League>, teams: &mut TeamMap, players: &mut PlayerMap, count: usize, year: u32, data: &Data, rng: &mut ThreadRng) {
    // record history
    for (league_idx, league) in leagues.iter_mut().enumerate() {
        for (rank, team_id) in league.teams.iter().enumerate() {
            let team = teams.get_mut(&team_id).unwrap();
            for player_id in &team.players {
                let player = players.get_mut(&player_id).unwrap();
                check_record(&mut league.records, &player.get_stats(), *player_id, *team_id, year, team.results.games());
                player.record_stat_history(year, league.id, *team_id);
            }
            team.record_results(year, league_idx, rank, team.results);
        }
    }

    // relegate/promite
    for league_idx in 0..(leagues.len() - 1) {
        let upper = league_idx;
        let lower = league_idx + 1;

        let len = leagues[upper].teams.len();
        let relegated = leagues[upper].teams.split_off(len - count);

        let mut promoted = Vec::new();
        for _ in 0..count {
            promoted.push(leagues[lower].teams.remove(0));
        }

        leagues[upper].teams.append(&mut promoted);
        for rel in relegated {
            leagues[lower].teams.insert(0, rel);
        }
    }

    // reset league
    for league in leagues.iter_mut() {
        league.reset_schedule(teams, rng);
    }

    //update all players
    for player in players.values_mut() {
        player.fatigue = 0;
    }

    // retire players
    let mut retired = 0;
    for player in players.values_mut().filter(|o| o.active && o.should_retire(year, rng)) {
        player.active = false;
        //println!("[Retired] {} Age: {}", player.fullname(), player.age(year));
        retired += 1;
    }

    generate_players(players, retired, year, &data, rng);

    // collect available players
    let mut available = collect_all_active(players);
    for team in teams.values_mut() {
        team.players.retain(|o| players.get(o).unwrap().active);
        available.retain(|k, _| !team.players.contains(k));
    }

    // repopulate teams
    for team in teams.values_mut() {
        team.populate(&mut available, players);
    }
}
