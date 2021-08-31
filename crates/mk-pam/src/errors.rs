//! Error types.

use mk_common::errors::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use thiserror::Error as ThisError;

use crate::ffi;

pub type PamResult<T> = Result<T, PamError>;

/// All possible error types.
#[derive(ThisError, Debug)]
pub enum PamError {
    /// Raw PAM error code.
    #[error("Raw PAM error code")]
    Raw(#[from] RawError),

    /// FFI error.
    #[error("FFI error")]
    Ffi(FfiError),
}

impl<T> From<T> for PamError
where
    T: Into<FfiError>,
{
    fn from(e: T) -> Self {
        Self::Ffi(e.into())
    }
}

/// PAM return codes, which we treat as errors for easier handling.
#[derive(ThisError, IntoPrimitive, TryFromPrimitive, Debug, Clone)]
#[repr(i32)]
pub enum RawError {
    /// `dlopen()` failure when dynamically loading a service module.
    #[error("`dlopen()` failure when dynamically loading a service module.")]
    Open = ffi::PAM_OPEN_ERR as i32,

    /// Symbol error.
    #[error("Symbol error")]
    Symbol = ffi::PAM_SYMBOL_ERR as i32,

    /// Service error.
    #[error("Service error")]
    Service = ffi::PAM_SERVICE_ERR as i32,

    /// System error.
    #[error("System error")]
    System = ffi::PAM_SYSTEM_ERR as i32,

    /// Memory Buffer error.
    #[error("Buffer error")]
    Buffer = ffi::PAM_BUF_ERR as i32,

    /// Permission Denied.
    #[error("Permission Denied")]
    Permission = ffi::PAM_PERM_DENIED as i32,

    /// Authentication error.
    #[error("Authentication error")]
    Authentication = ffi::PAM_AUTH_ERR as i32,

    /// Could not access authentication data due to insufficient credentials.
    #[error("Could not access authentication data due to insufficient credentials")]
    InsufficientCreds = ffi::PAM_CRED_INSUFFICIENT as i32,

    /// Underlying authentication service could not retrieve authentication information.
    #[error("Authentication info unavailable")]
    AuthInfoUnavailable = ffi::PAM_AUTHINFO_UNAVAIL as i32,

    /// User not known to the underlying authentication service.
    #[error("Unknown user")]
    UnknownUser = ffi::PAM_USER_UNKNOWN as i32,

    /// An authentication service has maintained a retry count which has been reached.
    #[error("Maximum tries reached")]
    MaxTries = ffi::PAM_MAXTRIES as i32,

    /// New authentication token required.
    ///
    /// This is normally returned if the machine security policies require
    /// that the password should be changed because the password is NULL or it has aged.
    #[error("New auth token required")]
    NewAuthTokenRequired = ffi::PAM_NEW_AUTHTOK_REQD as i32,

    /// User account has expired.
    #[error("User account expired")]
    AccountExpired = ffi::PAM_ACCT_EXPIRED as i32,

    /// Can not make/remove an entry for the specified session.
    #[error("Session error")]
    Session = ffi::PAM_SESSION_ERR as i32,

    /// Underlying authentication service could not retrieve user credentials.
    #[error("Unavailable credentials")]
    CredsUnavailable = ffi::PAM_CRED_UNAVAIL as i32,

    /// User credentials expired.
    #[error("Credentials expired")]
    CredsExpired = ffi::PAM_CRED_EXPIRED as i32,

    /// Failure setting user credentials.
    #[error("Credentials error")]
    Creds = ffi::PAM_CRED_ERR as i32,

    /// No module specific data is present.
    #[error("No module data")]
    NoModuleData = ffi::PAM_NO_MODULE_DATA as i32,

    /// Conversation error.
    #[error("Conversation error")]
    Conversation = ffi::PAM_CONV_ERR as i32,

    /// Authentication token manipulation error.
    #[error("Auth token error")]
    AuthToken = ffi::PAM_AUTHTOK_ERR as i32,

    /// Authentication information cannot be recovered.
    #[error("Auth token recovery error")]
    AuthTokenRecovery = ffi::PAM_AUTHTOK_RECOVERY_ERR as i32,

    /// Authentication token lock busy.
    #[error("Auth token lock busy")]
    AuthTokenLockBusy = ffi::PAM_AUTHTOK_LOCK_BUSY as i32,

    /// Authentication token aging disabled.
    #[error("Auth token aging disabled")]
    AuthTokenAgingDisabled = ffi::PAM_AUTHTOK_DISABLE_AGING as i32,

    /// Preliminary check by password service.
    #[error("Preliminary check by password service.")]
    TryAgain = ffi::PAM_TRY_AGAIN as i32,

    /// Ignore underlying account module regardless of whether
    /// the control flag is required, optional, or sufficient.
    #[error("Ignore module")]
    Ignore = ffi::PAM_IGNORE as i32,

    /// Critical error.
    #[error("Critical error")]
    Abort = ffi::PAM_ABORT as i32,

    /// User authentication token has expired.
    #[error("Auth token expired")]
    AuthTokenExpired = ffi::PAM_AUTHTOK_EXPIRED as i32,

    /// Unknown module.
    #[error("Unknown module")]
    UnknownModule = ffi::PAM_MODULE_UNKNOWN as i32,

    /// Bad item received.
    #[error("Bad item")]
    BadItem = ffi::PAM_BAD_ITEM as i32,

    /// No data available.
    ///
    /// The conversation function is event driven. This occurs when there is
    /// no data available yet.
    #[error("No data available")]
    ConversationAgain = ffi::PAM_CONV_AGAIN as i32,

    /// Incomplete authentication stack.
    ///
    /// Verify that the conversation is completed, and call the function again
    /// to complete the authentication stack
    #[error("Incomplete authentication stack")]
    Incomplete = ffi::PAM_INCOMPLETE as i32,
}
