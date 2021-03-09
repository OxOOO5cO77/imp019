use crate::player::Player;
use crate::results::Results;
use crate::data::Data;

pub struct Team {
    pub abbr: String,
    city: String,
    state: String,
    nickname: String,
    players: Vec<Player>,
    pub results: Results,
}

impl Team {
    pub fn new(data: &mut Data) -> Self {
        let loc = data.pull_loc();
        let mut loc = loc.split(',');
        Team {
            abbr: loc.next().unwrap().into(),
            city: loc.next().unwrap().into(),
            state: loc.next().unwrap().into(),
            nickname: data.pull_nick(),
            players: vec![],
            results: Results::new(),
        }
    }

    fn name(&self) -> String {
        format!("{} {} ({})", self.city, self.nickname, self.state)
    }
}
