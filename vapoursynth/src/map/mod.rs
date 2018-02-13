use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::{ptr, result, slice};
use vapoursynth_sys as ffi;

use api::API;
use frame::Frame;
use function::Function;
use node::Node;

mod errors;
pub use self::errors::{Error, InvalidKeyError};
use self::errors::Result;

mod iterators;
pub use self::iterators::{Iter, Keys};

mod value;
pub use self::value::{Value, ValueArray, ValueRef, ValueType, Values};

// TODO: impl Eq on this stuff.
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

impl Clone for Map {
    fn clone(&self) -> Self {
        let mut map = Map::new(self.api);

        for i in 0..self.key_count() {
            let key = self.key_raw(i);
            let value = unsafe { self.values_raw_unchecked(key).unwrap() };

            // TODO: this is stupid.
            match value {
                ValueArray::Ints(xs) => unsafe {
                    map.set_values_raw_unchecked(key, Values::IntArray(xs));
                },
                ValueArray::Floats(xs) => unsafe {
                    map.set_values_raw_unchecked(key, Values::FloatArray(xs));
                },
                ValueArray::Data(xs) => unsafe {
                    map.set_values_raw_unchecked(key, Values::Data(&mut xs.iter().map(|&x| x)));
                },
                ValueArray::Nodes(xs) => unsafe {
                    map.set_values_raw_unchecked(key, Values::Nodes(&mut xs.iter()));
                },
                ValueArray::Frames(xs) => unsafe {
                    map.set_values_raw_unchecked(key, Values::Frames(&mut xs.iter()));
                },
                ValueArray::Functions(xs) => unsafe {
                    map.set_values_raw_unchecked(key, Values::Functions(&mut xs.iter()));
                },
            }
        }

        map
    }
}

impl Map {
    /// Creates a new `Map`.
    #[inline]
    pub fn new(api: API) -> Self {
        let handle = api.create_map();

        Self { api, handle }
    }

    /// Checks if the key is valid. Valid keys start with an alphabetic character or an underscore,
    /// and contain only alphanumeric characters and underscores.
    pub fn is_key_valid(key: &str) -> result::Result<(), InvalidKeyError> {
        if key.is_empty() {
            return Err(InvalidKeyError::EmptyKey);
        }

        // TODO: use `AsciiExt` stuff when it gets stabilized.
        fn is_alpha(c: char) -> bool {
            (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
        }
        fn is_numeric(c: char) -> bool {
            c >= '0' && c <= '9'
        }

        let mut chars = key.chars();

        let first = chars.next().unwrap();
        if !is_alpha(first) && first != '_' {
            return Err(InvalidKeyError::InvalidCharacter(0));
        }

        for (i, c) in chars.enumerate() {
            if !is_alpha(c) && !is_numeric(c) && c != '_' {
                return Err(InvalidKeyError::InvalidCharacter(i + 1));
            }
        }

        Ok(())
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
    fn key_raw(&self, index: usize) -> &CStr {
        assert!(index < self.key_count());
        let index = index as i32;

        unsafe { CStr::from_ptr(self.api().prop_get_key(self.handle(), index)) }
    }

    /// Returns a key from a map.
    ///
    /// # Panics
    /// Panics if `index >= self.key_count()`.
    #[inline]
    fn key(&self, index: usize) -> &str {
        self.key_raw(index).to_str().unwrap()
    }

    /// Returns an iterator over all keys in a map.
    #[inline]
    fn keys(&self) -> Keys<Self>
    where
        Self: Sized,
    {
        Keys::new(self)
    }

    /// Returns the number of elements associated with a key in a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    unsafe fn value_count_raw_unchecked(&self, key: &CStr) -> Result<usize> {
        let rv = self.api().prop_num_elements(self.handle(), key.as_ptr());
        if rv == -1 {
            Err(Error::KeyNotFound)
        } else {
            debug_assert!(rv >= 0);
            Ok(rv as usize)
        }
    }

    /// Returns the number of elements associated with a key in a map.
    #[inline]
    fn value_count(&self, key: &str) -> Result<usize> {
        Map::is_key_valid(key)?;
        let key = CString::new(key).unwrap();
        unsafe { self.value_count_raw_unchecked(&key) }
    }

    /// Retrieves a value type from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    unsafe fn value_type_raw_unchecked(&self, key: &CStr) -> Result<ValueType> {
        match self.api().prop_get_type(self.handle(), key.as_ptr()) {
            x if x == ffi::VSPropTypes::ptUnset as c_char => Err(Error::KeyNotFound),
            x if x == ffi::VSPropTypes::ptInt as c_char => Ok(ValueType::Int),
            x if x == ffi::VSPropTypes::ptFloat as c_char => Ok(ValueType::Float),
            x if x == ffi::VSPropTypes::ptData as c_char => Ok(ValueType::Data),
            x if x == ffi::VSPropTypes::ptNode as c_char => Ok(ValueType::Node),
            x if x == ffi::VSPropTypes::ptFrame as c_char => Ok(ValueType::Frame),
            x if x == ffi::VSPropTypes::ptFunction as c_char => Ok(ValueType::Function),
            _ => unreachable!(),
        }
    }

