use std::ffi::CStr;
use std::marker::PhantomData;
use vapoursynth_sys as ffi;

/// Contains information about a video format.
#[derive(Debug, Clone, Copy, Eq)]
pub struct Format<'a> {
    handle: *const ffi::VSFormat,
    _owner: PhantomData<&'a ()>,
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

/// A unique format identifier.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FormatID(pub(crate) i32);

impl<'a> PartialEq for Format<'a> {
    #[inline]
    fn eq(&self, other: &Format) -> bool {
        self.id() == other.id()
    }
}

impl<'a> Format<'a> {
    /// Wraps a raw pointer in a `Format`.
    ///
    /// # Safety
    /// The caller must ensure `ptr` is valid.
    #[inline]
    pub(crate) unsafe fn from_ptr(ptr: *const ffi::VSFormat) -> Self {
        Self {
            handle: ptr,
            _owner: PhantomData,
        }
    }

    /// Gets the unique identifier of this format.
    #[inline]
    pub fn id(self) -> FormatID {
        FormatID(unsafe { (*self.handle).id })
    }

    /// Gets the printable name of this format.
    #[inline]
    pub fn name(self) -> &'a str {
        unsafe { CStr::from_ptr(&(*self.handle).name as _).to_str().unwrap() }
    }

    /// Gets the number of planes of this format.
    #[inline]
    pub fn plane_count(self) -> usize {
        let plane_count = unsafe { (*self.handle).numPlanes };
        debug_assert!(plane_count >= 0);
        plane_count as usize
    }

    /// Gets the color family of this format.
    #[inline]
    pub fn color_family(self) -> ColorFamily {
        match unsafe { (*self.handle).colorFamily } {
            x if x == ffi::VSColorFamily::cmGray as i32 => ColorFamily::Gray,
            x if x == ffi::VSColorFamily::cmRGB as i32 => ColorFamily::RGB,
            x if x == ffi::VSColorFamily::cmYUV as i32 => ColorFamily::YUV,
            x if x == ffi::VSColorFamily::cmYCoCg as i32 => ColorFamily::YCoCg,
            x if x == ffi::VSColorFamily::cmCompat as i32 => ColorFamily::Compat,
            _ => unreachable!(),
        }
    }

    /// Gets the sample type of this format.
    #[inline]
    pub fn sample_type(self) -> SampleType {
        match unsafe { (*self.handle).sampleType } {
            x if x == ffi::VSSampleType::stInteger as i32 => SampleType::Integer,
            x if x == ffi::VSSampleType::stFloat as i32 => SampleType::Float,
            _ => unreachable!(),
        }
    }

    /// Gets the number of significant bits per sample.
    #[inline]
    pub fn bits_per_sample(self) -> u8 {
        let rv = unsafe { (*self.handle).bitsPerSample };
        debug_assert!(rv >= 0 && rv <= i32::from(u8::max_value()));
        rv as u8
    }

    /// Gets the number of bytes needed for a sample. This is always a power of 2 and the smallest
    /// possible that can fit the number of bits used per sample.
    #[inline]
    pub fn bytes_per_sample(self) -> u8 {
        let rv = unsafe { (*self.handle).bytesPerSample };
        debug_assert!(rv >= 0 && rv <= i32::from(u8::max_value()));
        rv as u8
    }

    /// log2 subsampling factor, applied to second and third plane.
    #[inline]
    pub fn sub_sampling_w(self) -> u8 {
        let rv = unsafe { (*self.handle).subSamplingW };
        debug_assert!(rv >= 0 && rv <= i32::from(u8::max_value()));
        rv as u8
    }

    /// log2 subsampling factor, applied to second and third plane.
    #[inline]
    pub fn sub_sampling_h(self) -> u8 {
        let rv = unsafe { (*self.handle).subSamplingH };
        debug_assert!(rv >= 0 && rv <= i32::from(u8::max_value()));
        rv as u8
    }
}

impl From<PresetFormat> for FormatID {
    fn from(x: PresetFormat) -> Self {
        FormatID(x as i32)
    }
}

#[doc(hidden)]
impl From<ColorFamily> for ffi::VSColorFamily {
    #[inline]
    fn from(x: ColorFamily) -> Self {
        match x {
            ColorFamily::Gray => ffi::VSColorFamily::cmGray,
            ColorFamily::RGB => ffi::VSColorFamily::cmRGB,
            ColorFamily::YUV => ffi::VSColorFamily::cmYUV,
            ColorFamily::YCoCg => ffi::VSColorFamily::cmYCoCg,
            ColorFamily::Compat => ffi::VSColorFamily::cmCompat,
        }
    }
}

#[doc(hidden)]
impl From<SampleType> for ffi::VSSampleType {
    #[inline]
    fn from(x: SampleType) -> Self {
        match x {
            SampleType::Integer => ffi::VSSampleType::stInteger,
            SampleType::Float => ffi::VSSampleType::stFloat,
        }
    }
}
