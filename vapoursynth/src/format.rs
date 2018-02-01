use std::ffi::CStr;
use vapoursynth_sys as ffi;

// TODO: expand this into fields like `VideoInfo`.
/// Contains information about a video format.
#[derive(Debug, Clone, Copy)]
pub struct Format {
    handle: *const ffi::VSFormat,
}

impl PartialEq for Format {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        unsafe { (*self.handle).id == (*other.handle).id }
    }
}

impl Eq for Format {}

impl Format {
    /// Wraps a raw pointer in a `Format`.
    ///
    /// # Safety
    /// The caller must ensure `ptr` is valid.
    #[inline]
    pub(crate) unsafe fn from_ptr(ptr: *const ffi::VSFormat) -> Self {
        Self { handle: ptr }
    }

    /// Gets the printable name of this `Format`.
    #[inline]
    pub fn name(self) -> &'static CStr {
        unsafe { CStr::from_ptr(&(*self.handle).name as _) }
    }

    /// Gets the number of planes of this `Format`.
    #[inline]
    pub fn plane_count(self) -> usize {
        let plane_count = unsafe { (*self.handle).numPlanes };
        assert!(plane_count >= 0);
        plane_count as usize
    }
}
