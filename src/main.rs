#![doc = include_str!("../README.md")]
#![feature(never_type)]

pub mod auth;
pub mod command;
pub mod env;
pub mod errors;
pub mod prelude;
pub mod util;

fn main() {
    let authenticator = auth::pam::PamAuthenticator::new();

    let mut env = env::Env::new(
        mk_pwd::Passwd::from_uid(util::get_uid()).unwrap(),
        mk_pwd::Passwd::from_uid(util::get_euid()).unwrap(),
    );
    env.init_args().unwrap();
    env.init_vars().unwrap();

    // Execute the first argument as a command - for now.
    command::CommandExecutor::new(env, Box::new(authenticator))
        .exec_cmd(&std::env::args().nth(1).unwrap()[..])
        .unwrap();
}
