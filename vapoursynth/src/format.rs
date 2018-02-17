use std::ffi::CStr;
use vapoursynth_sys as ffi;

// TODO: expand this into fields like `VideoInfo`.
/// Contains information about a video format.
#[derive(Debug, Clone, Copy)]
pub struct Format {
    handle: *const ffi::VSFormat,
}

/// Preset VapourSynth formats.
///
/// The presets suffixed with H and S have floating point sample type. The H and S suffixes stand
/// for half precision and single precision, respectively.
///
/// The compat formats are the only packed formats in VapourSynth. Everything else is planar. They
/// exist for compatibility with Avisynth plugins. They are not to be implemented in native
/// VapourSynth plugins.
#[cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]
#[repr(i32)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PresetFormat {
    Gray8 = 1000010,
    Gray16 = 1000011,
    GrayH = 1000012,
    GrayS = 1000013,
    YUV420P8 = 3000010,
    YUV422P8 = 3000011,
    YUV444P8 = 3000012,
    YUV410P8 = 3000013,
    YUV411P8 = 3000014,
    YUV440P8 = 3000015,
    YUV420P9 = 3000016,
    YUV422P9 = 3000017,
    YUV444P9 = 3000018,
    YUV420P10 = 3000019,
    YUV422P10 = 3000020,
    YUV444P10 = 3000021,
    YUV420P16 = 3000022,
    YUV422P16 = 3000023,
    YUV444P16 = 3000024,
    YUV444PH = 3000025,
    YUV444PS = 3000026,
    YUV420P12 = 3000027,
    YUV422P12 = 3000028,
    YUV444P12 = 3000029,
    YUV420P14 = 3000030,
    YUV422P14 = 3000031,
    YUV444P14 = 3000032,
    RGB24 = 2000010,
    RGB27 = 2000011,
    RGB30 = 2000012,
    RGB48 = 2000013,
    RGBH = 2000014,
    RGBS = 2000015,
    CompatBGR32 = 9000010,
    CompatYUY2 = 9000011,
}

/// Format color families.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ColorFamily {
    Gray,
    RGB,
    YUV,
    YCoCg,
    Compat,
}

/// Format sample types.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SampleType {
    Integer,
    Float,
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

    /// Gets the unique identifier of this `Format`.
    #[inline]
    pub fn id(self) -> i32 {
        unsafe { (*self.handle).id }
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
        debug_assert!(plane_count >= 0);
        plane_count as usize
    }
}

impl From<PresetFormat> for i32 {
    fn from(x: PresetFormat) -> Self {
        x as i32
    }
}

impl ColorFamily {
    #[inline]
    pub(crate) fn ffi_type(self) -> ffi::VSColorFamily {
        match self {
            ColorFamily::Gray => ffi::VSColorFamily::cmGray,
            ColorFamily::RGB => ffi::VSColorFamily::cmRGB,
            ColorFamily::YUV => ffi::VSColorFamily::cmYUV,
            ColorFamily::YCoCg => ffi::VSColorFamily::cmYCoCg,
            ColorFamily::Compat => ffi::VSColorFamily::cmCompat,
        }
    }
}

impl SampleType {
    #[inline]
    pub(crate) fn ffi_type(self) -> ffi::VSSampleType {
        match self {
            SampleType::Integer => ffi::VSSampleType::stInteger,
            SampleType::Float => ffi::VSSampleType::stFloat,
        }
    }
}
