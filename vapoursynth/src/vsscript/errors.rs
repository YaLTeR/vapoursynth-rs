use std::ffi::NulError;
use std::{fmt, io, result};

use vsscript::*;

/// The error type for `vsscript` operations.
#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Couldn't convert to a CString")]
    CStringConversion(#[cause] NulError),
    #[fail(display = "Couldn't open the file")]
    FileOpen(#[cause] io::Error),
    #[fail(display = "Couldn't read the file")]
    FileRead(#[cause] io::Error),
    #[fail(display = "Path isn't valid Unicode")]
    PathInvalidUnicode,
    #[fail(display = "An error occurred in vsscript")]
    VSScript(#[cause] VSScriptError),
}

impl From<NulError> for Error {
    fn from(x: NulError) -> Self {
        Error::CStringConversion(x)
    }
}

impl From<VSScriptError> for Error {
    fn from(x: VSScriptError) -> Self {
        Error::VSScript(x)
    }
}

pub type Result<T> = result::Result<T, Error>;

/// A container for a VSScript error.
#[derive(Fail, Debug)]
pub struct VSScriptError(Environment);

impl fmt::Display for VSScriptError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = unsafe { self.0.get_error() };
        write!(f, "{}", message.to_string_lossy())
    }
}

impl VSScriptError {
    /// Wraps `environment` into a `VSScriptError`.
    ///
    /// # Safety
    /// The caller must ensure `environment` has an error.
    pub(crate) unsafe fn from_environment(environment: Environment) -> Self {
        VSScriptError(environment)
    }
}
