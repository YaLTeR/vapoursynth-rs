use std::collections::HashMap;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::slice;
use vapoursynth_sys as ffi;

use api::API;
use frame::Frame;
use function::Function;
use node::Node;

mod errors;
pub use self::errors::Error;
use self::errors::Result;

mod iterators;
pub use self::iterators::{Iter, Keys};

mod value;
pub use self::value::{Value, ValueArray};

// TODO: impl Eq on this stuff, impl Clone on Map, impl From for/to HashMap.
// TODO: the way current traits work is they return objects with lifetime bounds of the MapRefs
// while they should probably be bound to lifetimes of owners instead.

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
    pub(crate) unsafe fn from_ptr(api: API, handle: *const ffi::VSMap) -> Self {
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

impl<'a> MapRefMut<'a> {
    /// Wraps `handle` in a `MapRefMut`.
    ///
    /// # Safety
    /// The caller must ensure `handle` is valid and the provided owner's lifetime is correct for
    /// the given `handle`.
    #[inline]
    pub(crate) unsafe fn from_ptr(api: API, handle: *mut ffi::VSMap) -> Self {
        Self {
            api,
            handle,
            owner: PhantomData,
        }
    }
}

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
        debug_assert!(count >= 0);
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

    /// Returns an iterator over all keys in a map.
    fn keys(&self) -> Keys<Self>
    where
        Self: Sized,
    {
        Keys::new(self)
    }

    /// Returns the number of elements associated with a key in a map.
    ///
    /// If there's no such key in the map, `None` is returned.
    #[inline]
    fn value_count(&self, key: &CStr) -> Option<usize> {
        let rv = unsafe { self.api().prop_num_elements(self.handle(), key.as_ptr()) };
        if rv == -1 {
            None
        } else {
            debug_assert!(rv >= 0);
            Some(rv as usize)
        }
    }

    /// Retrieves a value type from a map.
    #[doc(hidden)]
    fn value_type(&self, key: &CStr) -> ffi::VSPropTypes {
        match unsafe { self.api().prop_get_type(self.handle(), key.as_ptr()) } {
            x if x == ffi::VSPropTypes::ptUnset as c_char => ffi::VSPropTypes::ptUnset,
            x if x == ffi::VSPropTypes::ptInt as c_char => ffi::VSPropTypes::ptInt,
            x if x == ffi::VSPropTypes::ptFloat as c_char => ffi::VSPropTypes::ptFloat,
            x if x == ffi::VSPropTypes::ptData as c_char => ffi::VSPropTypes::ptData,
            x if x == ffi::VSPropTypes::ptNode as c_char => ffi::VSPropTypes::ptNode,
            x if x == ffi::VSPropTypes::ptFrame as c_char => ffi::VSPropTypes::ptFrame,
            x if x == ffi::VSPropTypes::ptFunction as c_char => ffi::VSPropTypes::ptFunction,
            _ => unreachable!(),
        }
    }

    /// Retrieves a value from a map.
    ///
    /// # Panics
    /// Panics if `index > i32::max_value()`.
    fn value(&self, key: &CStr, index: usize) -> Result<Value> {
        assert!(index <= i32::max_value() as usize);
        let index = index as i32;

        macro_rules! get_value {
            ($func:ident, $value:path, $process:expr) => {{
                let mut error = 0;
                let value = unsafe {
                    self.api().$func(self.handle(), key.as_ptr(), index, &mut error)
                };

                match error {
                    0 => {}
                    x if x == ffi::VSGetPropErrors::peIndex as i32 => {
                        return Err(Error::IndexOutOfBounds)
                    }
                    _ => unreachable!(),
                }

                Ok($value($process(value)))
            }}
        }

        match self.value_type(key) {
            ffi::VSPropTypes::ptUnset => Err(Error::KeyNotFound),
            ffi::VSPropTypes::ptInt => get_value!(prop_get_int, Value::Int, |x| x),
            ffi::VSPropTypes::ptFloat => get_value!(prop_get_float, Value::Float, |x| x),
            ffi::VSPropTypes::ptData => get_value!(prop_get_data, Value::Data, |x| {
                let mut error = 0;
                let size = unsafe {
                    self.api()
                        .prop_get_data_size(self.handle(), key.as_ptr(), index, &mut error)
                };
                debug_assert!(error == 0);
                debug_assert!(size >= 0);
                unsafe { slice::from_raw_parts(x as *const u8, size as usize) }
            }),
            ffi::VSPropTypes::ptNode => get_value!(prop_get_node, Value::Node, |x| unsafe {
                Node::from_ptr(self.api(), x)
            }),
            ffi::VSPropTypes::ptFrame => get_value!(prop_get_frame, Value::Frame, |x| unsafe {
                Frame::from_ptr(self.api(), x)
            }),
            ffi::VSPropTypes::ptFunction => {
                get_value!(prop_get_func, Value::Function, |x| unsafe {
                    Function::from_ptr(self.api(), x)
                })
            }
        }
    }

    /// Retrieves all values for a given key from a map.
    fn values(&self, key: &CStr) -> Result<ValueArray> {
        let count = self.value_count(key).ok_or(Error::KeyNotFound)?;

        #[cfg(feature = "gte-vapoursynth-api-31")]
        macro_rules! get_value_array {
            ($func:ident, $value:path) => {{
                    let mut error = 0;
                    let ptr = unsafe {
                        self.api().$func(self.handle(), key.as_ptr(), &mut error)
                    };
                    debug_assert!(error == 0);

                    Ok($value(unsafe { slice::from_raw_parts(ptr, count) }))
            }}
        }

        macro_rules! get_values {
            ($func:ident, $value:path, $process:expr) => (
                Ok($value(
                    (0..count as i32)
                        .map(|index| {
                            let mut error = 0;
                            let value = unsafe {
                                self.api().$func(self.handle(), key.as_ptr(), index, &mut error)
                            };
                            debug_assert!(error == 0);
                            (index, value)
                        })
                        .map($process)
                        .collect()
                ))
            )
        }

        match self.value_type(key) {
            ffi::VSPropTypes::ptUnset => Err(Error::KeyNotFound),

            #[cfg(feature = "gte-vapoursynth-api-31")]
            ffi::VSPropTypes::ptInt => get_value_array!(prop_get_int_array, ValueArray::Ints),
            #[cfg(feature = "gte-vapoursynth-api-31")]
            ffi::VSPropTypes::ptFloat => get_value_array!(prop_get_float_array, ValueArray::Floats),

            #[cfg(not(feature = "gte-vapoursynth-api-31"))]
            ffi::VSPropTypes::ptInt => get_values!(prop_get_int, ValueArray::Ints, |(_, x)| x),
            #[cfg(not(feature = "gte-vapoursynth-api-31"))]
            ffi::VSPropTypes::ptFloat => {
                get_values!(prop_get_float, ValueArray::Floats, |(_, x)| x)
            }

            ffi::VSPropTypes::ptData => {
                get_values!(prop_get_data, ValueArray::Data, |(index, x)| {
                    let mut error = 0;
                    let size = unsafe {
                        self.api().prop_get_data_size(
                            self.handle(),
                            key.as_ptr(),
                            index,
                            &mut error,
                        )
                    };
                    debug_assert!(error == 0);
                    debug_assert!(size >= 0);
                    unsafe { slice::from_raw_parts(x as *const u8, size as usize) }
                })
            }
            ffi::VSPropTypes::ptNode => {
                get_values!(prop_get_node, ValueArray::Nodes, |(_, x)| unsafe {
                    Node::from_ptr(self.api(), x)
                })
            }
            ffi::VSPropTypes::ptFrame => {
                get_values!(prop_get_frame, ValueArray::Frames, |(_, x)| unsafe {
                    Frame::from_ptr(self.api(), x)
                })
            }
            ffi::VSPropTypes::ptFunction => {
                get_values!(prop_get_func, ValueArray::Functions, |(_, x)| unsafe {
                    Function::from_ptr(self.api(), x)
                })
            }
        }
    }

    /// Returns an iterator over the entries.
    fn iter(&self) -> Iter<Self>
    where
        Self: Sized,
    {
        Iter::new(self)
    }

    /// Returns a `MapRef` to this map.
    #[inline]
    fn get_ref(&self) -> MapRef {
        unsafe { MapRef::from_ptr(self.api(), self.handle()) }
    }
}

impl<'owner, 'map> From<&'map MapRef<'owner>> for HashMap<&'map CStr, ValueArray<'map>> {
    fn from(x: &'map MapRef<'owner>) -> Self {
        x.iter().collect()
    }
}

impl<'owner, 'map> From<&'map MapRefMut<'owner>> for HashMap<&'map CStr, ValueArray<'map>> {
    fn from(x: &'map MapRefMut<'owner>) -> Self {
        x.iter().collect()
    }
}

impl<'map> From<&'map Map> for HashMap<&'map CStr, ValueArray<'map>> {
    fn from(x: &'map Map) -> Self {
        x.iter().collect()
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

    /// Returns a `MapRefMut` to this map.
    #[inline]
    fn get_ref_mut(&mut self) -> MapRefMut {
        unsafe { MapRefMut::from_ptr(self.api(), self.handle_mut()) }
    }
}

// Do this manually for each type so it shows up in rustdoc
impl<'a> VSMap for MapRef<'a> {}
impl<'a> VSMap for MapRefMut<'a> {}
impl<'a> VSMapMut for MapRefMut<'a> {}
impl VSMap for Map {}
impl VSMapMut for Map {}

pub(crate) use self::sealed::*;

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
