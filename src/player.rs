use rand::rngs::ThreadRng;

use crate::data::Data;
use rand::seq::SliceRandom;

pub(crate) struct Player {
    name_first: String,
    name_last: String,
}

impl Player {
    pub(crate) fn new(data: &Data, rng: &mut ThreadRng) -> Self {
        let name_first = data.names_first.choose_weighted(rng, |o| o.1).unwrap().0.clone();
        let name_last = data.names_last.choose_weighted(rng, |o| o.1).unwrap().0.clone();

        Player {
            name_first,
            name_last,
        }
    }

    pub(crate) fn fullname(&self) -> String {
        format!("{} {}", self.name_first, self.name_last)
    }
}
