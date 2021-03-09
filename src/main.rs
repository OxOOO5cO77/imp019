use rand::seq::SliceRandom;

use crate::data::Data;
use crate::league::League;

mod team;
mod player;
mod results;
mod league;
mod data;
mod schedule;


fn main() {
    let mut data = Data::new();

    let mut rng = rand::thread_rng();
    data.loc.shuffle(&mut rng);
    data.nick.shuffle(&mut rng);

    let mut leagues = Vec::new();
    leagues.push(League::new(&mut data, 28));
    leagues.push(League::new(&mut data, 28));
    leagues.push(League::new(&mut data, 28));

    for _ in 0..5 {
        for league in &mut leagues {
            league.sim(&mut rng);
        }

        league::relegate_promote(&mut leagues, 4);

        for league in &mut leagues {
            league.reset();
        }
    }
}
