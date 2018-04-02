//! VapourSynth frames.

use std::{mem, ptr, slice};
use std::ops::{Deref, DerefMut};
use vapoursynth_sys as ffi;

use api::API;
use core::CoreRef;
use format::Format;
use map::Map;
use video_info::Resolution;

/// An error indicating that the frame data has non-zero padding.
#[derive(Fail, Debug)]
#[fail(display = "Frame data has non-zero padding: {}", _0)]
pub struct NonZeroPadding(usize);

/// One frame of a clip.
// WARNING: use ONLY references to this type. The only thing this type is for is doing
// &ffi::VSFrameRef and &mut ffi::VSFrameRef without exposing the (unknown size) ffi type outside.
pub struct Frame(ffi::VSFrameRef);

unsafe impl Sync for Frame {}

#[doc(hidden)]
impl Deref for Frame {
    type Target = ffi::VSFrameRef;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(self) }
    }
}

#[doc(hidden)]
impl DerefMut for Frame {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { mem::transmute(self) }
    }
}

/// A reference to a ref-counted frame.
#[derive(Debug)]
pub struct FrameRef {
    handle: *const ffi::VSFrameRef,
}

unsafe impl Send for FrameRef {}
unsafe impl Sync for FrameRef {}

/// A reference to a mutable frame.
#[derive(Debug)]
pub struct FrameRefMut {
    handle: *mut ffi::VSFrameRef,
}

unsafe impl Send for FrameRefMut {}
unsafe impl Sync for FrameRefMut {}

impl Drop for FrameRef {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            API::get_cached().free_frame(self);
        }
    }
}

impl Clone for FrameRef {
    #[inline]
    fn clone(&self) -> Self {
        let handle = unsafe { API::get_cached().clone_frame(self) };
        Self { handle }
    }
}

impl Drop for FrameRefMut {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            API::get_cached().free_frame(self);
        }
    }
}

impl Deref for FrameRef {
    type Target = Frame;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { Frame::from_ptr(self.handle) }
    }
}

impl Deref for FrameRefMut {
    type Target = Frame;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { Frame::from_ptr(self.handle) }
    }
}

impl DerefMut for FrameRefMut {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { Frame::from_mut_ptr(self.handle) }
    }
}

impl FrameRef {
    /// Wraps `handle` in a `FrameRef`.
    ///
    /// # Safety
    /// The caller must ensure `handle` is valid and API is cached.
    #[inline]
    pub(crate) unsafe fn from_ptr(handle: *const ffi::VSFrameRef) -> Self {
        Self { handle }
    }

    /// Returns the underlying pointer.
    #[inline]
    pub(crate) fn ptr(&self) -> *const ffi::VSFrameRef {
        self.handle
    }
}

impl FrameRefMut {
    /// Wraps `handle` in a `FrameRefMut`.
    ///
    /// # Safety
    /// The caller must ensure `handle` is valid and API is cached.
    #[inline]
    pub(crate) unsafe fn from_ptr(handle: *mut ffi::VSFrameRef) -> Self {
        Self { handle }
    }

    /// Returns the underlying pointer.
    #[inline]
    pub(crate) fn ptr(&self) -> *mut ffi::VSFrameRef {
        self.handle
    }

    /// Creates a copy of the given frame.
    ///
    /// The plane data is copy-on-write, so this isn't very expensive by itself.
    ///
    /// Judging by the underlying implementation, it seems that any valid `core` can be used.
    #[inline]
    pub fn copy_of(core: CoreRef, frame: &Frame) -> Self {
        Self {
            handle: unsafe { API::get_cached().copy_frame(frame, core.ptr()) },
        }
    }

    /// Creates a new frame with uninitialized plane data.
    ///
    /// Optionally copies the frame properties from the provided `prop_src` frame.
    ///
    /// # Safety
    /// The returned frame contains uninitialized plane data. This should be handled carefully. See
    /// the docs for `std::mem::uninitialized()` for more information.
    ///
    /// # Panics
    /// Panics if the given resolution has components that don't fit into an `i32`.
    #[inline]
    pub unsafe fn new_uninitialized(
        core: CoreRef,
        prop_src: Option<&Frame>,
        format: Format,
        resolution: Resolution,
    ) -> Self {
        assert!(resolution.width <= i32::max_value() as usize);
        assert!(resolution.height <= i32::max_value() as usize);

        Self {
            handle: unsafe {
                API::get_cached().new_video_frame(
                    format.ptr(),
                    resolution.width as i32,
                    resolution.height as i32,
                    prop_src.map(Frame::ptr).unwrap_or(ptr::null()),
                    core.ptr(),
                )
            },
        }
    }
}

impl From<FrameRefMut> for FrameRef {
    #[inline]
    fn from(x: FrameRefMut) -> FrameRef {
        let rv = FrameRef { handle: x.handle };
        mem::forget(x);
        rv
    }
}

impl Frame {
    /// Converts a pointer to a frame to a reference.
    ///
    /// # Safety
    /// The caller needs to ensure the pointer is valid, the lifetime is valid and there are no
    /// active mutable references to the map during the lifetime.
    #[inline]
    pub(crate) unsafe fn from_ptr<'a>(handle: *const ffi::VSFrameRef) -> &'a Frame {
        #[cfg_attr(feature = "cargo-clippy", allow(transmute_ptr_to_ref))]
        unsafe { mem::transmute(handle) }
    }

