use std::fs;

pub struct Data {
    pub loc: Vec<String>,
    pub nick: Vec<String>,
}

impl Data {
    pub fn new() -> Data {
        let loc = if let Ok(loc_data) = fs::read_to_string("data/loc.txt") {
            loc_data.lines().map(|o| o.into()).collect()
        } else { Vec::new() };

        let nick = if let Ok(nick_data) = fs::read_to_string("data/nick.txt") {
            nick_data.lines().map(|o| o.into()).collect()
        } else { Vec::new() };

        Data {
            loc,
            nick,
        }
    }

    fn pull(vec: &mut Vec<String>) -> String {
        if let Some(result) = vec.pop() {
            result
        } else {
            "".into()
        }
    }

    pub fn pull_loc(&mut self) -> String {
        Data::pull(&mut self.loc)
    }
    pub fn pull_nick(&mut self) -> String {
        Data::pull(&mut self.nick)
    }
}
