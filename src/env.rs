//! Environment setup utilities for running a command.

use std::ffi::CString;

use crate::errors::MkError;

/// Holds all environment related initializations.
pub struct Env {
    pub origin: pwd::Passwd,
    pub target: pwd::Passwd,
    args: Vec<CString>,
    vars: Vec<CString>,
}

impl Env {
    #[must_use]
    pub fn new(origin: pwd::Passwd, target: pwd::Passwd) -> Self {
        Self {
            origin,
            target,
            args: vec![],
            vars: vec![],
        }
    }

    /// Initialize argument list from existing environment.
    pub fn init_args(&mut self) -> Result<(), MkError> {
        let mut args = std::env::args();

        if args.next().is_some() {
            self.args = Vec::with_capacity(args.len());

            for i in args {
                self.push_arg(&i[..])?;
            }
        }

        Ok(())
    }

    /// Initialize basic environment variable list.
    pub fn init_vars(&mut self) -> Result<(), MkError> {
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
    pub fn push_arg(&mut self, arg: &str) -> Result<(), MkError> {
        self.args.push(CString::new(arg)?);
        Ok(())
    }

    /// Push a key value pair to the environment variable list.
    pub fn push_var(&mut self, key: &str, val: &str) -> Result<(), MkError> {
        self.vars.push(CString::new(format!("{}={}", key, val))?);
        Ok(())
    }

    /// Get environment argument list.
    pub fn get_args(&self) -> &[CString] {
        &self.args[..]
    }

    /// Get environment variable list.
    pub fn get_vars(&self) -> &[CString] {
        &self.vars[..]
    }
}
