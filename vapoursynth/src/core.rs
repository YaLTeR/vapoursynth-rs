use std::ffi::CStr;
use std::fmt;
use std::marker::PhantomData;
use vapoursynth_sys as ffi;

use api::API;
use format::{ColorFamily, Format, FormatID, SampleType};

/// Contains information about a VapourSynth core.
#[derive(Debug, Clone, Copy, Hash)]
pub struct Info {
    /// String containing the name of the library, copyright notice, core and API versions.
    pub version_string: &'static str,

    /// Version of the core.
    pub core_version: i32,

    /// Version of the API.
    pub api_version: i32,

    /// Number of worker threads.
    pub num_threads: usize,

    /// The framebuffer cache will be allowed to grow up to this size (bytes) before memory is
    /// aggressively reclaimed.
    pub max_framebuffer_size: u64,

    /// Current size of the framebuffer cache, in bytes.
    pub used_framebuffer_size: u64,
}

/// A reference to a VapourSynth core.
#[derive(Debug, Clone, Copy)]
pub struct CoreRef<'a> {
    api: API,
    handle: *mut ffi::VSCore,
    _owner: PhantomData<&'a ()>,
}

unsafe impl<'a> Send for CoreRef<'a> {}
unsafe impl<'a> Sync for CoreRef<'a> {}

impl<'a> CoreRef<'a> {
    /// Wraps `handle` in a `CoreRef`.
    ///
    /// # Safety
    /// The caller must ensure `handle` is valid.
    #[inline]
    pub(crate) unsafe fn from_ptr(api: API, handle: *mut ffi::VSCore) -> Self {
        Self {
            api,
            handle,
            _owner: PhantomData,
        }
    }

    /// Returns information about the VapourSynth core.
    pub fn info(self) -> Info {
        let raw_info = unsafe { self.api.get_core_info(self.handle).as_ref().unwrap() };

        let version_string = unsafe { CStr::from_ptr(raw_info.versionString).to_str().unwrap() };
        debug_assert!(raw_info.numThreads >= 0);
        debug_assert!(raw_info.maxFramebufferSize >= 0);
        debug_assert!(raw_info.usedFramebufferSize >= 0);

        Info {
            version_string,
            core_version: raw_info.core,
            api_version: raw_info.api,
            num_threads: raw_info.numThreads as usize,
            max_framebuffer_size: raw_info.maxFramebufferSize as u64,
            used_framebuffer_size: raw_info.usedFramebufferSize as u64,
        }
    }

    /// Retrieves a registered or preset `Format` by its id. The id can be of a previously
    /// registered format, or one of the `PresetFormat`.
    #[inline]
    pub fn get_format(&self, id: FormatID) -> Option<Format> {
        let ptr = unsafe { self.api.get_format_preset(id.0, self.handle) };
        unsafe { ptr.as_ref().map(|p| Format::from_ptr(p)) }
    }

    /// Registers a custom video format.
    ///
    /// Returns `None` if an invalid format is described.
    ///
    /// Registering compat formats is not allowed. Only certain privileged built-in filters are
    /// allowed to handle compat formats.
    ///
    /// RGB formats are not allowed to be subsampled.
    #[inline]
    pub fn register_format(
        &self,
        color_family: ColorFamily,
        sample_type: SampleType,
        bits_per_sample: u8,
        sub_sampling_w: u8,
        sub_sampling_h: u8,
    ) -> Option<Format> {
        unsafe {
            self.api
                .register_format(
                    color_family.into(),
                    sample_type.into(),
                    i32::from(bits_per_sample),
                    i32::from(sub_sampling_w),
                    i32::from(sub_sampling_h),
                    self.handle,
                )
                .as_ref()
                .map(|p| Format::from_ptr(p))
        }
    }
}

impl fmt::Display for Info {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.version_string)?;
        writeln!(f, "Worker threads: {}", self.num_threads)?;
        writeln!(
            f,
            "Max framebuffer cache size: {}",
            self.max_framebuffer_size
        )?;
        writeln!(
            f,
            "Current framebuffer cache size: {}",
            self.used_framebuffer_size
        )
    }
}
