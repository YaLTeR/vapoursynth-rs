use std::os::raw::c_char;
use vapoursynth_sys as ffi;

use vsscript;

/// A wrapper for the VapourSynth API.
#[derive(Debug, Clone, Copy)]
pub struct API {
    handle: *const ffi::VSAPI,
}

unsafe impl Send for API {}
unsafe impl Sync for API {}

// Macro for implementing repetitive functions.
macro_rules! prop_get_something {
    ($name:ident, $func:ident, $rv:ty) => (
        #[inline]
        pub(crate) unsafe fn $name(
            self,
            map: *const ffi::VSMap,
            key: *const c_char,
            index: i32,
            error: &mut i32,
        ) -> $rv {
            ((*self.handle).$func)(map, key, index, error)
        }
    )
}

impl API {
    /// Retrieves the VapourSynth API.
    ///
    /// Returns `None` on error, for example if the requested API version is not supported.
    // If we're linking to VSScript anyway, use the VSScript function.
    #[cfg(all(feature = "vsscript-functions", feature = "gte-vsscript-api-32"))]
    #[inline]
    pub fn get() -> Option<Self> {
        vsscript::maybe_initialize();

        let handle = unsafe { ffi::vsscript_getVSApi2(ffi::VAPOURSYNTH_API_VERSION) };
        if handle.is_null() {
            None
        } else {
            Some(Self { handle })
        }
    }

    /// Retrieves the VapourSynth API.
    ///
    /// Returns `None` on error, for example if the requested API version is not supported.
    #[cfg(all(feature = "vapoursynth-functions",
              not(all(feature = "vsscript-functions", feature = "gte-vsscript-api-32"))))]
    #[inline]
    pub fn get() -> Option<Self> {
        let handle = unsafe { ffi::getVapourSynthAPI(ffi::VAPOURSYNTH_API_VERSION) };
        if handle.is_null() {
            None
        } else {
            Some(Self { handle })
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

    /// Frees `frame`.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid.
    #[inline]
    pub(crate) unsafe fn free_frame(self, frame: *const ffi::VSFrameRef) {
        ((*self.handle).freeFrame)(frame);
    }

    /// Clones `frame`.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid.
    #[inline]
    pub(crate) unsafe fn clone_frame(
        self,
        frame: *const ffi::VSFrameRef,
    ) -> *const ffi::VSFrameRef {
        ((*self.handle).cloneFrameRef)(frame)
    }

    /// Retrieves the format of a frame.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid.
    #[inline]
    pub(crate) unsafe fn get_frame_format(
        self,
        frame: *const ffi::VSFrameRef,
    ) -> *const ffi::VSFormat {
        ((*self.handle).getFrameFormat)(frame)
    }

    /// Returns the width of a plane of a given frame, in pixels.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid and `plane` is valid for the given `frame`.
    #[inline]
    pub(crate) unsafe fn get_frame_width(self, frame: *const ffi::VSFrameRef, plane: i32) -> i32 {
        ((*self.handle).getFrameWidth)(frame, plane)
    }

    /// Returns the height of a plane of a given frame, in pixels.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid and `plane` is valid for the given `frame`.
    #[inline]
    pub(crate) unsafe fn get_frame_height(self, frame: *const ffi::VSFrameRef, plane: i32) -> i32 {
        ((*self.handle).getFrameHeight)(frame, plane)
    }

    /// Returns the distance in bytes between two consecutive lines of a plane of a frame.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid and `plane` is valid for the given `frame`.
    #[inline]
    pub(crate) unsafe fn get_frame_stride(self, frame: *const ffi::VSFrameRef, plane: i32) -> i32 {
        ((*self.handle).getStride)(frame, plane)
    }

    /// Returns a read-only pointer to a plane of a frame.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid and `plane` is valid for the given `frame`.
    #[inline]
    pub(crate) unsafe fn get_frame_read_ptr(
        self,
        frame: *const ffi::VSFrameRef,
        plane: i32,
    ) -> *const u8 {
        ((*self.handle).getReadPtr)(frame, plane)
    }

    /// Returns a read-only pointer to a frame's properties.
    ///
    /// # Safety
    /// The caller must ensure `frame` is valid and the correct lifetime is assigned to the
    /// returned map (it can't outlive `frame`).
    #[inline]
    pub(crate) unsafe fn get_frame_props_ro(
        self,
        frame: *const ffi::VSFrameRef,
    ) -> *const ffi::VSMap {
        ((*self.handle).getFramePropsRO)(frame)
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
    pub(crate) unsafe fn clear_map(self, map: *mut ffi::VSMap) {
        ((*self.handle).clearMap)(map);
    }

    /// Frees `map`.
    ///
    /// # Safety
    /// The caller must ensure `map` is valid.
    #[inline]
    pub(crate) unsafe fn free_map(self, map: *mut ffi::VSMap) {
        ((*self.handle).freeMap)(map);
    }

    /// Returns the number of keys contained in a map.
    ///
    /// # Safety
    /// The caller must ensure `map` is valid.
    #[inline]
    pub(crate) unsafe fn prop_num_keys(self, map: *const ffi::VSMap) -> i32 {
        ((*self.handle).propNumKeys)(map)
    }

    /// Returns a key from a property map.
    ///
    /// # Safety
    /// The caller must ensure `map` is valid and `index` is valid for `map`.
    #[inline]
    pub(crate) unsafe fn prop_get_key(self, map: *const ffi::VSMap, index: i32) -> *const c_char {
        ((*self.handle).propGetKey)(map, index)
    }

    /// Returns the number of elements associated with a key in a property map.
    ///
    /// # Safety
    /// The caller must ensure `map` and `key` are valid.
    #[inline]
    pub(crate) unsafe fn prop_num_elements(
        self,
        map: *const ffi::VSMap,
        key: *const c_char,
    ) -> i32 {
        ((*self.handle).propNumElements)(map, key)
    }

    /// Returns the type of the elements associated with the given key in a property map.
    ///
    /// # Safety
    /// The caller must ensure `map` and `key` are valid.
    #[inline]
    pub(crate) unsafe fn prop_get_type(self, map: *const ffi::VSMap, key: *const c_char) -> c_char {
        ((*self.handle).propGetType)(map, key)
    }

    /// Returns the size in bytes of a property of type ptData.
    ///
    /// # Safety
    /// The caller must ensure `map` and `key` are valid.
    #[inline]
    pub(crate) unsafe fn prop_get_data_size(
        self,
        map: *const ffi::VSMap,
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

    /// Retrieves an array of integers from a map.
    ///
    /// # Safety
    /// The caller must ensure `map` and `key` are valid.
    #[cfg(feature = "gte-vapoursynth-api-31")]
    #[inline]
    pub(crate) unsafe fn prop_get_int_array(
        self,
        map: *const ffi::VSMap,
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
        map: *const ffi::VSMap,
        key: *const c_char,
        error: &mut i32,
    ) -> *const f64 {
        ((*self.handle).propGetFloatArray)(map, key, error)
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
}
