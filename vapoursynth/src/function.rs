use vapoursynth_sys as ffi;

use api::API;

/// Holds a reference to a function that may be called.
#[derive(Debug)]
pub struct Function {
    api: API,
    handle: *mut ffi::VSFuncRef,
}

unsafe impl Send for Function {}
unsafe impl Sync for Function {}

impl Drop for Function {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.api.free_func(self.handle);
        }
    }
}

impl Clone for Function {
    #[inline]
    fn clone(&self) -> Self {
        let handle = unsafe { self.api.clone_func(self.handle) };
        Self {
            api: self.api,
            handle,
        }
    }
}

impl Function {
    /// Wraps `handle` in a `Function`.
    ///
    /// # Safety
    /// The caller must ensure `handle` is valid.
    #[inline]
    pub(crate) unsafe fn from_ptr(api: API, handle: *mut ffi::VSFuncRef) -> Self {
        Self { api, handle }
    }
}
