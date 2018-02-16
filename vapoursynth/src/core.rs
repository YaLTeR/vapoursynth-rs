use vapoursynth_sys as ffi;

use api::API;

pub struct CoreRef {
    api: API,
    handle: *mut ffi::VSCore,
}

impl CoreRef {
    /// Wraps `handle` in a `CoreRef`.
    ///
    /// # Safety
    /// The caller must ensure `handle` is valid.
    #[inline]
    pub(crate) unsafe fn from_ptr(api: API, handle: *mut ffi::VSCore) -> Self {
        Self { api, handle }
    }
}
