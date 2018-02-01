use std::slice;
use vapoursynth_sys as ffi;

use api::API;
use format::Format;
use map::MapRef;
use video_info::Resolution;

/// Contains one frame of a clip.
#[derive(Debug)]
pub struct Frame {
    api: API,
    handle: *const ffi::VSFrameRef,
}

unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

impl Drop for Frame {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.api.free_frame(self.handle);
        }
    }
}

impl Clone for Frame {
    #[inline]
    fn clone(&self) -> Self {
        let handle = unsafe { self.api.clone_frame(self.handle) };
        Self {
            api: self.api,
            handle,
        }
    }
}

impl Frame {
    /// Wraps `handle` in a `Frame`.
    ///
    /// # Safety
    /// The caller must ensure `handle` is valid.
    #[inline]
    pub(crate) unsafe fn from_ptr(api: API, handle: *const ffi::VSFrameRef) -> Self {
        Self { api, handle }
    }

    /// Returns the frame format.
    #[inline]
    pub fn format(&self) -> Format {
        unsafe {
            let ptr = self.api.get_frame_format(self.handle);
            Format::from_ptr(ptr)
        }
    }

    /// Returns the width of a plane, in pixels.
    ///
    /// The width depends on the plane number because of the possible chroma subsampling.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()`.
    #[inline]
    pub fn width(&self, plane: usize) -> usize {
        assert!(plane < self.format().plane_count());

        unsafe { self.api.get_frame_width(self.handle, plane as i32) as usize }
    }

    /// Returns the height of a plane, in pixels.
    ///
    /// The height depends on the plane number because of the possible chroma subsampling.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()`.
    #[inline]
    pub fn height(&self, plane: usize) -> usize {
        assert!(plane < self.format().plane_count());

        unsafe { self.api.get_frame_height(self.handle, plane as i32) as usize }
    }

    /// Returns the resolution of a plane.
    ///
    /// The resolution depends on the plane number because of the possible chroma subsampling.
    ///
    /// # Panics
    /// Panics if `plane` is invalid for this frame.
    #[inline]
    pub fn resolution(&self, plane: usize) -> Resolution {
        assert!(plane < self.format().plane_count());

        Resolution {
            width: self.width(plane),
            height: self.height(plane),
        }
    }

    /// Returns the distance in bytes between two consecutive lines of a plane.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()`.
    #[inline]
    pub fn stride(&self, plane: usize) -> usize {
        assert!(plane < self.format().plane_count());

        unsafe { self.api.get_frame_stride(self.handle, plane as i32) as usize }
    }

    /// Returns a slice of the plane's pixels.
    ///
    /// The length of the returned slice is `height() * stride()`.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()` or if the computed plane size doesn't fit in a
    /// `usize`.
    pub fn data(&self, plane: usize) -> &[u8] {
        assert!(plane < self.format().plane_count());

        let height = self.height(plane);
        let stride = self.stride(plane);
        let length = height.checked_mul(stride).unwrap();
        let ptr = unsafe { self.api.get_frame_read_ptr(self.handle, plane as i32) };

        unsafe { slice::from_raw_parts(ptr, length) }
    }

    /// Returns a map of frame's properties.
    #[inline]
    pub fn props(&self) -> MapRef {
        unsafe { MapRef::from_ptr(self.api, self, self.api.get_frame_props_ro(self.handle)) }
    }
}
