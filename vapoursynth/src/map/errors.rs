use std::result;

/// The error type for `Map` operations.
#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "The requested key wasn't found in the map")]
    KeyNotFound,
    #[fail(display = "The requested index was out of bounds")]
    IndexOutOfBounds,
}

pub(crate) type Result<T> = result::Result<T, Error>;
