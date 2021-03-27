use std::collections::HashSet;

use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

pub(crate) struct Data {
    loc: Vec<String>,
    nick: Vec<String>,
    names_first: Vec<(String, u32)>,
    names_last: Vec<(String, u32)>,
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

    pub(crate) fn get_locs(&self, existing: &mut HashSet<(String, String, String)>, rng: &mut ThreadRng, count: usize) -> Vec<(String, String, String)> {
        while existing.len() != count {
            let mut loc = self.loc.choose(rng).unwrap().split(',');
            let abbr = loc.next().unwrap_or("").to_owned();
            let city = loc.next().unwrap_or("").to_owned();
            let state = format!("{}-{}", loc.next().unwrap_or(""), loc.next().unwrap_or(""));

            existing.insert((abbr, city, state));
        }
        existing.iter().cloned().collect()
    }

    pub(crate) fn get_nicks(&self, nicks: &mut HashSet<String>, rng: &mut ThreadRng, count: usize) -> Vec<String> {
        while nicks.len() != count {
            nicks.insert(self.nick.choose(rng).unwrap().to_owned());
        }
        nicks.iter().cloned().collect()
    }

    pub(crate) fn choose_name_first(&self, rng: &mut ThreadRng) -> String {
        self.names_first.choose_weighted(rng, |o| o.1).unwrap().0.clone()
    }

    pub(crate) fn choose_name_last(&self, rng: &mut ThreadRng) -> String {
        self.names_last.choose_weighted(rng, |o| o.1).unwrap().0.clone()
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
