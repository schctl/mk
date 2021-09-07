#![doc = include_str!("../README.md")]
#![feature(never_type)]

#[macro_use]
pub mod util;

mod auth;
mod cli;
mod config;
mod errors;
mod options;
mod prelude;

pub use errors::*;

use config::Config;

fn main() {
    let mut app = cli::App::new(Config {}).unwrap();

    match app.run(cli::options_from_terminal().unwrap()) {
        Err(e) => eprintln!("{}", e),
        Ok(Some(e)) => {
            println!("Process exited with status: {}", e);
            ::std::process::exit(e);
        }
        _ => {}
    };
}
