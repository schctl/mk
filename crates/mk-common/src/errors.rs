//! Error handling tools.

/// Convenient exit for when a null pointer is encountered.
#[macro_export]
macro_rules! nullptr_bail {
    () => {
        return Err(::std::io::Error::new(::std::io::ErrorKind::InvalidData, "null pointer").into())
    };
}

/// Convinient exit for when an operation is not permitted.
#[macro_export]
macro_rules! auth_bail {
    ($val:expr) => {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, $val).into())
    };
}
