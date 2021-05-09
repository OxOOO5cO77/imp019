use std::collections::{HashMap, HashSet};

use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct LocData {
    pub(crate) abbr: &'static str,
    pub(crate) city: &'static str,
    pub(crate) state: &'static str,
    pub(crate) country: &'static str,
    population: u32,
//    coords: String,
}

impl LocData {
    fn parse(in_str: &'static str) -> Self {
        let mut parts = in_str.split(',');
        let abbr = parts.next().unwrap_or("");
        let city = parts.next().unwrap_or("");
        let state = parts.next().unwrap_or("");
        let country = parts.next().unwrap_or("");
        let population = parts.next().unwrap_or("").parse::<u32>().unwrap_or(0);
//        let coords = parts.next().unwrap_or("").to_owned();
        Self {
            abbr,
            city,
            state,
            country,
            population,
//            coords,
        }
    }
}

pub(crate) struct Data {
    loc: Vec<LocData>,
    nick: Vec<&'static str>,
    names_first: HashMap<&'static str, Vec<(&'static str, u32)>>,
    names_last: HashMap<&'static str, Vec<(&'static str, u32)>>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            loc: Vec::new(),
            nick: Vec::new(),
            names_first: HashMap::new(),
            names_last: HashMap::new(),
        }
    }
}

fn weighted(in_str: &'static str) -> Option<(&'static str, u32)> {
    let mut line = in_str.split(',');
    let value = line.next();
    let weight = line.next().and_then(|o| o.parse::<u32>().ok());

    Some((value?, weight?))
}

impl Data {
    pub(crate) fn new() -> Self {
        let loc = include_str!("../data/loc.txt").lines().map(|o| LocData::parse(o)).collect();
        let nick = include_str!("../data/nick.txt").lines().collect();

        let mut names_first = HashMap::new();
        names_first.insert("US", include_str!("../data/names_us_first.txt").lines().map(weighted).filter_map(|o| o).collect());
        names_first.insert("CA", include_str!("../data/names_ca_first.txt").lines().map(weighted).filter_map(|o| o).collect());
        names_first.insert("MX", include_str!("../data/names_mx_first.txt").lines().map(weighted).filter_map(|o| o).collect());
        let mut names_last = HashMap::new();
        names_last.insert("US", include_str!("../data/names_us_last.txt").lines().map(weighted).filter_map(|o| o).collect());
        names_last.insert("CA", include_str!("../data/names_ca_last.txt").lines().map(weighted).filter_map(|o| o).collect());
        names_last.insert("MX", include_str!("../data/names_mx_last.txt").lines().map(weighted).filter_map(|o| o).collect());

        Self {
            loc,
            nick,
            names_first,
            names_last,
        }
    }

    pub(crate) fn get_locs(&self, existing: &mut HashSet<LocData>, rng: &mut ThreadRng, count: usize) -> Vec<LocData> {
        while existing.len() != count {
            existing.insert(self.loc.choose(rng).unwrap().clone());
        }
        existing.iter().cloned().collect()
    }

    pub(crate) fn get_nicks(&self, nicks: &mut HashSet<String>, rng: &mut ThreadRng, count: usize) -> Vec<String> {
        while nicks.len() != count {
            nicks.insert(self.nick.choose(rng).unwrap().to_string());
        }
        nicks.iter().cloned().collect()
    }

    pub(crate) fn choose_name_first(&self, country: &str, rng: &mut ThreadRng) -> &'static str {
        if let Ok(first_name) = self.names_first.get(country).unwrap().choose_weighted(rng, |o| o.1) {
            first_name.0
        } else {
            ""
        }
    }

    pub(crate) fn choose_name_last(&self, country: &str, rng: &mut ThreadRng) -> &'static str {
        if let Ok(last_name) = self.names_last.get(country).unwrap().choose_weighted(rng, |o| o.1) {
            last_name.0
        } else {
            ""
        }
    }

    pub(crate) fn choose_location(&self, rng: &mut ThreadRng) -> &LocData {
        self.loc.choose_weighted(rng, |o| o.population).unwrap()
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
