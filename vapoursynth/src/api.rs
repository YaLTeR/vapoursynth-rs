//! Most general VapourSynth API functions.

use std::ffi::{CStr, CString, NulError};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::{mem, panic, process, ptr};
use std::os::raw::{c_char, c_int, c_void};
use vapoursynth_sys as ffi;

/// A wrapper for the VapourSynth API.
#[derive(Debug, Clone, Copy)]
pub struct API {
    handle: *const ffi::VSAPI,
}

unsafe impl Send for API {}
unsafe impl Sync for API {}

/// A cached API pointer. Note that this is `*const ffi::VSAPI`, not `*mut`.
static RAW_API: AtomicPtr<ffi::VSAPI> = AtomicPtr::new(ptr::null_mut());

/// VapourSynth log message types.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum MessageType {
    Debug,
    Warning,
    Critical,

    /// The process will `abort()` after the message handler returns.
    Fatal,
}

// Macros for implementing repetitive functions.
macro_rules! prop_get_something {
    ($name:ident, $func:ident, $rv:ty) => (
        #[inline]
        pub(crate) unsafe fn $name(
            self,
            map: &ffi::VSMap,
            key: *const c_char,
            index: i32,
            error: &mut i32,
        ) -> $rv {
            ((*self.handle).$func)(map, key, index, error)
        }
    )
}

macro_rules! prop_set_something {
    ($name:ident, $func:ident, $type:ty) => (
        #[inline]
        pub(crate) unsafe fn $name(
            self,
            map: &mut ffi::VSMap,
            key: *const c_char,
            value: $type,
            append: ffi::VSPropAppendMode,
        ) -> i32 {
            ((*self.handle).$func)(map, key, value, append as i32)
        }
    )
}

impl API {
    /// Retrieves the VapourSynth API.
    ///
    /// Returns `None` on error, for example if the requested API version (selected with features,
    /// see the crate-level docs) is not supported.
    // If we're linking to VSScript anyway, use the VSScript function.
    #[cfg(all(feature = "vsscript-functions", feature = "gte-vsscript-api-32"))]
    #[inline]
    pub fn get() -> Option<Self> {
        use vsscript;

        // Check if we already have the API.
        let handle = RAW_API.load(Ordering::Relaxed);

        let handle = if handle.is_null() {
            // Attempt retrieving it otherwise.
            vsscript::maybe_initialize();
            let handle = unsafe { ffi::vsscript_getVSApi2(ffi::VAPOURSYNTH_API_VERSION) };

            if !handle.is_null() {
                // If we successfully retrieved the API, cache it.
                RAW_API.store(handle as *mut _, Ordering::Relaxed);
            }
            handle
        } else {
            handle
        };

        if handle.is_null() {
            None
        } else {
            Some(Self { handle })
        }
    }

