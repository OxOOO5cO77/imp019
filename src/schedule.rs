use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::game::Game;
use crate::team::TeamId;

#[derive(Default)]
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
