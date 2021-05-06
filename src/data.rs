use std::collections::HashSet;

use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

struct LocData {
    abbr: String,
    city: String,
    state: String,
    country: String,
    population: u32,
//    coords: String,
}

impl LocData {
    fn parse(in_str: &str) -> Self {
        let mut parts = in_str.split(',');
        let abbr = parts.next().unwrap_or("").to_owned();
        let city = parts.next().unwrap_or("").to_owned();
        let state = parts.next().unwrap_or("").to_owned();
        let country = parts.next().unwrap_or("").to_owned();
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
    nick: Vec<String>,
    names_first: Vec<(String, u32)>,
    names_last: Vec<(String, u32)>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
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
    pub(crate) fn new() -> Self {
        let loc = include_str!("../data/loc.txt").lines().map(|o| LocData::parse(o)).collect();
        let nick = include_str!("../data/nick.txt").lines().map(|o| o.to_string()).collect();
        let names_first = include_str!("../data/names_first.txt").lines().map(weighted).filter_map(|o| o).collect();
        let names_last = include_str!("../data/names_last.txt").lines().map(weighted).filter_map(|o| o).collect();

        Self {
            loc,
            nick,
            names_first,
            names_last,
        }
    }

    pub(crate) fn get_locs(&self, existing: &mut HashSet<(String, String, String)>, rng: &mut ThreadRng, count: usize) -> Vec<(String, String, String)> {
        while existing.len() != count {
            let loc = self.loc.choose(rng).unwrap();
            let abbr = loc.abbr.clone();
            let city = loc.city.clone();
            let state = format!("{}-{}", loc.state.clone(), loc.country.clone());

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
        if let Ok(first_name) = self.names_first.choose_weighted(rng, |o| o.1) {
            first_name.0.clone()
        } else {
            "".to_string()
        }
    }

    pub(crate) fn choose_name_last(&self, rng: &mut ThreadRng) -> String {
        if let Ok(last_name) = self.names_last.choose_weighted(rng, |o| o.1) {
            last_name.0.clone()
        } else {
            "".to_string()
        }
    }

    pub(crate) fn choose_location(&self, rng: &mut ThreadRng) -> String {
        if let Ok(loc_data) = self.loc.choose_weighted(rng, |o| o.population) {
            format!("{}, {}, {}", loc_data.city, loc_data.state, loc_data.country )
        } else {
            "".to_string()
        }

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
