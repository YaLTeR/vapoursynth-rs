use std::ffi::{CString, NulError};
use std::{fmt, io, result};

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
    #[fail(display = "An error occurred in VSScript")]
    VSScript(#[cause] VSScriptError),
    #[fail(display = "There's no such variable")]
    NoSuchVariable,
    #[fail(display = "Couldn't get the core")]
    NoCore,
    #[fail(display = "There's no output on the requested index")]
    NoOutput,
    #[fail(display = "Couldn't get the VapourSynth API")]
    NoAPI,
}

impl From<NulError> for Error {
    #[inline]
    fn from(x: NulError) -> Self {
        Error::CStringConversion(x)
    }
}

impl From<VSScriptError> for Error {
    #[inline]
    fn from(x: VSScriptError) -> Self {
        Error::VSScript(x)
    }
}

pub(crate) type Result<T> = result::Result<T, Error>;

/// A container for a VSScript error.
#[derive(Fail, Debug)]
pub struct VSScriptError(CString);

impl fmt::Display for VSScriptError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.to_string_lossy())
    }
}

impl VSScriptError {
    /// Creates a new `VSScriptError` with the given error message.
    #[inline]
    pub(crate) fn new(message: CString) -> Self {
        VSScriptError(message)
    }
}
