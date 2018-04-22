//! VapourSynth plugins.

use std::ffi::{CStr, CString, NulError};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use vapoursynth_sys as ffi;

use api::API;
use map::{Map, OwnedMap};
use plugins::{self, FilterFunction};

/// A VapourSynth plugin.
#[derive(Debug, Clone, Copy)]
pub struct Plugin<'core> {
    handle: NonNull<ffi::VSPlugin>,
    _owner: PhantomData<&'core ()>,
}

unsafe impl<'core> Send for Plugin<'core> {}
unsafe impl<'core> Sync for Plugin<'core> {}

impl<'core> Plugin<'core> {
    /// Wraps `handle` in a `Plugin`.
    ///
    /// # Safety
    /// The caller must ensure `handle` is valid and API is cached.
    #[inline]
    pub(crate) unsafe fn from_ptr(handle: *mut ffi::VSPlugin) -> Self {
        Self {
            handle: NonNull::new_unchecked(handle),
            _owner: PhantomData,
        }
    }

    /// Returns a map containing a list of the filters exported by a plugin.
    ///
    /// Keys: the filter names;
    ///
    /// Values: the filter name followed by its argument string, separated by a semicolon.
    // TODO: parse the values on the crate side and return a nice struct.
    #[inline]
    pub fn functions(&self) -> OwnedMap<'core> {
        unsafe { OwnedMap::from_ptr(API::get_cached().get_functions(self.handle.as_ptr())) }
    }

    /// Returns the absolute path to the plugin, including the plugin's file name. This is the real
    /// location of the plugin, i.e. there are no symbolic links in the path.
    ///
    /// Path elements are always delimited with forward slashes.
    #[cfg(feature = "gte-vapoursynth-api-31")]
    #[inline]
    pub fn path(&self) -> Option<&'core CStr> {
        let ptr = unsafe { API::get_cached().get_plugin_path(self.handle.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) })
        }
    }

    /// Invokes a filter.
    ///
    /// `invoke()` makes sure the filter has no compat input nodes, checks that the args passed to
    /// the filter are consistent with the argument list registered by the plugin that contains the
    /// filter, creates the filter, and checks that the filter doesn't return any compat nodes. If
    /// everything goes smoothly, the filter will be ready to generate frames after `invoke()`
    /// returns.
    ///
    /// Returns a map containing the filter's return value(s). Use `Map::error()` to check if the
    /// filter was invoked successfully.
    ///
    /// Most filters will either add an error to the map, or one or more clips with the key `clip`.
    /// The exception to this are functions, for example `LoadPlugin`, which doesn't return any
    /// clips for obvious reasons.
    #[inline]
    pub fn invoke(&self, name: &str, args: &Map<'core>) -> Result<OwnedMap<'core>, NulError> {
        let name = CString::new(name)?;
        Ok(unsafe {
            OwnedMap::from_ptr(API::get_cached().invoke(
                self.handle.as_ptr(),
                name.as_ptr(),
                args.deref(),
            ))
        })
    }

    /// Registers a filter function to be exported by a non-readonly plugin.
    #[inline]
    pub fn register_function<F: FilterFunction>(&self, filter_function: F) -> Result<(), NulError> {
        // TODO: this is almost the same code as plugins::ffi::call_register_function().
        let name_cstring = CString::new(filter_function.name())?;
        let args_cstring = CString::new(filter_function.args())?;

        let data = Box::new(plugins::ffi::FilterFunctionData::<F> {
            filter_function,
            name: name_cstring,
        });

        unsafe {
            API::get_cached().register_function(
                data.name.as_ptr(),
                args_cstring.as_ptr(),
                plugins::ffi::create::<F>,
                Box::into_raw(data) as _,
                self.handle.as_ptr(),
            );
        }

        Ok(())
    }
}
