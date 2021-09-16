//! Authenticator configurations.

/// Paths for default field values.
mod default {
    use super::Rules;
    use super::AuthService;

    pub const fn auth_ty() -> AuthService {
        #[cfg(feature = "pam")]
        return AuthService::Pam;

        #[cfg(not(feature = "pam"))]
        return AuthService::Pwd;
    }

    pub const fn auth_rules() -> Rules {
        Rules {}
    }
}

/// All supported authentication services.
#[allow(unused)]
#[non_exhaustive]
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Copy)]
pub enum AuthService {
    /// Authentication using PAM.
    #[cfg(feature = "pam")]
    Pam,
    /// Authentication using the system password database.
    Pwd,
}

/// Predefined rules for a user session.
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Rules {
    // /// Authentication timeout.
    // timeout: u64,
}

impl Rules {
}

/// Information used to create an authenticator.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AuthConfig {
    /// Type of the underlying authentication service to use.
    #[serde(rename = "type")]
    #[serde(default = "default::auth_ty")]
    pub ty: AuthService,
    /// Authenticator rules.
    #[serde(default = "default::auth_rules")]
    pub rules: Rules,
}
