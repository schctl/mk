//! Environment setup utilities for running a command.

use std::collections::HashMap;

use crate::prelude::*;

/// Command execution environment.
///
/// This is passed to [`crate::command::CommandExecutor`] and wraps all execution details, such
/// as the user's `mk` config, options, target user, environment variables, and arguments.
pub struct Env {
    pub origin: pwd::Passwd,
    pub target: pwd::Passwd,
    args: Vec<String>,
    vars: HashMap<String, String>,
}

impl Env {
    #[must_use]
    pub fn new(origin: pwd::Passwd, target: pwd::Passwd) -> Self {
        Self {
            origin,
            target,
            args: Vec::new(),
            vars: HashMap::new(),
        }
    }

    /// Initialize argument list from existing environment.
    pub fn init_args(&mut self) -> MkResult<()> {
        let mut args = std::env::args();

        // First arg is path to this binary.
        let _ = args.next();

        if args.next().is_some() {
            self.args = Vec::with_capacity(args.len());

            for i in args {
                self.push_arg(&i[..])?;
            }
        }

        Ok(())
    }

    /// Initialize basic environment variable list.
    pub fn init_vars(&mut self) -> MkResult<()> {
        let target = self.target.clone();

        self.push_var("USER", &target.name[..])?;
        self.push_var("HOME", &target.dir[..])?;
        self.push_var("SHELL", &target.shell[..])?;

        self.push_var(
            "PATH",
            &if let Some(p) = std::env::vars().find(|p| p.0 == "PATH") {
                p.1
            } else {
                String::from("/usr/local/sbin:/usr/local/bin:/usr/bin")
            }[..],
        )?;

        self.push_var("LOGNAME", &target.name[..])?;

        Ok(())
    }

    /// Push an argument.
    pub fn push_arg(&mut self, arg: &str) -> MkResult<()> {
        self.args.push(String::from(arg));
        Ok(())
    }

    /// Push a key value pair to the environment variable list.
    pub fn push_var(&mut self, key: &str, val: &str) -> MkResult<()> {
        self.vars.insert(key.to_string(), val.to_string());
        Ok(())
    }

    /// Get environment argument list.
    pub fn get_args(&self) -> &[String] {
        &self.args[..]
    }

    /// Get environment variable list.
    pub fn get_vars(&self) -> &HashMap<String, String> {
        &self.vars
    }
}
