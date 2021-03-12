use rand::rngs::ThreadRng;
use crate::team::Team;
use crate::schedule::Schedule;
use crate::data::Data;

pub struct League {
    pub(crate) teams: Vec<Team>,
    schedule: Schedule,
}

impl League {
    pub fn new(data: &mut Data, team_count: usize) -> League {
        let mut teams = Vec::<Team>::new();
        for _ in 0..team_count {
            teams.push(Team::new(data));
        }

        let schedule = Schedule::new(team_count);

        League {
            teams,
            schedule,
        }
    }

    pub fn reset(&mut self) {
        for team in &mut self.teams {
            team.results.reset();
        }
    }

    pub fn sim(&mut self, mut rng: &mut ThreadRng) {
        for game in &mut self.schedule.games {
            game.sim(&mut rng);

            if game.home.r > game.away.r {
                self.teams[game.home.team].results.win += 1;
                self.teams[game.away.team].results.lose += 1;
            } else {
                self.teams[game.away.team].results.win += 1;
                self.teams[game.home.team].results.lose += 1;
            }
        }

        self.teams.sort_by_key(|o| o.results.lose);
    }
}

pub fn relegate_promote(leagues: &mut Vec<League>, count: usize) {
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
}
