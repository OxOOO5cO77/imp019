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
        let loc = include_str!("../data/loc.txt").lines().map(|o| o.to_string()).collect();
        let nick = include_str!("../data/nick.txt").lines().map(|o| o.to_string()).collect();

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