    /// Retrieves the VapourSynth API.
    ///
    /// Returns `None` on error, for example if the requested API version (selected with features,
    /// see the crate-level docs) is not supported.
    #[cfg(all(feature = "vapoursynth-functions",
              not(all(feature = "vsscript-functions", feature = "gte-vsscript-api-32"))))]
    #[inline]
    pub fn get() -> Option<Self> {
        // Check if we already have the API.
        let handle = RAW_API.load(Ordering::Relaxed);

        let handle = if handle.is_null() {
            // Attempt retrieving it otherwise.
            let handle = unsafe { ffi::getVapourSynthAPI(ffi::VAPOURSYNTH_API_VERSION) };

            if !handle.is_null() {
                // If we successfully retrieved the API, cache it.
                RAW_API.store(handle as *mut _, Ordering::Relaxed);
            }
            handle
        } else {
            handle
        };

        if handle.is_null() {
            None
        } else {
            Some(Self { handle })
        }
    }

    /// Returns the cached API.
    ///
    /// # Safety
    /// This function assumes the cache contains a valid API pointer.
    #[inline]
    pub(crate) unsafe fn get_cached() -> Self {
        Self {
            handle: RAW_API.load(Ordering::Relaxed),
        }
    }

    /// Stores the API in the cache.
    ///
    /// # Safety
    /// The given pointer should be valid.
    #[inline]
    pub(crate) unsafe fn set(handle: *const ffi::VSAPI) {
        RAW_API.store(handle as *mut _, Ordering::Relaxed);
    }

    /// Sends a message through VapourSynth’s logging framework.
    #[cfg(feature = "gte-vapoursynth-api-34")]
    #[inline]
    pub fn log(self, message_type: MessageType, message: &str) -> Result<(), NulError> {
        let message = CString::new(message)?;
        unsafe {
            ((*self.handle).logMessage)(message_type.ffi_type(), message.as_ptr());
        }
        Ok(())
    }

    /// Installs a custom handler for the various error messages VapourSynth emits. The message
    /// handler is currently global, i.e. per process, not per VSCore instance.
    ///
    /// The default message handler simply sends the messages to the standard error stream.
    ///
    /// The callback arguments are the message type and the message itself. If the callback panics,
    /// the process is aborted.
    ///
    /// This function allocates to store the callback, this memory is leaked if the message handler
    /// is subsequently changed.
    #[inline]
    pub fn set_message_handler<F>(self, callback: F)
    where
        F: FnMut(MessageType, &CStr) + Send + 'static,
    {
        struct CallbackData {
            callback: Box<FnMut(MessageType, &CStr) + Send + 'static>,
        }

        unsafe extern "system" fn c_callback(
            msg_type: c_int,
            msg: *const c_char,
            user_data: *mut c_void,
        ) {
            let mut user_data = Box::from_raw(user_data as *mut CallbackData);

            {
                let closure = panic::AssertUnwindSafe(|| {
                    let message_type = MessageType::from_ffi_type(msg_type).unwrap();
                    let message = CStr::from_ptr(msg);

                    (user_data.callback)(message_type, message);
                });

                if panic::catch_unwind(closure).is_err() {
                    process::abort();
                }
            }

            // Don't drop user_data, we're not done using it.
            mem::forget(user_data);
        }

        let user_data = Box::new(CallbackData {
            callback: Box::new(callback),
        });

        unsafe {
            ((*self.handle).setMessageHandler)(
                Some(c_callback),
                Box::into_raw(user_data) as *mut c_void,
            );
        }
    }

    /// Installs a custom handler for the various error messages VapourSynth emits. The message
    /// handler is currently global, i.e. per process, not per VSCore instance.
    ///
    /// The default message handler simply sends the messages to the standard error stream.
    ///
    /// The callback arguments are the message type and the message itself. If the callback panics,
    /// the process is aborted.
    ///
    /// This version does not allocate at the cost of accepting a function pointer rather than an
    /// arbitrary closure. It can, however, be used with simple closures.
    #[inline]
    pub fn set_message_handler_trivial(self, callback: fn(MessageType, &CStr)) {
        unsafe extern "system" fn c_callback(
            msg_type: c_int,
            msg: *const c_char,
            user_data: *mut c_void,
        ) {
            let closure = panic::AssertUnwindSafe(|| {
                let message_type = MessageType::from_ffi_type(msg_type).unwrap();
                let message = CStr::from_ptr(msg);

                // Is there a better way of casting this?
                let callback = *(&user_data as *const _ as *const fn(MessageType, &CStr));
                (callback)(message_type, message);
            });

            if panic::catch_unwind(closure).is_err() {
                eprintln!("panic in the set_message_handler_trivial() callback, aborting");
                process::abort();
            }
        }

        unsafe {
            ((*self.handle).setMessageHandler)(Some(c_callback), callback as *mut c_void);
        }
    }

    /// Clears any custom message handler, restoring the default one.
    #[inline]
    pub fn clear_message_handler(self) {
        unsafe {
            ((*self.handle).setMessageHandler)(None, ptr::null_mut());
        }
    }

    /// Frees `node`.
    ///
    /// # Safety
    /// The caller must ensure `node` is valid.
    #[inline]
    pub(crate) unsafe fn free_node(self, node: *mut ffi::VSNodeRef) {
        ((*self.handle).freeNode)(node);
    }

    /// Clones `node`.
    ///
    /// # Safety
    /// The caller must ensure `node` is valid.
    #[inline]
    pub(crate) unsafe fn clone_node(self, node: *mut ffi::VSNodeRef) -> *mut ffi::VSNodeRef {
        ((*self.handle).cloneNodeRef)(node)
    }

    /// Returns a pointer to the video info associated with `node`. The pointer is valid as long as
    /// the node lives.
    ///
    /// # Safety
    /// The caller must ensure `node` is valid.
    #[inline]
    pub(crate) unsafe fn get_video_info(
        self,
        node: *mut ffi::VSNodeRef,
    ) -> *const ffi::VSVideoInfo {
        ((*self.handle).getVideoInfo)(node)
    }

    /// Generates a frame directly.
    ///
    /// # Safety
    /// The caller must ensure `node` is valid.
    ///
    /// # Panics
    /// Panics if `err_msg` is larger than `i32::max_value()`.
    #[inline]
    pub(crate) unsafe fn get_frame(
        self,
        n: i32,
        node: *mut ffi::VSNodeRef,
        err_msg: &mut [c_char],
    ) -> *const ffi::VSFrameRef {
        let len = err_msg.len();
        assert!(len <= i32::max_value() as usize);
        let len = len as i32;

        ((*self.handle).getFrame)(n, node, err_msg.as_mut_ptr(), len)
    }

    /// Generates a frame directly.
    ///
    /// # Safety
    /// The caller must ensure `node` and `callback` are valid.
    #[inline]
    pub(crate) unsafe fn get_frame_async(
        self,
        n: i32,
        node: *mut ffi::VSNodeRef,
        callback: ffi::VSFrameDoneCallback,
        user_data: *mut c_void,
    ) {
        ((*self.handle).getFrameAsync)(n, node, callback, user_data);
    }

    /// Frees `frame`.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid.
    #[inline]
    pub(crate) unsafe fn free_frame(self, frame: &ffi::VSFrameRef) {
        ((*self.handle).freeFrame)(frame);
    }

    /// Clones `frame`.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid.
    #[inline]
    pub(crate) unsafe fn clone_frame(self, frame: &ffi::VSFrameRef) -> *const ffi::VSFrameRef {
        ((*self.handle).cloneFrameRef)(frame)
    }

    /// Retrieves the format of a frame.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid.
    #[inline]
    pub(crate) unsafe fn get_frame_format(self, frame: &ffi::VSFrameRef) -> *const ffi::VSFormat {
        ((*self.handle).getFrameFormat)(frame)
    }

    /// Returns the width of a plane of a given frame, in pixels.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid and `plane` is valid for the given `frame`.
    #[inline]
    pub(crate) unsafe fn get_frame_width(self, frame: &ffi::VSFrameRef, plane: i32) -> i32 {
        ((*self.handle).getFrameWidth)(frame, plane)
    }

    /// Returns the height of a plane of a given frame, in pixels.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid and `plane` is valid for the given `frame`.
    #[inline]
    pub(crate) unsafe fn get_frame_height(self, frame: &ffi::VSFrameRef, plane: i32) -> i32 {
        ((*self.handle).getFrameHeight)(frame, plane)
    }

    /// Returns the distance in bytes between two consecutive lines of a plane of a frame.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid and `plane` is valid for the given `frame`.
    #[inline]
    pub(crate) unsafe fn get_frame_stride(self, frame: &ffi::VSFrameRef, plane: i32) -> i32 {
        ((*self.handle).getStride)(frame, plane)
    }

    /// Returns a read-only pointer to a plane of a frame.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid and `plane` is valid for the given `frame`.
    #[inline]
    pub(crate) unsafe fn get_frame_read_ptr(
        self,
        frame: &ffi::VSFrameRef,
        plane: i32,
    ) -> *const u8 {
        ((*self.handle).getReadPtr)(frame, plane)
    }

    /// Returns a read-write pointer to a plane of a frame.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid and `plane` is valid for the given `frame`.
    #[inline]
    pub(crate) unsafe fn get_frame_write_ptr(
        self,
        frame: &mut ffi::VSFrameRef,
        plane: i32,
    ) -> *mut u8 {
        ((*self.handle).getWritePtr)(frame, plane)
    }

    /// Returns a read-only pointer to a frame's properties.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid and the correct lifetime is assigned to the
    /// returned map (it can't outlive `frame`).
    #[inline]
    pub(crate) unsafe fn get_frame_props_ro(self, frame: &ffi::VSFrameRef) -> *const ffi::VSMap {
        ((*self.handle).getFramePropsRO)(frame)
    }

    /// Returns a read-write pointer to a frame's properties.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid and the correct lifetime is assigned to the
    /// returned map (it can't outlive `frame`).
    #[inline]
    pub(crate) unsafe fn get_frame_props_rw(self, frame: &mut ffi::VSFrameRef) -> *mut ffi::VSMap {
        ((*self.handle).getFramePropsRW)(frame)
    }

    /// Creates a new `VSMap`.
    #[inline]
    pub(crate) fn create_map(self) -> *mut ffi::VSMap {
        unsafe { ((*self.handle).createMap)() }
    }

    /// Clears `map`.
    ///
    /// # Safety
    /// The caller must ensure `map` is valid.
    #[inline]
    pub(crate) unsafe fn clear_map(self, map: &mut ffi::VSMap) {
        ((*self.handle).clearMap)(map);
    }

    /// Frees `map`.
    ///
    /// # Safety
    /// The caller must ensure `map` is valid.
    #[inline]
    pub(crate) unsafe fn free_map(self, map: &mut ffi::VSMap) {
        ((*self.handle).freeMap)(map);
    }

    /// Returns a pointer to the error message contained in the map, or NULL if there is no error
    /// message.
    ///
    /// # Safety
    /// The caller must ensure `map` is valid.
    #[inline]
    pub(crate) unsafe fn get_error(self, map: &ffi::VSMap) -> *const c_char {
        ((*self.handle).getError)(map)
    }

    /// Adds an error message to a map. The map is cleared first. The error message is copied.
    ///
    /// # Safety
    /// The caller must ensure `map` and `errorMessage` are valid.
    #[inline]
    pub(crate) unsafe fn set_error(self, map: &mut ffi::VSMap, error_message: *const c_char) {
        ((*self.handle).setError)(map, error_message)
    }

    /// Returns the number of keys contained in a map.
    ///
    /// # Safety
    /// The caller must ensure `map` is valid.
    #[inline]
    pub(crate) unsafe fn prop_num_keys(self, map: &ffi::VSMap) -> i32 {
        ((*self.handle).propNumKeys)(map)
    }

    /// Returns a key from a property map.
    ///
    /// # Safety
    /// The caller must ensure `map` is valid and `index` is valid for `map`.
    #[inline]
    pub(crate) unsafe fn prop_get_key(self, map: &ffi::VSMap, index: i32) -> *const c_char {
        ((*self.handle).propGetKey)(map, index)
    }

    /// Removes the key from a property map.
    ///
    /// # Safety
    /// The caller must ensure `map` and `key` are valid.
    #[inline]
    pub(crate) unsafe fn prop_delete_key(self, map: &mut ffi::VSMap, key: *const c_char) -> i32 {
        ((*self.handle).propDeleteKey)(map, key)
    }

    /// Returns the number of elements associated with a key in a property map.
    ///
    /// # Safety
    /// The caller must ensure `map` and `key` are valid.
    #[inline]
    pub(crate) unsafe fn prop_num_elements(self, map: &ffi::VSMap, key: *const c_char) -> i32 {
        ((*self.handle).propNumElements)(map, key)
    }

    /// Returns the type of the elements associated with the given key in a property map.
    ///
    /// # Safety
    /// The caller must ensure `map` and `key` are valid.
    #[inline]
    pub(crate) unsafe fn prop_get_type(self, map: &ffi::VSMap, key: *const c_char) -> c_char {
        ((*self.handle).propGetType)(map, key)
    }

    /// Returns the size in bytes of a property of type ptData.
    ///
    /// # Safety
    /// The caller must ensure `map` and `key` are valid.
    #[inline]
    pub(crate) unsafe fn prop_get_data_size(
        self,
        map: &ffi::VSMap,
        key: *const c_char,
        index: i32,
        error: &mut i32,
    ) -> i32 {
        ((*self.handle).propGetDataSize)(map, key, index, error)
    }

    prop_get_something!(prop_get_int, propGetInt, i64);
    prop_get_something!(prop_get_float, propGetFloat, f64);
    prop_get_something!(prop_get_data, propGetData, *const c_char);
    prop_get_something!(prop_get_node, propGetNode, *mut ffi::VSNodeRef);
    prop_get_something!(prop_get_frame, propGetFrame, *const ffi::VSFrameRef);
    prop_get_something!(prop_get_func, propGetFunc, *mut ffi::VSFuncRef);

    prop_set_something!(prop_set_int, propSetInt, i64);
    prop_set_something!(prop_set_float, propSetFloat, f64);
    prop_set_something!(prop_set_node, propSetNode, *mut ffi::VSNodeRef);
    prop_set_something!(prop_set_frame, propSetFrame, *const ffi::VSFrameRef);
    prop_set_something!(prop_set_func, propSetFunc, *mut ffi::VSFuncRef);

    /// Retrieves an array of integers from a map.
    ///
    /// # Safety
    /// The caller must ensure `map` and `key` are valid.
    #[cfg(feature = "gte-vapoursynth-api-31")]
    #[inline]
    pub(crate) unsafe fn prop_get_int_array(
        self,
        map: &ffi::VSMap,
        key: *const c_char,
        error: &mut i32,
    ) -> *const i64 {
        ((*self.handle).propGetIntArray)(map, key, error)
    }

    /// Retrieves an array of floating point numbers from a map.
    ///
    /// # Safety
    /// The caller must ensure `map` and `key` are valid.
    #[cfg(feature = "gte-vapoursynth-api-31")]
    #[inline]
    pub(crate) unsafe fn prop_get_float_array(
        self,
        map: &ffi::VSMap,
        key: *const c_char,
        error: &mut i32,
    ) -> *const f64 {
        ((*self.handle).propGetFloatArray)(map, key, error)
    }

    /// Adds a data property to the map.
    ///
    /// # Safety
    /// The caller must ensure `map` and `key` are valid.
    ///
    /// # Panics
    /// Panics if `value.len()` can't fit in an `i32`.
    #[inline]
    pub(crate) unsafe fn prop_set_data(
        self,
        map: &mut ffi::VSMap,
        key: *const c_char,
        value: &[u8],
        append: ffi::VSPropAppendMode,
    ) -> i32 {
        let length = value.len();
        assert!(length <= i32::max_value() as usize);
        let length = length as i32;

        ((*self.handle).propSetData)(map, key, value.as_ptr() as _, length, append as i32)
    }

    /// Adds an array of integers to the map.
    ///
    /// # Safety
    /// The caller must ensure `map` and `key` are valid.
    ///
    /// # Panics
    /// Panics if `value.len()` can't fit in an `i32`.
    #[cfg(feature = "gte-vapoursynth-api-31")]
    #[inline]
    pub(crate) unsafe fn prop_set_int_array(
        self,
        map: &mut ffi::VSMap,
        key: *const c_char,
        value: &[i64],
    ) -> i32 {
        let length = value.len();
        assert!(length <= i32::max_value() as usize);
        let length = length as i32;

        ((*self.handle).propSetIntArray)(map, key, value.as_ptr(), length)
    }

    /// Adds an array of floating point numbers to the map.
    ///
    /// # Safety
    /// The caller must ensure `map` and `key` are valid.
    ///
    /// # Panics
    /// Panics if `value.len()` can't fit in an `i32`.
    #[cfg(feature = "gte-vapoursynth-api-31")]
    #[inline]
    pub(crate) unsafe fn prop_set_float_array(
        self,
        map: &mut ffi::VSMap,
        key: *const c_char,
        value: &[f64],
    ) -> i32 {
        let length = value.len();
        assert!(length <= i32::max_value() as usize);
        let length = length as i32;

        ((*self.handle).propSetFloatArray)(map, key, value.as_ptr(), length)
    }

    /// Frees `function`.
    ///
    /// # Safety
    /// The caller must ensure `function` is valid.
    #[inline]
    pub(crate) unsafe fn free_func(self, function: *mut ffi::VSFuncRef) {
        ((*self.handle).freeFunc)(function);
    }

    /// Clones `function`.
    ///
    /// # Safety
    /// The caller must ensure `function` is valid.
    #[inline]
    pub(crate) unsafe fn clone_func(self, function: *mut ffi::VSFuncRef) -> *mut ffi::VSFuncRef {
        ((*self.handle).cloneFuncRef)(function)
    }

    /// Returns information about the VapourSynth core.
    ///
    /// # Safety
    /// The caller must ensure `core` is valid.
    #[inline]
    pub(crate) unsafe fn get_core_info(self, core: *mut ffi::VSCore) -> *const ffi::VSCoreInfo {
        ((*self.handle).getCoreInfo)(core)
    }

    /// Returns a VSFormat structure from a video format identifier.
    ///
    /// # Safety
    /// The caller must ensure `core` is valid.
    #[inline]
    pub(crate) unsafe fn get_format_preset(
        self,
        id: i32,
        core: *mut ffi::VSCore,
    ) -> *const ffi::VSFormat {
        ((*self.handle).getFormatPreset)(id, core)
    }

    /// Registers a custom video format.
    ///
    /// # Safety
    /// The caller must ensure `core` is valid.
    #[inline]
    pub(crate) unsafe fn register_format(
        self,
        color_family: ffi::VSColorFamily,
        sample_type: ffi::VSSampleType,
        bits_per_sample: i32,
        sub_sampling_w: i32,
        sub_sampling_h: i32,
        core: *mut ffi::VSCore,
    ) -> *const ffi::VSFormat {
        ((*self.handle).registerFormat)(
            color_family as i32,
            sample_type as i32,
            bits_per_sample,
            sub_sampling_w,
            sub_sampling_h,
            core,
        )
    }

    /// Creates a new filter node.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid.
    #[inline]
    pub(crate) unsafe fn create_filter(
        self,
        in_: *const ffi::VSMap,
        out: *mut ffi::VSMap,
        name: *const c_char,
        init: ffi::VSFilterInit,
        get_frame: ffi::VSFilterGetFrame,
        free: ffi::VSFilterFree,
        filter_mode: ffi::VSFilterMode,
        flags: ffi::VSNodeFlags,
        instance_data: *mut c_void,
        core: *mut ffi::VSCore,
    ) {
        ((*self.handle).createFilter)(
            in_,
            out,
            name,
            init,
            get_frame,
            free,
            filter_mode as _,
            flags.0,
            instance_data,
            core,
        );
    }

    /// Sets node's video info.
    ///
    /// # Safety
    /// The caller must ensure `node` is valid.
    ///
    /// # Panics
    /// Panics if `vi.len()` can't fit in an `i32`.
    #[inline]
    pub(crate) unsafe fn set_video_info(self, vi: &[ffi::VSVideoInfo], node: *mut ffi::VSNode) {
        let length = vi.len();
        assert!(length <= i32::max_value() as usize);
        let length = length as i32;

        ((*self.handle).setVideoInfo)(vi.as_ptr(), length, node);
    }

    /// Adds an error message to a frame context, replacing the existing message, if any.
    ///
    /// This is the way to report errors in a filter's "get frame" function. Such errors are not
    /// necessarily fatal, i.e. the caller can try to request the same frame again.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid.
    #[inline]
    pub(crate) unsafe fn set_filter_error(
        self,
        message: *const c_char,
        frame_ctx: *mut ffi::VSFrameContext,
    ) {
        ((*self.handle).setFilterError)(message, frame_ctx);
    }

    /// Requests a frame from a node and returns immediately.
    ///
    /// This is only used in filters' "get frame" functions.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid and this is called from a filter "get frame"
    /// function.
    #[inline]
    pub(crate) unsafe fn request_frame_filter(
        self,
        n: i32,
        node: *mut ffi::VSNodeRef,
        frame_ctx: *mut ffi::VSFrameContext,
    ) {
        ((*self.handle).requestFrameFilter)(n, node, frame_ctx);
    }

    /// Retrieves a frame that was previously requested with `request_frame_filter()`.
    ///
    /// This is only used in filters' "get frame" functions.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid and this is called from a filter "get frame"
    /// function.
    #[inline]
    pub(crate) unsafe fn get_frame_filter(
        self,
        n: i32,
        node: *mut ffi::VSNodeRef,
        frame_ctx: *mut ffi::VSFrameContext,
    ) -> *const ffi::VSFrameRef {
        ((*self.handle).getFrameFilter)(n, node, frame_ctx)
    }

    /// Duplicates the frame (not just the reference). As the frame buffer is shared in a
    /// copy-on-write fashion, the frame content is not really duplicated until a write operation
    /// occurs. This is transparent for the user.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid.
    #[inline]
    pub(crate) unsafe fn copy_frame(
        self,
        f: &ffi::VSFrameRef,
        core: *mut ffi::VSCore,
    ) -> *mut ffi::VSFrameRef {
        ((*self.handle).copyFrame)(f, core)
    }

    /// Creates a new frame, optionally copying the properties attached to another frame. The new
    /// frame contains uninitialised memory.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid and that the uninitialized plane data of the
    /// returned frame is handled carefully.
    #[inline]
    pub(crate) unsafe fn new_video_frame(
        self,
        format: &ffi::VSFormat,
        width: i32,
        height: i32,
        prop_src: *const ffi::VSFrameRef,
        core: *mut ffi::VSCore,
    ) -> *mut ffi::VSFrameRef {
        ((*self.handle).newVideoFrame)(format, width, height, prop_src, core)
    }

    /// Returns a pointer to the plugin with the given identifier, or a null pointer if not found.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid.
    #[inline]
    pub(crate) unsafe fn get_plugin_by_id(
        self,
        identifier: *const c_char,
        core: *mut ffi::VSCore,
    ) -> *mut ffi::VSPlugin {
        ((*self.handle).getPluginById)(identifier, core)
    }

    /// Returns a pointer to the plugin with the given namespace, or a null pointer if not found.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid.
    #[inline]
    pub(crate) unsafe fn get_plugin_by_ns(
        self,
        namespace: *const c_char,
        core: *mut ffi::VSCore,
    ) -> *mut ffi::VSPlugin {
        ((*self.handle).getPluginByNs)(namespace, core)
    }

    /// Returns a map containing a list of all loaded plugins.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid.
    #[inline]
    pub(crate) unsafe fn get_plugins(self, core: *mut ffi::VSCore) -> *mut ffi::VSMap {
        ((*self.handle).getPlugins)(core)
    }

    /// Returns a map containing a list of the filters exported by a plugin.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid.
    #[inline]
    pub(crate) unsafe fn get_functions(self, plugin: *mut ffi::VSPlugin) -> *mut ffi::VSMap {
        ((*self.handle).getFunctions)(plugin)
    }

    /// Returns the absolute path to the plugin, including the plugin's file name. This is the real
    /// location of the plugin, i.e. there are no symbolic links in the path.
    ///
    /// Path elements are always delimited with forward slashes.
    ///
    /// VapourSynth retains ownership of the returned pointer.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid.
    // This was introduced in R25 without bumping the API version (R3) but we must be sure it's
    // there, so require R3.1.
    #[cfg(feature = "gte-vapoursynth-api-31")]
    #[inline]
    pub(crate) unsafe fn get_plugin_path(self, plugin: *mut ffi::VSPlugin) -> *const c_char {
        ((*self.handle).getPluginPath)(plugin)
    }

    /// Invokes a filter.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid.
    #[inline]
    pub(crate) unsafe fn invoke(
        self,
        plugin: *mut ffi::VSPlugin,
        name: *const c_char,
        args: *const ffi::VSMap,
    ) -> *mut ffi::VSMap {
        ((*self.handle).invoke)(plugin, name, args)
    }

    /// Returns the index of the node from which the frame is being requested.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid.
    #[inline]
    pub(crate) unsafe fn get_output_index(self, frame_ctx: *mut ffi::VSFrameContext) -> i32 {
        ((*self.handle).getOutputIndex)(frame_ctx)
    }

    /// Creates a user-defined function.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid.
    #[inline]
    pub(crate) unsafe fn create_func(
        self,
        func: ffi::VSPublicFunction,
        user_data: *mut c_void,
        free: ffi::VSFreeFuncData,
        core: *mut ffi::VSCore,
    ) -> *mut ffi::VSFuncRef {
        ((*self.handle).createFunc)(func, user_data, free, core, self.handle)
    }

    /// Calls a function. If the call fails out will have an error set.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid.
    #[inline]
    pub(crate) unsafe fn call_func(
        self,
        func: *mut ffi::VSFuncRef,
        in_: *const ffi::VSMap,
        out: *mut ffi::VSMap,
    ) {
        ((*self.handle).callFunc)(func, in_, out, ptr::null_mut(), ptr::null());
    }

    /// Registers a filter exported by the plugin. A plugin can export any number of filters.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid.
    #[inline]
    pub(crate) unsafe fn register_function(
        self,
        name: *const c_char,
        args: *const c_char,
        args_func: ffi::VSPublicFunction,
        function_data: *mut c_void,
        plugin: *mut ffi::VSPlugin,
    ) {
        ((*self.handle).registerFunction)(name, args, args_func, function_data, plugin);
    }
}

impl MessageType {
    #[inline]
    fn ffi_type(self) -> c_int {
        let rv = match self {
            MessageType::Debug => ffi::VSMessageType::mtDebug,
            MessageType::Warning => ffi::VSMessageType::mtWarning,
            MessageType::Critical => ffi::VSMessageType::mtCritical,
            MessageType::Fatal => ffi::VSMessageType::mtFatal,
        };
        rv as c_int
    }

    #[inline]
    fn from_ffi_type(x: c_int) -> Option<Self> {
        match x {
            x if x == ffi::VSMessageType::mtDebug as c_int => Some(MessageType::Debug),
            x if x == ffi::VSMessageType::mtWarning as c_int => Some(MessageType::Warning),
            x if x == ffi::VSMessageType::mtCritical as c_int => Some(MessageType::Critical),
            x if x == ffi::VSMessageType::mtFatal as c_int => Some(MessageType::Fatal),
            _ => None,
        }
    }
}
