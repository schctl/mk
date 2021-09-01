#![doc = include_str!("../README.md")]
#![feature(never_type)]

#[allow(unused)]
mod auth;
mod config;
mod env;
mod errors;
mod prelude;
mod prompt;
pub mod util;

use config::Config;
use env::Env;

fn main() {
    let mut env = Env::new(Config {});

    let mut raw_args = std::env::args();

    // Path to this binary
    let _ = raw_args.next();

    if let Some(c) = raw_args.next() {
        let mut args = Vec::with_capacity(raw_args.len());

        for arg in raw_args {
            args.push(arg)
        }

        eprintln!(
            "{}",
            env.exec(
                &c[..],
                args,
                &mk_pwd::Passwd::from_uid(util::get_euid()).unwrap(),
            )
        );
    }
}
