use std::fs;

pub struct Data {
    pub(crate) loc: Vec<String>,
    pub(crate) nick: Vec<String>,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            loc: Vec::new(),
            nick: Vec::new(),
        }
    }
}

impl Data {
    pub(crate) fn new() -> Data {
        let loc = if let Ok(loc_data) = fs::read_to_string("data/loc.txt") {
            loc_data.lines().map(|o| o.to_string()).collect()
        } else { Vec::new() };

        let nick = if let Ok(nick_data) = fs::read_to_string("data/nick.txt") {
            nick_data.lines().map(|o| o.to_string()).collect()
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

    pub(crate) fn pull_loc(&mut self) -> String {
        Data::pull(&mut self.loc)
    }
    pub(crate) fn pull_nick(&mut self) -> String {
        Data::pull(&mut self.nick)
    }
}
