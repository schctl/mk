#![doc = include_str!("../README.md")]

mod auth;
mod env;
mod errors;

fn main() {
    let mut authenticator = auth::shadow::ShadowAuthenticator::new();

    let mut env = env::Env::new(
        pwd::Passwd::from_uid(nix::unistd::getuid().as_raw()).unwrap(),
        pwd::Passwd::from_uid(nix::unistd::geteuid().as_raw()).unwrap(),
    );
    env.init_args().unwrap();
    env.init_vars().unwrap();

    auth::authenticated_session(
        &mut authenticator,
        &std::env::args().nth(1).unwrap()[..],
        &env,
    )
    .unwrap();
}
