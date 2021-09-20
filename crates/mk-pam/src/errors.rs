//! Error types.

use std::ffi::NulError;
use std::io;
use std::str::Utf8Error;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::ffi;

pub type Result<T> = core::result::Result<T, Error>;

/// All possible error types.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Internal PAM library error.
    #[error("{0}")]
    Raw(#[from] PamError),

    /// IO Error.
    #[error("{0}")]
    Io(#[from] io::Error),
}

impl From<Utf8Error> for Error {
    fn from(e: Utf8Error) -> Self {
        Self::Io(io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

impl From<NulError> for Error {
    fn from(e: NulError) -> Self {
        Self::Io(e.into())
    }
}

/// PAM return codes, treated as errors for easier handling.
///
/// See [`pam(3)`](https://linux.die.net/man/3/pam) for more.
#[derive(thiserror::Error, IntoPrimitive, TryFromPrimitive, Debug, Clone)]
#[repr(i32)]
pub enum PamError {
    /// Critical error.
    #[error("critical error")]
    Abort = ffi::PAM_ABORT as i32,

    /// User account has expired.
    #[error("account expired")]
    AcctExpired = ffi::PAM_ACCT_EXPIRED as i32,

    /// Authentication information is unavailable.
    #[error("authentication info unavailable")]
    AuthInfoUnavailable = ffi::PAM_AUTHINFO_UNAVAIL as i32,

    /// Authentication token aging is disabled.
    #[error("authentication token aging disabled")]
    AuthTokenAgingDisabled = ffi::PAM_AUTHTOK_DISABLE_AGING as i32,

    /// Authentication token manipulation error.
    #[error("authentication token error")]
    AuthToken = ffi::PAM_AUTHTOK_ERR as i32,

    /// Authentication token has expired.
    #[error("authentication token expired")]
    AuthTokenExpired = ffi::PAM_AUTHTOK_EXPIRED as i32,

    /// Authentication token lock is busy.
    #[error("authentication token lock busy")]
    AuthTokenLockBusy = ffi::PAM_AUTHTOK_LOCK_BUSY as i32,

    /// Authentication information cannot be recovered.
    #[error("authentication token recovery error")]
    AuthTokenRecovery = ffi::PAM_AUTHTOK_RECOVERY_ERR as i32,

    /// Authentication error.
    #[error("authentication error")]
    Auth = ffi::PAM_AUTH_ERR as i32,

    /// Memory Buffer error.
    #[error("buffer error")]
    Buffer = ffi::PAM_BUF_ERR as i32,

    /// Conversation error.
    #[error("conversation error")]
    Conversation = ffi::PAM_CONV_ERR as i32,

    /// Failure setting user credentials.
    #[error("credentials error")]
    Creds = ffi::PAM_CRED_ERR as i32,

    /// User credentials expired.
    #[error("credentials expired")]
    CredsExpired = ffi::PAM_CRED_EXPIRED as i32,

    /// Insufficient credentials to access authentication data.
    #[error("insufficient credentials")]
    CredsInsufficient = ffi::PAM_CRED_INSUFFICIENT as i32,

    /// Underlying authentication service could not retrieve user credentials.
    #[error("unavailable credentials")]
    CredsUnavailable = ffi::PAM_CRED_UNAVAIL as i32,

    /// Ignore underlying account module regardless of whether
    /// the control flag is required, optional, or sufficient.
    #[error("ignore module")]
    Ignore = ffi::PAM_IGNORE as i32,

    /// An authentication service has maintained a retry count which has been reached.
    #[error("maximum tries reached")]
    MaxTries = ffi::PAM_MAXTRIES as i32,

    /// Unknown module.
    #[error("unknown module")]
    UnknownModule = ffi::PAM_MODULE_UNKNOWN as i32,

    /// New authentication token required.
    ///
    /// This is normally returned if the machine security policies require
    /// that the password should be changed because the password is NULL or it has aged.
    #[error("new authentication token required")]
    NewAuthTokenRequired = ffi::PAM_NEW_AUTHTOK_REQD as i32,

    /// No module specific data is present.
    #[error("no module data")]
    NoModuleData = ffi::PAM_NO_MODULE_DATA as i32,

    /// `dlopen()` failure when dynamically loading a service module.
    #[error("`dlopen()` failure when dynamically loading a service module")]
    Open = ffi::PAM_OPEN_ERR as i32,

    /// Permission Denied.
    #[error("permission denied")]
    PermissionDenied = ffi::PAM_PERM_DENIED as i32,

    /// Service error.
    #[error("service error")]
    Service = ffi::PAM_SERVICE_ERR as i32,

    /// Can not make/remove an entry for the specified session.
    #[error("session error")]
    Session = ffi::PAM_SESSION_ERR as i32,

    /// Symbol error.
    #[error("symbol error")]
    Symbol = ffi::PAM_SYMBOL_ERR as i32,

    /// System error.
    #[error("system error")]
    System = ffi::PAM_SYSTEM_ERR as i32,

    /// Failed preliminary check by password service.
    #[error("preliminary check by password service.")]
    TryAgain = ffi::PAM_TRY_AGAIN as i32,

    /// User not known to the underlying authentication service.
    #[error("could not find user")]
    UnknownUser = ffi::PAM_USER_UNKNOWN as i32,

    // Target specific codes
    // ---------------------

    // OpenPAM
    // -------
    /// Bad constant.
    #[cfg(feature = "open-pam")]
    #[error("bad constant")]
    BadConstant = ffi::PAM_BAD_CONSTANT as i32,

    /// Unrecognized or restricted feature.
    #[cfg(feature = "open-pam")]
    #[error("bad feature")]
    BadFeature = ffi::PAM_BAD_FEATURE as i32,

    /// Invalid PAM handle.
    #[cfg(feature = "open-pam")]
    #[error("bad handle")]
    BadHandle = ffi::PAM_BAD_HANDLE as i32,

    // Linux-PAM
    // ---------
    /// No data available.
    ///
    /// The conversation function is event driven. This occurs when there is
    /// no data available yet.
    #[cfg(feature = "linux-pam")]
    #[error("no data available")]
    ConversationAgain = ffi::PAM_CONV_AGAIN as i32,

    /// Incomplete authentication stack.
    ///
    /// Verify that the conversation is completed, and call the function again
    /// to complete the authentication stack.
    #[cfg(feature = "linux-pam")]
    #[error("incomplete authentication stack")]
    Incomplete = ffi::PAM_INCOMPLETE as i32,

    // OpenPAM + Linux-PAM
    // -------------------
    /// Unrecognized or restricted item.
    #[cfg(any(feature = "linux-pam", feature = "open-pam"))]
    #[error("bad item")]
    BadItem = ffi::PAM_BAD_ITEM as i32,
}
