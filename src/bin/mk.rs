//! Runs the `mk` cli.

use mk::cli;

fn main() {
    cli::run(std::env::args());
}
