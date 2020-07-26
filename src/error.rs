use std::error;
use std::fmt;

pub struct Error {
    msg: String,
}

impl error::Error for Error {}

impl<T> From<&T> for Error
where
    T: error::Error + fmt::Display,
{
    fn from(e: &T) -> Self {
        Error {
            msg: format!("{:}", e),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:}", self)
    }
}
