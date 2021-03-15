use rand::rngs::ThreadRng;

use crate::data::Data;
use crate::schedule::Schedule;
use crate::team::Team;

pub struct League {
    pub(crate) teams: Vec<Team>,
    pub(crate) schedule: Schedule,
}

impl League {
    pub(crate) fn new(data: &mut Data, team_count: usize, rng: &mut ThreadRng) -> League {
        let mut teams = Vec::<Team>::new();
        for _ in 0..team_count {
            teams.push(Team::new(data));
        }

        let schedule = Schedule::new(team_count, rng);

        League {
            teams,
            schedule,
        }
    }

    pub(crate) fn reset_results(&mut self, rng: &mut ThreadRng) {
        for team in &mut self.teams {
            team.results.reset();
        }
        self.schedule = Schedule::new(self.teams.len(),rng)
    }

    pub(crate) fn sim(&mut self, mut rng: &mut ThreadRng) -> bool {
        if let Some(first_idx) = self.schedule.games.iter().position(|o| o.home.r == o.away.r) {
            let teams = self.teams.len();
            for idx in first_idx..(first_idx + (teams / 2)) {
                if let Some(game) = self.schedule.games.get_mut(idx) {
                    game.sim(&mut rng);

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

pub(crate) fn relegate_promote(leagues: &mut Vec<League>, count: usize, rng: &mut ThreadRng) {
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

    for league in leagues {
        league.reset_results(rng);
    }
}
