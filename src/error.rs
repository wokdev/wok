use std::error;
use std::fmt;

/// An [Error](std::error::Error) implementation having its Debug output
/// generated from a source Error's Display output.
///
/// Useful for a friendly Error representation to a user in a terminal session.
///
/// # Examples
///
/// ```
/// use wok;
/// fn could_be_main() -> Result<(), wok::Error> {
///     let some_error = std::io::Error::from(std::io::ErrorKind::NotFound);
///     Err(wok::Error::from(&some_error))
/// }
/// assert_eq!(format!("{:?}", could_be_main().unwrap_err()), "entity not found")
/// ```
pub struct Error {
    msg: String,
}

impl error::Error for Error {}

impl<T> From<&T> for Error
where
    T: error::Error,
{
    /// Generates and stores the error message from `T`'s Display output.
    fn from(e: &T) -> Self {
        Error {
            msg: format!("{:}", e),
        }
    }
}

impl fmt::Display for Error {
    /// Uses the error message generated as Display output.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl fmt::Debug for Error {
    /// Uses Display output as Debug output.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:}", self)
    }
}
