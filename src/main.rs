#![doc = include_str!("../README.md")]
#![feature(never_type)]

mod auth;
mod command;
mod env;
mod errors;
mod prelude;

fn main() {
    let authenticator = auth::shadow::ShadowAuthenticator::new();

    let mut env = env::Env::new(
        pwd::Passwd::from_uid(nix::unistd::getuid().as_raw()).unwrap(),
        pwd::Passwd::from_uid(nix::unistd::geteuid().as_raw()).unwrap(),
    );
    env.init_args().unwrap();
    env.init_vars().unwrap();

    // Execute the first argument as a command - for now.
    command::CommandExecutor::new(env, Box::new(authenticator))
        .exec_cmd(&std::env::args().nth(1).unwrap()[..])
        .unwrap();
}
