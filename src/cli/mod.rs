//! Command line tools for `mk`.

use std::process::exit;

mod app;
mod options;

use crate::config::Config;
use crate::prelude::*;

fn exit_with_err(err: MkError) -> ! {
    eprintln!("{}: {}", SERVICE_NAME, err);
    exit(0);
}

pub fn run<I, T>(args: I)
where
    I: IntoIterator<Item = T>,
    T: Into<String> + Clone,
{
    let mut app = match app::App::new(Config {}) {
        Err(e) => exit_with_err(e),
        Ok(a) => a,
    };

    let opts = match options::from_terminal(args) {
        Err(e) => exit_with_err(e),
        Ok(o) => o,
    };

    match app.run(opts) {
        Err(e) => exit_with_err(e),
        Ok(Some(e)) => exit(e),
        _ => {}
    };
}
