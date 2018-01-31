use std::ffi::{CStr, CString};
use vapoursynth_sys as ffi;

use api::API;
use frame::Frame;
use video_info::VideoInfo;

bitflags! {
    /// Node flags.
    pub struct Flags: i32 {
        /// This flag indicates that the frames returned by the filter should not be cached. "Fast"
        /// filters should set this to reduce cache bloat.
        const NO_CACHE = ffi::VSNodeFlags_nfNoCache.0;
        /// This flag must not be used in third-party filters. It is used to mark instances of the
        /// built-in Cache filter. Strange things may happen to your filter if you use this flag.
        const IS_CACHE = ffi::VSNodeFlags_nfIsCache.0;

        /// This flag should be used by filters which prefer linear access, like source filters,
        /// where seeking around can cause significant slowdowns. This flag only has any effect if
        /// the filter using it is immediately followed by an instance of the built-in Cache
        /// filter.
        #[cfg(feature = "gte-vapoursynth-api-33")]
        const MAKE_LINEAR = ffi::VSNodeFlags_nfMakeLinear.0;
    }
}

impl From<ffi::VSNodeFlags> for Flags {
    fn from(flags: ffi::VSNodeFlags) -> Self {
        Self::from_bits_truncate(flags.0)
    }
}

/// A reference to a node in the constructed filter graph.
#[derive(Debug)]
pub struct Node {
    api: API,
    handle: *mut ffi::VSNodeRef,
}

unsafe impl Send for Node {}
unsafe impl Sync for Node {}

impl Drop for Node {
    fn drop(&mut self) {
        unsafe {
            self.api.free_node(self.handle);
        }
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        let handle = unsafe { self.api.clone_node(self.handle) };
        Self {
            api: self.api,
            handle,
        }
    }
}

impl Node {
    /// Wraps `handle` in a `Node`.
    ///
    /// # Safety
    /// The caller must ensure `handle` is valid.
    pub(crate) unsafe fn from_ptr(api: API, handle: *mut ffi::VSNodeRef) -> Self {
        Self { api, handle }
    }

    /// Returns the video info associated with this `Node`.
    pub fn info(&self) -> VideoInfo {
        unsafe {
            let ptr = self.api.get_video_info(self.handle);
            VideoInfo::from_ptr(ptr)
        }
    }

    /// Generates a frame directly.
    ///
    /// # Panics
    /// Panics is `n` is greater than `i32::max_value()`.
    pub fn get_frame(&self, n: usize) -> Result<Frame, CString> {
        assert!(n <= i32::max_value() as usize);
        let n = n as i32;

        // Kinda arbitrary. Same value as used in vsvfw.
        const ERROR_BUF_CAPACITY: usize = 32 * 1024;

        let mut err_buf = Vec::with_capacity(ERROR_BUF_CAPACITY);
        err_buf.resize(ERROR_BUF_CAPACITY, 0);
        let mut err_buf = err_buf.into_boxed_slice();

        let handle = unsafe { self.api.get_frame(n, self.handle, &mut *err_buf) };

        if handle.is_null() {
            // TODO: remove this extra allocation by reusing `Box<[c_char]>`.
            let error = unsafe { CStr::from_ptr(err_buf.as_ptr()) }.to_owned();
            Err(error)
        } else {
            Ok(unsafe { Frame::new(self.api, handle) })
        }
    }
}
