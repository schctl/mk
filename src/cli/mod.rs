//! Command line tools for `mk`.

use std::process::exit;

use crate::config::Config;
use crate::prelude::*;

mod app;
mod options;

pub use app::App;

fn exit_with_err(err: &Error) -> ! {
    eprintln!("{}: {}", SERVICE_NAME, err);
    exit(-1);
}

pub fn run(args: Vec<String>) -> ! {
    let opts = match options::from_terminal(args) {
        Err(e) => exit_with_err(&e),
        Ok(i) => i,
    };

    let conf = match Config::from_file("/etc/mk.conf") {
        Err(e) => exit_with_err(&e),
        Ok(i) => i,
    };

    let mut app = match App::new(&conf) {
        Err(e) => exit_with_err(&e),
        Ok(i) => i,
    };

    match app.run(opts) {
        Err(e) => exit_with_err(&e),
        Ok(Some(e)) => exit(e),
        _ => exit(0),
    }
}