    /// Retrieves a value type from a map.
    #[inline]
    fn value_type(&self, key: &str) -> Result<ValueType> {
        Map::is_key_valid(key)?;
        let key = CString::new(key).unwrap();
        unsafe { self.value_type_raw_unchecked(&key) }
    }

    /// Retrieves a value from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid and `index >= 0`.
    unsafe fn value_raw_unchecked(&self, key: &CStr, index: i32) -> Result<Value> {
        macro_rules! get_value {
            ($func:ident, $value:path, $process:expr) => {{
                let mut error = 0;
                let value = self.api().$func(self.handle(), key.as_ptr(), index, &mut error);

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

        match self.value_type_raw_unchecked(key)? {
            ValueType::Int => get_value!(prop_get_int, Value::Int, |x| x),
            ValueType::Float => get_value!(prop_get_float, Value::Float, |x| x),
            ValueType::Data => get_value!(prop_get_data, Value::Data, |x| {
                let mut error = 0;
                let size =
                    self.api()
                        .prop_get_data_size(self.handle(), key.as_ptr(), index, &mut error);
                debug_assert!(error == 0);
                debug_assert!(size >= 0);
                slice::from_raw_parts(x as *const u8, size as usize)
            }),
            ValueType::Node => get_value!(prop_get_node, Value::Node, |x| Node::from_ptr(
                self.api(),
                x
            )),
            ValueType::Frame => get_value!(prop_get_frame, Value::Frame, |x| Frame::from_ptr(
                self.api(),
                x
            )),
            ValueType::Function => get_value!(prop_get_func, Value::Function, |x| {
                Function::from_ptr(self.api(), x)
            }),
        }
    }

    /// Retrieves a value from a map.
    ///
    /// # Panics
    /// Panics if `index > i32::max_value()`.
    #[inline]
    fn value(&self, key: &str, index: usize) -> Result<Value> {
        Map::is_key_valid(key)?;
        let key = CString::new(key).unwrap();

        assert!(index <= i32::max_value() as usize);
        let index = index as i32;

        unsafe { self.value_raw_unchecked(&key, index) }
    }

    /// Retrieves all values for a given key from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    unsafe fn values_raw_unchecked(&self, key: &CStr) -> Result<ValueArray> {
        let count = self.value_count_raw_unchecked(key)?;

        #[cfg(feature = "gte-vapoursynth-api-31")]
        macro_rules! get_value_array {
            ($func:ident, $value:path) => {{
                    let mut error = 0;
                    let ptr = self.api().$func(self.handle(), key.as_ptr(), &mut error);
                    debug_assert!(error == 0);

                    Ok($value(slice::from_raw_parts(ptr, count)))
            }}
        }

        macro_rules! get_values {
            ($func:ident, $value:path, $process:expr) => (
                Ok($value(
                    (0..count as i32)
                        .map(|index| {
                            let mut error = 0;
                            let value =
                                self.api().$func(self.handle(), key.as_ptr(), index, &mut error);
                            debug_assert!(error == 0);
                            (index, value)
                        })
                        .map($process)
                        .collect()
                ))
            )
        }

        match self.value_type_raw_unchecked(key)? {
            #[cfg(feature = "gte-vapoursynth-api-31")]
            ValueType::Int => get_value_array!(prop_get_int_array, ValueArray::Ints),
            #[cfg(feature = "gte-vapoursynth-api-31")]
            ValueType::Float => get_value_array!(prop_get_float_array, ValueArray::Floats),

            #[cfg(not(feature = "gte-vapoursynth-api-31"))]
            ValueType::Int => get_values!(prop_get_int, ValueArray::Ints, |(_, x)| x),
            #[cfg(not(feature = "gte-vapoursynth-api-31"))]
            ValueType::Float => get_values!(prop_get_float, ValueArray::Floats, |(_, x)| x),

            ValueType::Data => get_values!(prop_get_data, ValueArray::Data, |(index, x)| {
                let mut error = 0;
                let size =
                    self.api()
                        .prop_get_data_size(self.handle(), key.as_ptr(), index, &mut error);
                debug_assert!(error == 0);
                debug_assert!(size >= 0);
                slice::from_raw_parts(x as *const u8, size as usize)
            }),
            ValueType::Node => get_values!(prop_get_node, ValueArray::Nodes, |(_, x)| {
                Node::from_ptr(self.api(), x)
            }),
            ValueType::Frame => get_values!(prop_get_frame, ValueArray::Frames, |(_, x)| {
                Frame::from_ptr(self.api(), x)
            }),
            ValueType::Function => get_values!(prop_get_func, ValueArray::Functions, |(_, x)| {
                Function::from_ptr(self.api(), x)
            }),
        }
    }

    /// Retrieves all values for a given key from a map.
    #[inline]
    fn values(&self, key: &str) -> Result<ValueArray> {
        Map::is_key_valid(key)?;
        let key = CString::new(key).unwrap();
        unsafe { self.values_raw_unchecked(&key) }
    }

    /// Returns an iterator over the entries.
    #[inline]
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

impl<'owner, 'map> From<&'map MapRef<'owner>> for HashMap<&'map str, ValueArray<'map>> {
    fn from(x: &'map MapRef<'owner>) -> Self {
        x.iter().collect()
    }
}

impl<'owner, 'map> From<&'map MapRefMut<'owner>> for HashMap<&'map str, ValueArray<'map>> {
    fn from(x: &'map MapRefMut<'owner>) -> Self {
        x.iter().collect()
    }
}

impl<'map> From<&'map Map> for HashMap<&'map str, ValueArray<'map>> {
    fn from(x: &'map Map) -> Self {
        x.iter().collect()
    }
}

/// A mutable VapourSynth map interface.
///
/// This trait is sealed and is not meant for implementation outside of this crate.
pub trait VSMapMut: VSMap + sealed::VSMapMutInterface {
    /// Returns a `MapRefMut` to this map.
    #[inline]
    fn get_ref_mut(&mut self) -> MapRefMut {
        unsafe { MapRefMut::from_ptr(self.api(), self.handle_mut()) }
    }

    /// Clears the map.
    #[inline]
    fn clear(&mut self) {
        unsafe {
            self.api().clear_map(self.handle_mut());
        }
    }

    /// Deletes the given key.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    unsafe fn delete_key_raw_unchecked(&mut self, key: &CStr) -> Result<()> {
        let result = self.api().prop_delete_key(self.handle_mut(), key.as_ptr());
        if result == 0 {
            Err(Error::KeyNotFound)
        } else {
            debug_assert!(result == 1);
            Ok(())
        }
    }

    /// Deletes the given key.
    #[inline]
    fn delete_key(&mut self, key: &str) -> Result<()> {
        Map::is_key_valid(key)?;
        let key = CString::new(key).unwrap();
        unsafe { self.delete_key_raw_unchecked(&key) }
    }

    /// Sets the property value.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    ///
    /// # Panics
    /// Panics if `value` is `Value::Data(v)` and `v.len()` can't fit in an `i32`.
    unsafe fn set_value_raw_unchecked(&mut self, key: &CStr, value: ValueRef) {
        macro_rules! set_value {
            ($func:ident, $value:expr) => ({
                let result = self.api().$func(
                    self.handle_mut(),
                    key.as_ptr(),
                    $value,
                    ffi::VSPropAppendMode::paReplace
                );
                debug_assert!(result == 0);
            })
        }

        match value {
            ValueRef::Int(x) => set_value!(prop_set_int, x),
            ValueRef::Float(x) => set_value!(prop_set_float, x),
            ValueRef::Data(x) => set_value!(prop_set_data, x),
            ValueRef::Node(x) => set_value!(prop_set_node, x.ptr()),
            ValueRef::Frame(x) => set_value!(prop_set_frame, x.ptr()),
            ValueRef::Function(x) => set_value!(prop_set_func, x.ptr()),
        }
    }

    /// Sets the property value.
    ///
    /// # Panics
    /// Panics if `value` is `Value::Data(v)` and `v.len()` can't fit in an `i32`.
    #[inline]
    fn set_value(&mut self, key: &str, value: ValueRef) -> Result<()> {
        Map::is_key_valid(key)?;
        let key = CString::new(key).unwrap();
        unsafe {
            self.set_value_raw_unchecked(&key, value);
        }
        Ok(())
    }

    /// Sets the property value to an array of values.
    ///
    /// When using VapourSynth API >= R3.1, this performs better on integer and floating point
    /// arrays than calling `set_value()` in a loop.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    ///
    /// # Panics
    /// Panics if `values` contains `Data`, and one of the entries' length can't fit into an `i32`,
    /// and if `values` contains an `IntArray` or a `FloatArray`, and its length can't fit into an
    /// `i32`.
    unsafe fn set_values_raw_unchecked(&mut self, key: &CStr, values: Values) {
        macro_rules! set_values {
            ($iter:expr, $value:ident) => ({
                let first = $iter.next();
                if first.is_none() {
                    self.touch_raw_unchecked(&key, ValueType::$value);
                } else {
                    let first = first.unwrap();
                    self.set_value_raw_unchecked(&key, ValueRef::$value(first));

                    for x in $iter {
                        let result = self.append_value_raw_unchecked(&key, ValueRef::$value(x));
                        debug_assert!(result.is_ok());
                    }
                }
            })
        }

        match values {
            #[cfg(feature = "gte-vapoursynth-api-31")]
            Values::IntArray(xs) => {
                let result = self.api()
                    .prop_set_int_array(self.handle_mut(), key.as_ptr(), xs);
                debug_assert!(result == 0);
            }
            #[cfg(feature = "gte-vapoursynth-api-31")]
            Values::FloatArray(xs) => {
                let result = self.api()
                    .prop_set_float_array(self.handle_mut(), key.as_ptr(), xs);
                debug_assert!(result == 0);
            }

            #[cfg(not(feature = "gte-vapoursynth-api-31"))]
            Values::IntArray(xs) => set_values!(xs.iter().cloned(), Int),
            #[cfg(not(feature = "gte-vapoursynth-api-31"))]
            Values::FloatArray(xs) => set_values!(xs.iter().cloned(), Float),

            Values::Ints(xs) => set_values!(xs, Int),
            Values::Floats(xs) => set_values!(xs, Float),
            Values::Data(xs) => set_values!(xs, Data),
            Values::Nodes(xs) => set_values!(xs, Node),
            Values::Frames(xs) => set_values!(xs, Frame),
            Values::Functions(xs) => set_values!(xs, Function),
        }
    }

    /// Sets the property value to an array of values.
    ///
    /// When using VapourSynth API >= R3.1, this performs better on integer and floating point
    /// arrays than calling `set_value()` in a loop.
    ///
    /// # Panics
    /// Panics if `values` contains `Data`, and one of the entries' length can't fit into an `i32`,
    /// and if `values` contains an `IntArray` or a `FloatArray`, and its length can't fit into an
    /// `i32`.
    fn set_values(&mut self, key: &str, values: Values) -> Result<()> {
        Map::is_key_valid(key)?;
        let key = CString::new(key).unwrap();
        unsafe {
            self.set_values_raw_unchecked(&key, values);
        }
        Ok(())
    }

    /// Appends the value to the property with the given key.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    ///
    /// # Panics
    /// Panics if `value` is `Value::Data(v)` and `v.len()` can't fit in an `i32`.
    unsafe fn append_value_raw_unchecked(&mut self, key: &CStr, value: ValueRef) -> Result<()> {
        macro_rules! append_value {
            ($func:ident, $value:expr) => ({
                let result = self.api().$func(
                    self.handle_mut(),
                    key.as_ptr(),
                    $value,
                    ffi::VSPropAppendMode::paAppend
                );
                if result != 0 {
                    debug_assert!(result == 1);
                    return Err(Error::WrongValueType);
                }
            })
        }

        match value {
            ValueRef::Int(x) => append_value!(prop_set_int, x),
            ValueRef::Float(x) => append_value!(prop_set_float, x),
            ValueRef::Data(x) => append_value!(prop_set_data, x),
            ValueRef::Node(x) => append_value!(prop_set_node, x.ptr()),
            ValueRef::Frame(x) => append_value!(prop_set_frame, x.ptr()),
            ValueRef::Function(x) => append_value!(prop_set_func, x.ptr()),
        }

        Ok(())
    }

    /// Appends the value to the property with the given key.
    ///
    /// # Panics
    /// Panics if `value` is `Value::Data(v)` and `v.len()` can't fit in an `i32`.
    #[inline]
    fn append_value(&mut self, key: &str, value: ValueRef) -> Result<()> {
        Map::is_key_valid(key)?;
        let key = CString::new(key).unwrap();
        unsafe { self.append_value_raw_unchecked(&key, value) }
    }

    /// Touches the key. That is, if the key exists, nothing happens, otherwise a key is created
    /// with no values associated.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    unsafe fn touch_raw_unchecked(&mut self, key: &CStr, value_type: ValueType) {
        macro_rules! touch_value {
            ($func:ident, $value:expr) => ({
                let result = self.api().$func(
                    self.handle_mut(),
                    key.as_ptr(),
                    $value,
                    ffi::VSPropAppendMode::paTouch
                );
                debug_assert!(result == 0);
            })
        }

        match value_type {
            ValueType::Int => touch_value!(prop_set_int, 0),
            ValueType::Float => touch_value!(prop_set_float, 0f64),
            ValueType::Data => touch_value!(prop_set_data, &[]),
            ValueType::Node => touch_value!(prop_set_node, ptr::null_mut()),
            ValueType::Frame => touch_value!(prop_set_frame, ptr::null()),
            ValueType::Function => touch_value!(prop_set_func, ptr::null_mut()),
        }
    }

    /// Touches the key. That is, if the key exists, nothing happens, otherwise a key is created
    /// with no values associated.
    #[inline]
    fn touch(&mut self, key: &str, value_type: ValueType) -> Result<()> {
        Map::is_key_valid(key)?;
        let key = CString::new(key).unwrap();
        unsafe {
            self.touch_raw_unchecked(&key, value_type);
        }
        Ok(())
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