    /// Converts a mutable pointer to a frame to a reference.
    ///
    /// # Safety
    /// The caller needs to ensure the pointer is valid, the lifetime is valid and there are no
    /// active references to the map during the lifetime.
    #[inline]
    pub(crate) unsafe fn from_mut_ptr<'a>(handle: *mut ffi::VSFrameRef) -> &'a mut Frame {
        #[cfg_attr(feature = "cargo-clippy", allow(transmute_ptr_to_ref))]
        unsafe { mem::transmute(handle) }
    }

    /// Returns the underlying pointer.
    #[inline]
    pub(crate) fn ptr(&self) -> *const ffi::VSFrameRef {
        self.deref()
    }

    /// Returns the frame format.
    #[inline]
    pub fn format(&self) -> Format {
        unsafe {
            let ptr = API::get_cached().get_frame_format(self);
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

        unsafe { API::get_cached().get_frame_width(self, plane as i32) as usize }
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

        unsafe { API::get_cached().get_frame_height(self, plane as i32) as usize }
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

        unsafe { API::get_cached().get_frame_stride(self, plane as i32) as usize }
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

        unsafe { API::get_cached().get_frame_read_ptr(self, plane as i32) }
    }

    /// Returns a mutable pointer to the plane's pixels.
    ///
    /// The pointer points to an array with a length of `height() * stride()` and is valid for as
    /// long as the frame is alive.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()`.
    pub fn data_ptr_mut(&mut self, plane: usize) -> *mut u8 {
        assert!(plane < self.format().plane_count());

        unsafe { API::get_cached().get_frame_write_ptr(self, plane as i32) }
    }

    /// Returns a slice of a plane's pixel row.
    ///
    /// The length of the returned slice is equal to `width() * format().bytes_per_sample()`.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()` or if `row >= height()`.
    pub fn data_row(&self, plane: usize, row: usize) -> &[u8] {
        assert!(plane < self.format().plane_count());
        assert!(row < self.height(plane));

        let stride = self.stride(plane);
        let ptr = self.data_ptr(plane);

        let offset = stride * row;
        assert!(offset <= isize::max_value() as usize);
        let offset = offset as isize;

        let row_ptr = unsafe { ptr.offset(offset) };
        let width = self.width(plane) * usize::from(self.format().bytes_per_sample());

        unsafe { slice::from_raw_parts(row_ptr, width) }
    }

    /// Returns a mutable slice of a plane's pixel row.
    ///
    /// The length of the returned slice is equal to `width() * format().bytes_per_sample()`.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()` or if `row >= height()`.
    pub fn data_row_mut(&mut self, plane: usize, row: usize) -> &mut [u8] {
        assert!(plane < self.format().plane_count());
        assert!(row < self.height(plane));

        let stride = self.stride(plane);
        let ptr = self.data_ptr_mut(plane);

        let offset = stride * row;
        assert!(offset <= isize::max_value() as usize);
        let offset = offset as isize;

        let row_ptr = unsafe { ptr.offset(offset) };
        let width = self.width(plane) * usize::from(self.format().bytes_per_sample());

        unsafe { slice::from_raw_parts_mut(row_ptr, width) }
    }

    /// Returns a slice of the plane's pixels.
    ///
    /// The length of the returned slice is `height() * width() * format().bytes_per_sample()`. If
    /// the pixel data has non-zero padding (that is, `stride()` is larger than `width()`), and
    /// error is returned, since returning the data slice would open access to uninitialized bytes.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()` or if `row >= height()`.
    pub fn data(&self, plane: usize) -> Result<&[u8], NonZeroPadding> {
        assert!(plane < self.format().plane_count());

        let stride = self.stride(plane);
        let width = self.width(plane) * usize::from(self.format().bytes_per_sample());
        if stride != width {
            return Err(NonZeroPadding(stride - width));
        }

        let height = self.height(plane);
        let length = height * stride;
        let ptr = self.data_ptr(plane);

        Ok(unsafe { slice::from_raw_parts(ptr, length) })
    }

    /// Returns a mutable slice of the plane's pixels.
    ///
    /// The length of the returned slice is `height() * width() * format().bytes_per_sample()`. If
    /// the pixel data has non-zero padding (that is, `stride()` is larger than `width()`), and
    /// error is returned, since returning the data slice would open access to uninitialized bytes.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()` or if `row >= height()`.
    pub fn data_mut(&mut self, plane: usize) -> Result<&mut [u8], NonZeroPadding> {
        assert!(plane < self.format().plane_count());

        let stride = self.stride(plane);
        let width = self.width(plane) * usize::from(self.format().bytes_per_sample());
        if stride != width {
            return Err(NonZeroPadding(stride - width));
        }

        let height = self.height(plane);
        let length = height * stride;
        let ptr = self.data_ptr_mut(plane);

        Ok(unsafe { slice::from_raw_parts_mut(ptr, length) })
    }

    /// Returns a map of frame's properties.
    #[inline]
    pub fn props(&self) -> &Map {
        unsafe { Map::from_ptr(API::get_cached().get_frame_props_ro(self)) }
    }

    /// Returns a mutable map of frame's properties.
    #[inline]
    pub fn props_mut(&mut self) -> &mut Map {
        unsafe { Map::from_mut_ptr(API::get_cached().get_frame_props_rw(self)) }
    }
}
