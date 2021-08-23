//! User authentication agents.

pub mod pam;

/// Provides methods to authenticate a user. Additional required information must be
/// held by the implementer.
pub trait Authenticator {
    /// Authenticate the user `username` with their `password`.
    fn authenticate(&self, username: &str, password: &str) -> Result<(), ()>;
}
