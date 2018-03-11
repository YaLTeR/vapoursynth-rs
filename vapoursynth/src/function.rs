//! VapourSynth callable functions.

use vapoursynth_sys as ffi;

use api::API;

/// Holds a reference to a function that may be called.
#[derive(Debug)]
pub struct Function {
    handle: *mut ffi::VSFuncRef,
}

unsafe impl Send for Function {}
unsafe impl Sync for Function {}

impl Drop for Function {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            API::get_cached().free_func(self.handle);
        }
    }
}

impl Clone for Function {
    #[inline]
    fn clone(&self) -> Self {
        let handle = unsafe { API::get_cached().clone_func(self.handle) };
        Self { handle }
    }
}

impl Function {
    /// Wraps `handle` in a `Function`.
    ///
    /// # Safety
    /// The caller must ensure `handle` is valid and API is cached.
    #[inline]
    pub(crate) unsafe fn from_ptr(handle: *mut ffi::VSFuncRef) -> Self {
        Self { handle }
    }

    /// Returns the underlying pointer.
    #[inline]
    pub(crate) fn ptr(&self) -> *mut ffi::VSFuncRef {
        self.handle
    }
}
