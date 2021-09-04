#![doc = include_str!("../README.md")]
#![feature(never_type)]

#[macro_use]
mod prompt;

mod app;
#[allow(unused)]
mod auth;
mod cli;
mod config;
pub mod errors;
mod options;
mod prelude;
pub mod util;

pub use errors::*;

fn main() {
    let mut app = app::App::new(config::Config {}).unwrap();
    app.run(cli::options_from_terminal().unwrap()).unwrap();
}
