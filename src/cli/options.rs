//! Environment setup utilities for running a command.

use std::ffi::OsString;

use clap::{App, AppSettings, Arg};

use crate::options::*;
use crate::prelude::*;

/// Parse runtime options from the command line using [`clap`].
pub fn options_from_terminal<I, T>(iter: I) -> MkResult<MkOptions>
where
    I: IntoIterator<Item = T>,
    T: Into<String> + Clone,
{
    let mut app = App::new(SERVICE_NAME)
        .about(DESCRIPTION)
        .version(clap::crate_version!())
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

    let matches = match app.try_get_matches_from(iter.into_iter().map(|s| OsString::from(s.into())))
    {
        Ok(m) => m,
        Err(e) => {
            e.print()
                // If we get here, we're probably going to exit anyway
                .unwrap();
            return Ok(MkOptions::None);
        }
    };

    let opts = match matches.subcommand() {
        Some((ext_cmd, ext_args)) => {
            let target = match matches.value_of("user") {
                Some(u) => match mk_pwd::Passwd::from_name(u) {
                    Ok(p) => p,
                    // Could not find user from the password database
                    Err(e) => {
                        return Ok(MkOptions::Error(format!(
                            "Could not find user `{}`\nReason: \"{}\"",
                            u, e
                        )))
                    }
                },
                _ => panic!(),
            };

            let args = match ext_args.values_of("") {
                Some(v) => v.into_iter().map(|s| s.to_string()).collect(),
                None => Vec::new(),
            };

            // TODO: for now
            let env = None;

            MkOptions::Command(CommandOptions {
                target,
                command: ext_cmd.to_string(),
                args,
                env,
            })
        }
        _ => MkOptions::Text(usage),
    };

    Ok(opts)
}
