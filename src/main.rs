#![doc = include_str!("../README.md")]

mod auth;

use auth::Authenticator;

fn main() {
    let authenticator = auth::pam::PamAuthenticator {};

    authenticator
        .authenticate(
            std::env::var("MK_USER").unwrap().as_str(),
            std::env::var("MK_PWD").unwrap().as_str(),
        )
        .unwrap();
}
