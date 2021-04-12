use rand::prelude::*;
use rand_distr::{Normal, Gamma};

pub(crate) fn gen_normal(rng: &mut ThreadRng, mean: f64, stddev: f64) -> f64 {
    Normal::new(mean, stddev).unwrap().sample(rng)
}

pub(crate) fn gen_gamma(rng: &mut ThreadRng, shape: f64, scale: f64) -> f64 {
    Gamma::new(shape, scale).unwrap().sample(rng)
}
