//! User authentication agents.

use nix::unistd::{Gid, Uid};
use std::ffi::CString;

use crate::env;

pub mod pam;
pub mod shadow;

/// Provides methods to authenticate a user.
//
// Additional required information must be held by the implementer. The intention is for an
/// Authenticator to be dumped to a file and recovered between sessions.
pub trait Authenticator {
    /// Authenticate a user to be able to run as target user.
    fn authenticate(&mut self, user: Uid) -> Result<(), ()>;
}

/// Set all groups and uid values to the target `user`.
fn prepare_user(user: &pwd::Passwd) -> Result<(), ()> {
    // Set target Gid.
    let gid = Gid::from_raw(user.gid);
    nix::unistd::setresgid(gid, gid, gid).unwrap();

    // Initialize groups
    let name = CString::new(&user.name[..]).unwrap();
    nix::unistd::initgroups(name.as_c_str(), gid).unwrap();

    // Set target Uid.
    let uid = Uid::from_raw(user.uid);
    nix::unistd::setresuid(uid, uid, uid).unwrap();

    Ok(())
}

/// Authenticate a user and start a process with the `target` Uid and arguments.
///
/// After a user has been authenticated, they will be allowed to run a command
/// as any user they have been permitted to.
pub fn authenticated_session<T: Authenticator>(
    authenticator: &mut T,
    command: &str,
    env: &env::Env,
) -> Result<(), ()> {
    // First, check if the user is authenticated.
    authenticator.authenticate(Uid::from_raw(env.origin.uid))?;

    // Check if `user` is permitted to run as `target`.
    // TODO

    prepare_user(&env.target)?;

    nix::unistd::execvpe(
        &CString::new(command).unwrap()[..],
        env.get_args(),
        env.get_vars(),
    )
    .unwrap();

    Ok(())
}
