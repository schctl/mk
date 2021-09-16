//! Command line tools for `mk`.

use std::process::exit;

mod app;
mod options;

use crate::config::Config;
use crate::prelude::*;

fn exit_with_err(err: Error) -> ! {
    eprintln!("{}: {}", SERVICE_NAME, err);
    exit(0);
}

pub fn run<I, T>(args: I) -> !
where
    I: IntoIterator<Item = T>,
    T: Into<String> + Clone,
{
    let opts = match options::from_terminal(args) {
        Err(e) => exit_with_err(e),
        Ok(i) => i,
    };

    let conf = match Config::from_file("/etc/mk.conf") {
        Err(e) => exit_with_err(e),
        Ok(i) => i,
    };

    let mut app = match app::App::new(conf) {
        Err(e) => exit_with_err(e),
        Ok(i) => i,
    };

    match app.run(opts) {
        Err(e) => exit_with_err(e),
        Ok(Some(e)) => exit(e),
        _ => exit(0),
    }
}
