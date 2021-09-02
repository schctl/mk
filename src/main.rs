#![doc = include_str!("../README.md")]

#[allow(unused)]
mod auth;
mod config;
mod env;
mod errors;
mod options;
mod prelude;
mod prompt;
pub mod util;

use config::Config;
use env::Env;
use options::MkOptions;

fn main() {
    let mut env = Env::new(Config {});
    env.run(MkOptions::new()).unwrap();
}
