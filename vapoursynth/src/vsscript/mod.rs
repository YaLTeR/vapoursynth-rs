use std::sync::{Mutex, Once, ONCE_INIT};
use vapoursynth_sys as ffi;

lazy_static! {
    static ref FFI_CALL_MUTEX: Mutex<()> = Mutex::new(());
}

// Some `vsscript_*` function calls have threading issues. Protect them with a mutex.
// https://github.com/vapoursynth/vapoursynth/issues/367
macro_rules! call_vsscript {
    ($call:expr) => ({
        let _lock = FFI_CALL_MUTEX.lock();
        $call
    })
}

/// Ensures `vsscript_init()` has been called at least once.
// TODO: `vsscript_init()` is already thread-safe with `std::call_once()`, maybe this can be done
// differently to remove the thread protection on Rust's side? An idea is to have a special type
// which calls `vsscript_init()` in `new()` and `vsscript_finalize()` in `drop()` and have the rest
// of the API accessible through that type, however that could become somewhat unergonomic with
// having to store its lifetime everywhere and potentially pass it around the threads.
#[inline]
fn maybe_initialize() {
    static ONCE: Once = ONCE_INIT;

    ONCE.call_once(|| unsafe {
        ffi::vsscript_init();
    });
}

pub mod errors;
pub use self::errors::{Error, VSScriptError};

pub mod environment;
pub use self::environment::*;
