//! Environment setup utilities for running a command.

use std::collections::HashMap;

use clap::{App, AppSettings, Arg};

use crate::options::*;
use crate::prelude::*;

/// Parse runtime options from the command line using [`clap`].
pub fn options_from_terminal() -> MkResult<MkOptions> {
    let mut app = App::new(SERVICE_NAME)
        .about(DESCRIPTION)
        .version(VERSION)
        .setting(AppSettings::AllowExternalSubcommands)
        .arg(
            Arg::new("user")
                .short('u')
                .long("user")
                .takes_value(true)
                .about("User to run the command as.")
                .default_value("root"),
        )
        .arg(
            Arg::new("preserve-env")
                .long("preserve-env")
                .about("Preserve existing environment variables"),
        );

    let usage = app.generate_usage();
    let matches = app.get_matches();

    Ok(match matches.subcommand() {
        Some((ext_cmd, ext_args)) => MkOptions::Command(CommandOptions {
            target: match matches.value_of("user") {
                Some(u) => mk_pwd::Passwd::from_name(u)?,
                _ => panic!(),
            },
            command: ext_cmd.to_string(),
            args: match ext_args.values_of("") {
                Some(v) => v.into_iter().map(|s| s.to_string()).collect(),
                None => Vec::new(),
            },
            env: match matches.values_of("preserve-env") {
                Some(vals) => {
                    let mut env = HashMap::new();
                    for e in vals {
                        match std::env::var(e) {
                            Ok(v) => {
                                env.insert(e.to_string(), v);
                            }
                            _ => continue,
                        }
                    }
                    Some(env)
                }
                _ => None,
            },
        }),
        _ => MkOptions::Help(usage),
    })
}
