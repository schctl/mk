#![doc = include_str!("../README.md")]
#![feature(never_type)]

#[macro_use]
pub mod util;
pub mod errors;

mod app;
mod auth;
mod cli;
mod config;
mod options;
mod prelude;

pub use errors::*;

fn main() {
    let mut app = app::App::new(config::Config {}).unwrap();
    app.run(cli::options_from_terminal().unwrap()).unwrap();
}
