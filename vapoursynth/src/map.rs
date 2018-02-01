use std::ffi::CStr;
use std::marker::PhantomData;
use vapoursynth_sys as ffi;

use api::API;

// TODO: impl Eq on this stuff, impl Clone on Map, impl From for/to HashMap.

/// A non-owned non-mutable VapourSynth map.
#[derive(Debug, Clone, Copy)]
pub struct MapRef<'a> {
    api: API,
    handle: *const ffi::VSMap,
    owner: PhantomData<&'a ()>,
}

unsafe impl<'a> Send for MapRef<'a> {}
unsafe impl<'a> Sync for MapRef<'a> {}

impl<'a> MapRef<'a> {
    /// Wraps `handle` in a `MapRef`.
    ///
    /// # Safety
    /// The caller must ensure `handle` is valid and the provided owner's lifetime is correct for
    /// the given `handle`.
    #[inline]
    pub(crate) unsafe fn from_ptr<T>(api: API, _owner: &'a T, handle: *const ffi::VSMap) -> Self {
        Self {
            api,
            handle,
            owner: PhantomData,
        }
    }
}

/// A non-owned mutable VapourSynth map.
#[derive(Debug)]
pub struct MapRefMut<'a> {
    api: API,
    handle: *mut ffi::VSMap,
    owner: PhantomData<&'a mut ()>,
}

unsafe impl<'a> Send for MapRefMut<'a> {}
unsafe impl<'a> Sync for MapRefMut<'a> {}

/// An owned VapourSynth map.
#[derive(Debug)]
pub struct Map {
    api: API,
    handle: *mut ffi::VSMap,
}

unsafe impl Send for Map {}
unsafe impl Sync for Map {}

impl Drop for Map {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.api.free_map(self.handle);
        }
    }
}

impl Map {
    /// Creates a new `Map`.
    #[inline]
    pub fn new(api: API) -> Self {
        let handle = api.create_map();

        Self { api, handle }
    }
}

/// A non-mutable VapourSynth map interface.
///
/// This trait is sealed and is not meant for implementation outside of this crate.
pub trait VSMap: sealed::VSMapInterface {
    /// Returns the number of keys contained in a map.
    #[inline]
    fn key_count(&self) -> usize {
        let count = unsafe { self.api().prop_num_keys(self.handle()) };
        assert!(count >= 0);
        count as usize
    }

    /// Returns a key from a map.
    ///
    /// # Panics
    /// Panics if `index >= self.key_count()`.
    #[inline]
    fn key(&self, index: usize) -> &CStr {
        assert!(index < self.key_count());
        let index = index as i32;

        unsafe { CStr::from_ptr(self.api().prop_get_key(self.handle(), index)) }
    }
}

/// A mutable VapourSynth map interface.
///
/// This trait is sealed and is not meant for implementation outside of this crate.
pub trait VSMapMut: VSMap + sealed::VSMapMutInterface {
    /// Clears the map.
    #[inline]
    fn clear(&mut self) {
        unsafe {
            self.api().clear_map(self.handle_mut());
        }
    }
}

// Do this manually for each type so it shows up in rustdoc
impl<'a> VSMap for MapRef<'a> {}
impl<'a> VSMap for MapRefMut<'a> {}
impl<'a> VSMapMut for MapRefMut<'a> {}
impl VSMap for Map {}
impl VSMapMut for Map {}

mod sealed {
    use super::*;

    /// An interface for a non-mutable VapourSynth map.
    pub trait VSMapInterface {
        fn api(&self) -> API;
        fn handle(&self) -> *const ffi::VSMap;
    }

    /// An interface for a mutable VapourSynth map.
    pub trait VSMapMutInterface: VSMapInterface {
        fn handle_mut(&mut self) -> *mut ffi::VSMap;
    }

    impl<'a> VSMapInterface for MapRef<'a> {
        #[inline]
        fn api(&self) -> API {
            self.api
        }

        #[inline]
        fn handle(&self) -> *const ffi::VSMap {
            self.handle
        }
    }

    impl<'a> VSMapInterface for MapRefMut<'a> {
        #[inline]
        fn api(&self) -> API {
            self.api
        }

        #[inline]
        fn handle(&self) -> *const ffi::VSMap {
            self.handle
        }
    }

    impl<'a> VSMapMutInterface for MapRefMut<'a> {
        #[inline]
        fn handle_mut(&mut self) -> *mut ffi::VSMap {
            self.handle
        }
    }

    impl VSMapInterface for Map {
        #[inline]
        fn api(&self) -> API {
            self.api
        }

        #[inline]
        fn handle(&self) -> *const ffi::VSMap {
            self.handle
        }
    }

    impl VSMapMutInterface for Map {
        #[inline]
        fn handle_mut(&mut self) -> *mut ffi::VSMap {
            self.handle
        }
    }
}
