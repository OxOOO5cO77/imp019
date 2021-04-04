use std::collections::HashMap;

use rand::rngs::ThreadRng;

use crate::data::Data;
use crate::player::{Player,generate_players};
use crate::schedule::Schedule;
use crate::team::Team;

pub struct League {
    id: u32,
    pub(crate) teams: Vec<u64>,
    pub(crate) schedule: Schedule,
}

impl League {
    pub(crate) fn new(id: u32, team_count: usize, remaining_teams: &mut Vec<u64>, rng: &mut ThreadRng) -> League {
        let mut teams = Vec::new();
        for _ in 0..team_count {
            if let Some(team) = remaining_teams.pop() {
                teams.push(team);
            }
        }

        let schedule = Schedule::new(&teams, rng);

        League {
            id,
            teams,
            schedule,
        }
    }

    pub(crate) fn reset_schedule(&mut self, teams: &mut HashMap<u64, Team>, rng: &mut ThreadRng) {
        for team_id in &self.teams {
            let team = teams.get_mut(team_id).unwrap();
            team.results.reset();
        }
        self.schedule = Schedule::new(&self.teams, rng)
    }

    pub(crate) fn sim(&mut self, team_data: &mut HashMap<u64, Team>, players: &mut HashMap<u64, Player>, year: u32, mut rng: &mut ThreadRng) -> bool {
        if let Some(first_idx) = self.schedule.games.iter().position(|o| o.home.r == o.away.r) {
            let teams = self.teams.len();
            for idx in first_idx..(first_idx + (teams / 2)) {
                if let Some(game) = self.schedule.games.get_mut(idx) {
                    game.sim(team_data, players, year, &mut rng);
                }
            }
            return true;
        }

        self.teams.sort_by_key(|o| team_data.get(o).unwrap().get_losses());

        false
    }
}

pub(crate) fn end_of_season(leagues: &mut Vec<League>, teams: &mut HashMap<u64, Team>, players: &mut HashMap<u64, Player>, count: usize, year: u32, data: &Data, rng: &mut ThreadRng) {
    // record history
    for (league_idx, league) in leagues.iter().enumerate() {
        for (rank, team_id) in league.teams.iter().enumerate() {
            let team = teams.get_mut(&team_id).unwrap();
            team.record_results(year, league_idx, rank, team.results);

            for player_id in &team.players {
                let player = players.get_mut(player_id).unwrap();
                player.record_stat_history(year, league.id, *team_id);
            }
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
    let mut available = players.iter().filter(|(_, v)| v.active).collect::<HashMap<_, _>>();
    for team in teams.values_mut() {
        team.players.retain(|o| players.get(o).unwrap().active);
        available.retain(|k, _| !team.players.contains(k));
    }

    // repopulate teams
    for team in teams.values_mut() {
        team.populate(&mut available, players);
    }
}
