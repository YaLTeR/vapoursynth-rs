use std::ffi::NulError;
use std::result;

/// The error type for `Map` operations.
#[derive(Fail, Debug, Eq, PartialEq)]
pub enum Error {
    #[fail(display = "The requested key wasn't found in the map")]
    KeyNotFound,
    #[fail(display = "The requested index was out of bounds")]
    IndexOutOfBounds,
    #[fail(display = "The given/requested value type doesn't match the type of the property")]
    WrongValueType,
    #[fail(display = "The key is invalid")]
    InvalidKey(#[cause] InvalidKeyError),
    #[fail(display = "Couldn't convert to a CString")]
    CStringConversion(#[cause] NulError),
}

/// A specialized `Result` type for `Map` operations.
pub type Result<T> = result::Result<T, Error>;

/// An error indicating the map key is invalid.
#[derive(Fail, Debug, Eq, PartialEq)]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub enum InvalidKeyError {
    #[fail(display = "The key is empty")]
    EmptyKey,
    #[fail(display = "The key contains an invalid character at index {}", _0)]
    InvalidCharacter(usize),
}

impl From<InvalidKeyError> for Error {
    #[inline]
    fn from(x: InvalidKeyError) -> Self {
        Error::InvalidKey(x)
    }
}

impl From<NulError> for Error {
    #[inline]
    fn from(x: NulError) -> Self {
        Error::CStringConversion(x)
    }
}
