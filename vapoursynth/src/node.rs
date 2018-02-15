use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::{mem, panic};
use std::os::raw::{c_char, c_void};
use std::process;
use vapoursynth_sys as ffi;

use api::API;
use boxfnonce::NodeFnBox;
use frame::Frame;
use video_info::VideoInfo;

bitflags! {
    /// Node flags.
    pub struct Flags: i32 {
        /// This flag indicates that the frames returned by the filter should not be cached. "Fast"
        /// filters should set this to reduce cache bloat.
    #[inline]
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
    #[inline]
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
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.api.free_node(self.handle);
        }
    }
}

impl Clone for Node {
    #[inline]
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
    #[inline]
    pub(crate) unsafe fn from_ptr(api: API, handle: *mut ffi::VSNodeRef) -> Self {
        Self { api, handle }
    }

    /// Returns the underlying pointer.
    #[inline]
    pub(crate) fn ptr(&self) -> *mut ffi::VSNodeRef {
        self.handle
    }

    /// Returns the video info associated with this `Node`.
    #[inline]
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
            Ok(unsafe { Frame::from_ptr(self.api, handle) })
        }
    }

    /// Requests the generation of a frame. When the frame is ready, a user-provided function is
    /// called.
    ///
    /// If multiple frames were requested, they can be returned in any order.
    ///
    /// The callback arguments are:
    ///
    /// - the generated frame or an error message if the generation failed,
    /// - the frame number (equal to `n`),
    /// - the node that generated the frame (the same as `self`),
    ///
    /// If the callback panics, the process is aborted.
    ///
    /// # Panics
    /// Panics is `n` is greater than `i32::max_value()`.
    pub fn get_frame_async<F>(&self, n: usize, callback: F)
    where
        F: FnOnce(Result<Frame, Cow<str>>, usize, Node) + Send + 'static,
    {
        struct FrameDoneCallbackData {
            api: API,
            callback: Box<NodeFnBox>,
        }

        unsafe extern "C" fn frame_done_callback(
            user_data: *mut c_void,
            frame: *const ffi::VSFrameRef,
            n: i32,
            node: *mut ffi::VSNodeRef,
            error_msg: *const c_char,
        ) {
            let user_data = Box::from_raw(user_data as *mut FrameDoneCallbackData);

            let closure = panic::AssertUnwindSafe(move || {
                let frame = if frame.is_null() {
                    debug_assert!(!error_msg.is_null());
                    Err(CStr::from_ptr(error_msg).to_string_lossy())
                } else {
                    debug_assert!(error_msg.is_null());
                    Ok(Frame::from_ptr(user_data.api, frame))
                };

                let node = Node::from_ptr(user_data.api, node);

                debug_assert!(n >= 0);
                let n = n as usize;

                user_data.callback.call(frame, n, node);
            });

            if panic::catch_unwind(closure).is_err() {
                eprintln!("panic in the get_frame_async() callback, aborting");
                process::abort();
            }
        }

        assert!(n <= i32::max_value() as usize);
        let n = n as i32;

        let user_data = Box::new(FrameDoneCallbackData {
            api: self.api,
            callback: Box::new(callback),
        });

        let new_node = self.clone();

        unsafe {
            self.api.get_frame_async(
                n,
                new_node.handle,
                Some(frame_done_callback),
                Box::into_raw(user_data) as *mut c_void,
            );
        }

        // It'll be dropped by the callback.
        mem::forget(new_node);
    }
}
