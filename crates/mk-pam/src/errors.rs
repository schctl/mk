//! Linux-PAM error types.

use std::ffi::NulError;

use num_enum::IntoPrimitive;
use thiserror::Error;

use crate::ffi;

pub type PamResult<T> = Result<T, PamError>;

/// All possible error types.
#[derive(Error, Debug, Clone)]
pub enum PamError {
    /// Raw Pam error code.
    #[error("Raw PAM error code")]
    Raw(#[from] RawError),
    /// An interior nul byte was found.
    #[error("An interior nul byte was found")]
    NulError(#[from] NulError),
}

/// Pam return codes, which we treat as errors for easier handling.
#[repr(u32)]
#[derive(Error, IntoPrimitive, Debug, Clone)]
pub enum RawError {
    /// `dlopen()` failure when dynamically loading a service module.
    #[error("`dlopen()` failure when dynamically loading a service module.")]
    Open = ffi::PAM_OPEN_ERR,

    /// Symbol error.
    #[error("Symbol error")]
    Symbol = ffi::PAM_SYMBOL_ERR,

    /// Service error.
    #[error("Service error")]
    Service = ffi::PAM_SERVICE_ERR,

    /// System error.
    #[error("System error")]
    System = ffi::PAM_SYSTEM_ERR,

    /// Memory Buffer error.
    #[error("Buffer error")]
    Buffer = ffi::PAM_BUF_ERR,

    /// Permission Denied.
    #[error("Permission Denied")]
    Permission = ffi::PAM_PERM_DENIED,

    /// Authentication error.
    #[error("Authentication error")]
    Authentication = ffi::PAM_AUTH_ERR,

    /// Could not access authentication data due to insufficient credentials.
    #[error("Could not access authentication data due to insufficient credentials")]
    InsufficientCreds = ffi::PAM_CRED_INSUFFICIENT,

    /// Underlying authentication service could not retrieve authentication information.
    #[error("Authentication info unavailable")]
    AuthInfoUnavailable = ffi::PAM_AUTHINFO_UNAVAIL,

    /// User not known to the underlying authentication service.
    #[error("Unknown user")]
    UnknownUser = ffi::PAM_USER_UNKNOWN,

    /// An authentication service has maintained a retry count which has been reached.
    #[error("Maximum tries reached")]
    MaxTries = ffi::PAM_MAXTRIES,

    /// New authentication token required.
    ///
    /// This is normally returned if the machine security policies require
    /// that the password should be changed because the password is NULL or it has aged.
    #[error("New auth token required")]
    NewAuthTokenRequired = ffi::PAM_NEW_AUTHTOK_REQD,

    /// User account has expired.
    #[error("User account expired")]
    AccountExpired = ffi::PAM_ACCT_EXPIRED,

    /// Can not make/remove an entry for the specified session.
    #[error("Session error")]
    Session = ffi::PAM_SESSION_ERR,

    /// Underlying authentication service could not retrieve user credentials.
    #[error("Unavailable credentials")]
    CredsUnavailable = ffi::PAM_CRED_UNAVAIL,

    /// User credentials expired.
    #[error("Credentials expired")]
    CredsExpired = ffi::PAM_CRED_EXPIRED,

    /// Failure setting user credentials.
    #[error("Credentials error")]
    Creds = ffi::PAM_CRED_ERR,

    /// No module specific data is present.
    #[error("No module data")]
    NoModuleData = ffi::PAM_NO_MODULE_DATA,

    /// Conversation error.
    #[error("Conversation error")]
    Conversation = ffi::PAM_CONV_ERR,

    /// Authentication token manipulation error.
    #[error("Auth token error")]
    AuthToken = ffi::PAM_AUTHTOK_ERR,

    /// Authentication information cannot be recovered.
    #[error("Auth token recovery error")]
    AuthTokenRecovery = ffi::PAM_AUTHTOK_RECOVERY_ERR,

    /// Authentication token lock busy.
    #[error("Auth token lock busy")]
    AuthTokenLockBusy = ffi::PAM_AUTHTOK_LOCK_BUSY,

    /// Authentication token aging disabled.
    #[error("Auth token aging disabled")]
    AuthTokenAgingDisabled = ffi::PAM_AUTHTOK_DISABLE_AGING,

    /// Preliminary check by password service.
    #[error("Preliminary check by password service.")]
    TryAgain = ffi::PAM_TRY_AGAIN,

    /// Ignore underlying account module regardless of whether
    /// the control flag is required, optional, or sufficient.
    #[error("Ignore module")]
    Ignore = ffi::PAM_IGNORE,

    /// Critical error.
    #[error("Critical error")]
    Abort = ffi::PAM_ABORT,

    /// User authentication token has expired.
    #[error("Auth token expired")]
    AuthTokenExpired = ffi::PAM_AUTHTOK_EXPIRED,

    /// Unknown module.
    #[error("Unknown module")]
    UnknownModule = ffi::PAM_MODULE_UNKNOWN,

    /// Bad item received.
    #[error("Bad item")]
    BadItem = ffi::PAM_BAD_ITEM,

    /// No data available.
    ///
    /// The conversation function is event driven. This occurs when there is
    /// no data available yet.
    #[error("No data available")]
    ConversationAgain = ffi::PAM_CONV_AGAIN,

    /// Incomplete authentication stack.
    ///
    /// Verify that the conversation is completed, and call the function again
    /// to complete the authentication stack
    #[error("Incomplete authentication stack")]
    Incomplete = ffi::PAM_INCOMPLETE,
}
