use vapoursynth_sys as ffi;

/// A wrapper for the VapourSynth API.
#[derive(Debug, Clone, Copy)]
pub struct API {
    handle: *const ffi::VSAPI,
}

impl API {
    /// Retrieves the VapourSynth API.
    ///
    /// Returns `None` on error, for example if the requested API version is not supported.
    #[cfg(feature = "vapoursynth-functions")]
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
    pub(crate) unsafe fn free_node(self, node: *mut ffi::VSNodeRef) {
        ((*self.handle).freeNode)(node);
    }

    /// Clones `node`.
    ///
    /// # Safety
    /// The caller must ensure `node` is valid.
    pub(crate) unsafe fn clone_node(self, node: *mut ffi::VSNodeRef) -> *mut ffi::VSNodeRef {
        ((*self.handle).cloneNodeRef)(node)
    }

    /// Returns a pointer to the video info associated with `node`. The pointer is valid as long as
    /// the node lives.
    ///
    /// # Safety
    /// The caller must ensure `node` is valid.
    pub(crate) unsafe fn get_video_info(
        self,
        node: *mut ffi::VSNodeRef,
    ) -> *const ffi::VSVideoInfo {
        ((*self.handle).getVideoInfo)(node)
    }
}
