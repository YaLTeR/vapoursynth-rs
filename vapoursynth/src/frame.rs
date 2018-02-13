use std::slice;
use vapoursynth_sys as ffi;

use api::API;
use format::Format;
use map::MapRef;
use video_info::Resolution;

/// An error indicating that the frame data has non-zero padding.
#[derive(Fail, Debug)]
#[fail(display = "Frame data has non-zero padding: {}", _0)]
pub struct NonZeroPadding(usize);

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

    /// Returns the underlying pointer.
    #[inline]
    pub(crate) fn ptr(&self) -> *const ffi::VSFrameRef {
        self.handle
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

    /// Returns a pointer to the plane's pixels.
    ///
    /// The pointer points to an array with a length of `height() * stride()` and is valid for as
    /// long as the frame is alive.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()`.
    pub fn data_ptr(&self, plane: usize) -> *const u8 {
        assert!(plane < self.format().plane_count());

        unsafe { self.api.get_frame_read_ptr(self.handle, plane as i32) }
    }

    /// Returns a slice of a plane's pixel row.
    ///
    /// The length of the returned slice is equal to `width()`.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()`, if `row >= height()` or if the computed row
    /// offset overflows an `isize`.
    pub fn data_row(&self, plane: usize, row: usize) -> &[u8] {
        assert!(plane < self.format().plane_count());
        assert!(row < self.height(plane));

        let stride = self.stride(plane);
        let ptr = self.data_ptr(plane);

        let offset = stride.checked_mul(plane).unwrap();
        assert!(offset <= isize::max_value() as usize);
        let offset = offset as isize;

        let row_ptr = unsafe { ptr.offset(offset) };
        let width = self.width(plane);

        unsafe { slice::from_raw_parts(row_ptr, width) }
    }

    /// Returns a slice of the plane's pixels.
    ///
    /// The length of the returned slice is `height() * width()`. If the pixel data has non-zero
    /// padding (that is, `stride()` is larger than `width()`), and error is returned, since
    /// returning the data slice would open access to uninitialized bytes.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()` or if the computed plane size doesn't fit in a
    /// `usize`.
    pub fn data(&self, plane: usize) -> Result<&[u8], NonZeroPadding> {
        assert!(plane < self.format().plane_count());

        let stride = self.stride(plane);
        let width = self.width(plane);
        if stride != width {
            return Err(NonZeroPadding(stride - width));
        }

        let height = self.height(plane);
        let length = height.checked_mul(stride).unwrap();
        let ptr = self.data_ptr(plane);

        Ok(unsafe { slice::from_raw_parts(ptr, length) })
    }

    /// Returns a map of frame's properties.
    #[inline]
    pub fn props(&self) -> MapRef {
        unsafe { MapRef::from_ptr(self.api, self.api.get_frame_props_ro(self.handle)) }
    }
}
