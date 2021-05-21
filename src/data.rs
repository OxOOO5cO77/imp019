use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct LocData {
    pub(crate) abbr: &'static str,
    pub(crate) city: &'static str,
    pub(crate) state: &'static str,
    pub(crate) country: &'static str,
    population: u32,
    lang: &'static str,
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
        let lang = parts.next().unwrap_or("");
//        let coords = parts.next().unwrap_or("").to_owned();
        Self {
            abbr,
            city,
            state,
            country,
            population,
            lang,
//            coords,
        }
    }
}

#[derive(Clone, Eq)]
pub(crate) struct NickData {
    localized: HashMap<&'static str, &'static str>,
}

impl Hash for NickData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.localized.values().next().unwrap_or(&"").hash(state)
    }
}

impl PartialEq for NickData {
    fn eq(&self, other: &Self) -> bool {
        self.localized.values().next().unwrap() == other.localized.values().next().unwrap()
    }
}

impl NickData {
    pub(crate) fn name(&self, location: &LocData) -> &'static str {
        self.localized.get(location.lang).unwrap_or(&"")
    }

    fn parse(in_str: &'static str, headers: &[&'static str]) -> Self {
        Self {
            localized: in_str.split(',').zip(headers).map(|(nick, header)| (*header, nick)).collect::<HashMap<_, _>>()
        }
    }
}

pub(crate) struct Data {
    loc: Vec<LocData>,
    nick: Vec<NickData>,
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
        let mut nick_raw = include_str!("../data/nick.txt").lines();
        let headers = nick_raw.next().unwrap_or("EN").split(',').collect::<Vec<_>>();
        let nick = nick_raw.map(|o| NickData::parse(o, &headers)).collect();

        let mut names_first = HashMap::new();
        names_first.insert("US", include_str!("../data/names_us_first.txt").lines().map(weighted).flatten().collect());
        names_first.insert("CA", include_str!("../data/names_ca_first.txt").lines().map(weighted).flatten().collect());
        names_first.insert("MX", include_str!("../data/names_mx_first.txt").lines().map(weighted).flatten().collect());
        let mut names_last = HashMap::new();
        names_last.insert("US", include_str!("../data/names_us_last.txt").lines().map(weighted).flatten().collect());
        names_last.insert("CA", include_str!("../data/names_ca_last.txt").lines().map(weighted).flatten().collect());
        names_last.insert("MX", include_str!("../data/names_mx_last.txt").lines().map(weighted).flatten().collect());

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

    pub(crate) fn get_nicks(&self, nicks: &mut HashSet<NickData>, rng: &mut ThreadRng, count: usize) -> Vec<NickData> {
        while nicks.len() != count {
            nicks.insert(self.nick.choose(rng).unwrap().clone());
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
            .flatten()
            .collect::<Vec<_>>();

        abbr.sort_unstable();

        for idx in 1..abbr.len() {
            assert_ne!(abbr[idx - 1], abbr[idx]);
        }
    }

    #[test]
    fn test_nick() {
        let mut nick_raw = include_str!("../data/nick.txt").lines();
        let headers = nick_raw.next().unwrap().split(',').collect::<Vec<_>>();
        let nick = nick_raw.map(|o| o.split(',').collect::<Vec<_>>()).collect::<Vec<_>>();

        for line in &nick {
            assert_eq!(line.len(), headers.len())
        }

        let mut error = 0;
        for header in 0..headers.len() {
            let mut local = nick.iter().map(|o| o[header]).collect::<Vec<_>>();
            local.sort_unstable();
            for idx in 1..local.len() {
                if local[idx - 1] == local[idx] {
                    println!("{}: {}", headers[header], local[idx]);
                    error += 1;
                }
            }
        }
        assert_eq!(error, 0, "{} duplicates found.", error);
    }
}
