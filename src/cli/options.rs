//! CLI option parsing utilities.

use std::path::PathBuf;

use clap::{App, AppSettings, Arg};

use crate::options::*;
use crate::prelude::*;

/// Parse runtime options from the command line using [`clap`].
pub fn from_terminal(args: Vec<String>) -> Result<MkOptions> {
    let mut app = App::new(SERVICE_NAME)
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::AllowExternalSubcommands)
        .setting(AppSettings::ColoredHelp)
        .arg(
            Arg::new("user")
                .short('u')
                .long("user")
                .takes_value(true)
                .about("Target user to run the command as")
                .default_value("root"),
        )
        .arg(
            Arg::new("preserve-env")
                .short('E')
                .long("preserve-env")
                .takes_value(true)
                .about("Preserve existing environment variables"),
        )
        .arg(
            Arg::new("edit")
                .short('e')
                .long("edit")
                .takes_value(true)
                .about("Edit a file as the target user"),
        );

    let usage = app.generate_usage();

    let matches = match app.try_get_matches_from(args) {
        Ok(m) => m,
        Err(e) => {
            e.print()
                // If we get here, we're probably going to exit anyway
                .unwrap();
            return Ok(MkOptions::None);
        }
    };

    let target = mk_pwd::Passwd::from_name(match matches.value_of("user") {
        Some(u) => u,
        None => "root",
    })?;

    // Parse edit options
    if let Some(e) = matches.value_of("edit") {
        return Ok(MkOptions::Edit(EditOptions {
            target,
            path: PathBuf::from(e),
        }));
    }

    // Parse command options from external subcommand
    if let Some((ext_cmd, ext_args)) = matches.subcommand() {
        let args = match ext_args.values_of("") {
            Some(v) => v.into_iter().map(std::borrow::ToOwned::to_owned).collect(),
            _ => Vec::new(),
        };

        return Ok(MkOptions::Command(CommandOptions {
            target,
            command: ext_cmd.to_string(),
            args,
            preserve_env: matches
                .value_of("preserve-env")
                .map(|s| s.split(',').map(std::borrow::ToOwned::to_owned).collect()),
        }));
    }

    Ok(MkOptions::Text(usage))
}
