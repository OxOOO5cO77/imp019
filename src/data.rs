pub struct Data {
    pub(crate) loc: Vec<String>,
    pub(crate) nick: Vec<String>,
    pub(crate) names_first: Vec<(String, u32)>,
    pub(crate) names_last: Vec<(String, u32)>,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            loc: Vec::new(),
            nick: Vec::new(),
            names_first: Vec::new(),
            names_last: Vec::new(),
        }
    }
}

fn weighted(in_str: &str) -> Option<(String, u32)> {
    let mut line = in_str.split(',');
    let value = line.next();
    let weight = line.next().and_then(|o| o.parse::<u32>().ok());

    Some((value?.to_owned(), weight?))
}

impl Data {
    pub(crate) fn new() -> Data {
        let loc = include_str!("../data/loc.txt").lines().map(|o| o.to_string()).collect();
        let nick = include_str!("../data/nick.txt").lines().map(|o| o.to_string()).collect();
        let names_first = include_str!("../data/names_first.txt").lines().map(weighted).filter_map(|o| o).collect();
        let names_last = include_str!("../data/names_last.txt").lines().map(weighted).filter_map(|o| o).collect();

        Data {
            loc,
            nick,
            names_first,
            names_last,
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_abbr() {
        let mut abbr = include_str!("../data/loc.txt")
            .lines()
            .map(|o| o.split(',').next())
            .filter_map(|o| o)
            .collect::<Vec<_>>();

        abbr.sort_unstable();

        for idx in 1..abbr.len() {
            assert_ne!(abbr[idx - 1], abbr[idx]);
        }
    }
}
