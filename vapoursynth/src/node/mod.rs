//! VapourSynth nodes.

use std::borrow::Cow;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::os::raw::{c_char, c_void};
use std::process;
use std::ptr::NonNull;
use std::{mem, panic};
use vapoursynth_sys as ffi;

use api::API;
use frame::FrameRef;
use plugins::FrameContext;
use video_info::VideoInfo;

mod errors;
pub use self::errors::GetFrameError;

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
    #[inline]
    fn from(flags: ffi::VSNodeFlags) -> Self {
        Self::from_bits_truncate(flags.0)
    }
}

/// A reference to a node in the constructed filter graph.
#[derive(Debug)]
pub struct Node<'core> {
    handle: NonNull<ffi::VSNodeRef>,
    _owner: PhantomData<&'core ()>,
}

unsafe impl<'core> Send for Node<'core> {}
unsafe impl<'core> Sync for Node<'core> {}

impl<'core> Drop for Node<'core> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            API::get_cached().free_node(self.handle.as_ptr());
        }
    }
}

impl<'core> Clone for Node<'core> {
    #[inline]
    fn clone(&self) -> Self {
        let handle = unsafe { API::get_cached().clone_node(self.handle.as_ptr()) };
        Self {
            handle: unsafe { NonNull::new_unchecked(handle) },
            _owner: PhantomData,
        }
    }
}

impl<'core> Node<'core> {
    /// Wraps `handle` in a `Node`.
    ///
    /// # Safety
    /// The caller must ensure `handle` and the lifetime is valid and API is cached.
    #[inline]
    pub(crate) unsafe fn from_ptr(handle: *mut ffi::VSNodeRef) -> Self {
        Self {
            handle: NonNull::new_unchecked(handle),
            _owner: PhantomData,
        }
    }

    /// Returns the underlying pointer.
    #[inline]
    pub(crate) fn ptr(&self) -> *mut ffi::VSNodeRef {
        self.handle.as_ptr()
    }

