#![doc = include_str!("../README.md")]

mod auth;
mod env;

fn main() {
    let uid = nix::unistd::getuid();
    let euid = nix::unistd::geteuid();

    let mut authenticator = auth::shadow::ShadowAuthenticator::new();

    auth::authenticated_session(
        &mut authenticator,
        uid,
        euid,
        &std::env::args().nth(1).unwrap()[..],
    )
    .unwrap();
}
