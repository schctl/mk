//! Command line tools for `mk`.

mod app;
mod options;

pub use app::*;
pub use options::*;

use crate::config::Config;

pub fn run<I, T>(iter: I)
where
    I: IntoIterator<Item = T>,
    T: Into<String> + Clone,
{
    let mut app = App::new(Config {}).unwrap();

    match app.run(options_from_terminal(iter).unwrap()) {
        Err(e) => eprintln!("{}", e),
        Ok(Some(e)) => ::std::process::exit(e),
        _ => {}
    };
}
