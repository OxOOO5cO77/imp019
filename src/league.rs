use std::cmp::{max, min};

use rand::rngs::ThreadRng;

use crate::data::Data;
use crate::schedule::Schedule;
use crate::team::Team;

pub struct League {
    id: u32,
    pub(crate) teams: Vec<Team>,
    pub(crate) schedule: Schedule,
}

impl League {
    pub(crate) fn new(id: u32, data: &mut Data, team_count: usize, year: u32, team_id: &mut u64, player_id: &mut u64, rng: &mut ThreadRng) -> League {
        let mut teams = Vec::<Team>::new();
        for _ in 0..team_count {
            teams.push(Team::new(data, year, team_id, player_id, rng));
        }

        let schedule = Schedule::new(team_count, rng);

        League {
            id,
            teams,
            schedule,
        }
    }

    pub(crate) fn reset_schedule(&mut self, rng: &mut ThreadRng) {
        for team in &mut self.teams {
            team.results.reset();
        }
        self.schedule = Schedule::new(self.teams.len(), rng)
    }

    pub(crate) fn sim(&mut self, mut rng: &mut ThreadRng) -> bool {
        if let Some(first_idx) = self.schedule.games.iter().position(|o| o.home.r == o.away.r) {
            let teams = self.teams.len();
            for idx in first_idx..(first_idx + (teams / 2)) {
                if let Some(game) = self.schedule.games.get_mut(idx) {
                    game.sim(&mut self.teams, &mut rng);

                    if game.home.r > game.away.r {
                        self.teams[game.home.team].results.win += 1;
                        self.teams[game.away.team].results.lose += 1;
                    } else {
                        self.teams[game.away.team].results.win += 1;
                        self.teams[game.home.team].results.lose += 1;
                    }
                }
            }
            return true;
        }

        self.teams.sort_by_key(|o| o.results.lose);

        false
    }
}

pub(crate) fn end_of_season(leagues: &mut Vec<League>, count: usize, year: u32, rng: &mut ThreadRng) {
    // record history
    for (league_idx, league) in leagues.iter_mut().enumerate() {
        for (rank, team) in league.teams.iter_mut().enumerate() {
            team.history.record_results(year, league_idx, rank, team.results);
            team.history.wins += team.results.win;
            team.history.losses += team.results.lose;
            let pos = Some(((league_idx + 1) * 100 + (rank + 1)) as u32);
            team.history.best = if team.history.best.is_none() { pos } else { min(pos, team.history.best) };
            team.history.worst = if team.history.worst.is_none() { pos } else { max(pos, team.history.worst) };

            for player in &mut team.players {
                player.end_of_year(year, league.id, team.id);
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
    for league in leagues {
        league.reset_schedule(rng);
    }
}
