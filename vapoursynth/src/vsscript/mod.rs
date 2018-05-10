//! VapourSynth script-related things.

#[cfg(not(feature = "gte-vsscript-api-32"))]
use std::sync::Mutex;
use std::sync::{Once, ONCE_INIT};
use vapoursynth_sys as ffi;

#[cfg(not(feature = "gte-vsscript-api-32"))]
lazy_static! {
    static ref FFI_CALL_MUTEX: Mutex<()> = Mutex::new(());
}

// Some `vsscript_*` function calls have threading issues. Protect them with a mutex.
// https://github.com/vapoursynth/vapoursynth/issues/367
macro_rules! call_vsscript {
    ($call:expr) => {{
        // Fixed in VSScript API 3.2.
        // TODO: also not needed when we're running API 3.2 even without a feature.
        #[cfg(not(feature = "gte-vsscript-api-32"))]
        let _lock = FFI_CALL_MUTEX.lock();

        $call
    }};
}

/// Ensures `vsscript_init()` has been called at least once.
// TODO: `vsscript_init()` is already thread-safe with `std::call_once()`, maybe this can be done
// differently to remove the thread protection on Rust's side? An idea is to have a special type
// which calls `vsscript_init()` in `new()` and `vsscript_finalize()` in `drop()` and have the rest
// of the API accessible through that type, however that could become somewhat unergonomic with
// having to store its lifetime everywhere and potentially pass it around the threads.
#[inline]
pub(crate) fn maybe_initialize() {
    static ONCE: Once = ONCE_INIT;

    ONCE.call_once(|| unsafe {
        ffi::vsscript_init();

        // Verify the VSScript API version.
        #[cfg(feature = "gte-vsscript-api-31")]
        {
            fn split_version(version: i32) -> (i32, i32) {
                (version >> 16, version & 0xFFFF)
            }

            let vsscript_version = ffi::vsscript_getApiVersion();
            let (major, minor) = split_version(vsscript_version);
            let (my_major, my_minor) = split_version(ffi::VSSCRIPT_API_VERSION);

            if my_major != major {
                panic!(
                    "Invalid VSScript major API version (expected: {}, got: {})",
                    my_major, major
                );
            } else if my_minor > minor {
                panic!(
                    "Invalid VSScript minor API version (expected: >= {}, got: {})",
                    my_minor, minor
                );
            }
        }
    });
}

mod errors;
pub use self::errors::{Error, VSScriptError};

mod environment;
pub use self::environment::{Environment, EvalFlags};