    /// Returns the video info associated with this `Node`.
    // Since we don't store the pointer to the actual `ffi::VSVideoInfo` and the lifetime is that
    // of the `ffi::VSFormat`, this returns `VideoInfo<'core>` rather than `VideoInfo<'a>`.
    #[inline]
    pub fn info(&self) -> VideoInfo<'core> {
        unsafe {
            let ptr = API::get_cached().get_video_info(self.handle.as_ptr());
            VideoInfo::from_ptr(ptr)
        }
    }

    /// Generates a frame directly.
    ///
    /// The `'error` lifetime is unbounded because this function always returns owned data.
    ///
    /// # Panics
    /// Panics is `n` is greater than `i32::max_value()`.
    pub fn get_frame<'error>(&self, n: usize) -> Result<FrameRef<'core>, GetFrameError<'error>> {
        assert!(n <= i32::max_value() as usize);
        let n = n as i32;

        // Kinda arbitrary. Same value as used in vsvfw.
        const ERROR_BUF_CAPACITY: usize = 32 * 1024;

        let mut err_buf = Vec::with_capacity(ERROR_BUF_CAPACITY);
        err_buf.resize(ERROR_BUF_CAPACITY, 0);
        let mut err_buf = err_buf.into_boxed_slice();

        let handle = unsafe { API::get_cached().get_frame(n, self.handle.as_ptr(), &mut *err_buf) };

        if handle.is_null() {
            // TODO: remove this extra allocation by reusing `Box<[c_char]>`.
            let error = unsafe { CStr::from_ptr(err_buf.as_ptr()) }.to_owned();
            Err(GetFrameError::new(Cow::Owned(error)))
        } else {
            Ok(unsafe { FrameRef::from_ptr(handle) })
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
    /// - the node that generated the frame (the same as `self`).
    ///
    /// If the callback panics, the process is aborted.
    ///
    /// # Panics
    /// Panics is `n` is greater than `i32::max_value()`.
    pub fn get_frame_async<F>(&self, n: usize, callback: F)
    where
        F: FnOnce(Result<FrameRef<'core>, GetFrameError>, usize, Node<'core>) + Send + 'core,
    {
        struct CallbackData<'core> {
            callback: Box<CallbackFn<'core> + 'core>,
        }

        // A little bit of magic for Box<FnOnce>.
        trait CallbackFn<'core> {
            fn call(
                self: Box<Self>,
                frame: Result<FrameRef<'core>, GetFrameError>,
                n: usize,
                node: Node<'core>,
            );
        }

        impl<'core, F> CallbackFn<'core> for F
        where
            F: FnOnce(Result<FrameRef<'core>, GetFrameError>, usize, Node<'core>),
        {
            #[cfg_attr(feature = "cargo-clippy", allow(boxed_local))]
            fn call(
                self: Box<Self>,
                frame: Result<FrameRef<'core>, GetFrameError>,
                n: usize,
                node: Node<'core>,
            ) {
                (self)(frame, n, node)
            }
        }

        unsafe extern "system" fn c_callback(
            user_data: *mut c_void,
            frame: *const ffi::VSFrameRef,
            n: i32,
            node: *mut ffi::VSNodeRef,
            error_msg: *const c_char,
        ) {
            // The actual lifetime isn't 'static, it's 'core, but we don't really have a way of
            // retrieving it.
            let user_data = Box::from_raw(user_data as *mut CallbackData<'static>);

            let closure = panic::AssertUnwindSafe(move || {
                let frame = if frame.is_null() {
                    debug_assert!(!error_msg.is_null());
                    let error_msg = Cow::Borrowed(CStr::from_ptr(error_msg));
                    Err(GetFrameError::new(error_msg))
                } else {
                    debug_assert!(error_msg.is_null());
                    Ok(FrameRef::from_ptr(frame))
                };

                let node = Node::from_ptr(node);

                debug_assert!(n >= 0);
                let n = n as usize;

                user_data.callback.call(frame, n, node);
            });

            if panic::catch_unwind(closure).is_err() {
                process::abort();
            }
        }

        assert!(n <= i32::max_value() as usize);
        let n = n as i32;

        let user_data = Box::new(CallbackData {
            callback: Box::new(callback),
        });

        let new_node = self.clone();

        unsafe {
            API::get_cached().get_frame_async(
                n,
                new_node.handle.as_ptr(),
                Some(c_callback),
                Box::into_raw(user_data) as *mut c_void,
            );
        }

        // It'll be dropped by the callback.
        mem::forget(new_node);
    }

    /// Requests a frame from a node and returns immediately.
    ///
    /// This is only used in filters' "get frame" functions.
    ///
    /// A filter usually calls this function from `get_frame_initial()`. The requested frame can
    /// then be retrieved using `get_frame_filter()` from within filter's `get_frame()` function.
    ///
    /// It is safe to request a frame more than once. An unimportant consequence of requesting a
    /// frame more than once is that the filter's `get_frame()` function may be called more than
    /// once for the same frame.
    ///
    /// It is best to request frames in ascending order, i.e. `n`, `n+1`, `n+2`, etc.
    ///
    /// # Panics
    /// Panics is `n` is greater than `i32::max_value()`.
    pub fn request_frame_filter(&self, context: FrameContext, n: usize) {
        assert!(n <= i32::max_value() as usize);
        let n = n as i32;

        unsafe {
            API::get_cached().request_frame_filter(n, self.ptr(), context.ptr());
        }
    }

    /// Retrieves a frame that was previously requested with `request_frame_filter()`.
    ///
    /// A filter usually calls this function from `get_frame()`. It is safe to retrieve a frame
    /// more than once.
    ///
    /// # Panics
    /// Panics is `n` is greater than `i32::max_value()`.
    pub fn get_frame_filter(&self, context: FrameContext, n: usize) -> Option<FrameRef<'core>> {
        assert!(n <= i32::max_value() as usize);
        let n = n as i32;

        let ptr = unsafe { API::get_cached().get_frame_filter(n, self.ptr(), context.ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { FrameRef::from_ptr(ptr) })
        }
    }
}
